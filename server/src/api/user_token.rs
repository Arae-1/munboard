use crate::api::doc::USER_TOKEN_TAG;
use crate::api::error::{ApiError, ApiResult};
use crate::api::{self, ResourceParams, UnpagedResponse};
use crate::app::AppState;
use crate::auth::{header, token_crypto};
use crate::config::Action;
use crate::extract::{Ctx, Json, Path, Query};
use crate::model::enums::{AvatarStyle, ResourceOperation, ResourceType};
use crate::model::snapshot::NewSnapshot;
use crate::model::user::UserToken;
use crate::resource::user::MicroUser;
use crate::resource::user_token::{Field, UserTokenInfo, Visibility};
use crate::schema::{user, user_token};
use crate::snapshot;
use crate::string::{LargeString, SmallString};
use crate::time::DateTime;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SaveChangesDsl,
};
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

/// Tokens may not expire more than 10 years in the future.
const MAX_EXPIRATION_OFFSET_DAYS: i64 = 3650;

/// Rejects expiration times that are more than 10 years in the future.
fn validate_expiration_time(expiration_time: DateTime) -> ApiResult<DateTime> {
    let max_expiration = DateTime::from(
        time::OffsetDateTime::now_utc() + time::Duration::days(MAX_EXPIRATION_OFFSET_DAYS),
    );
    if expiration_time > max_expiration {
        return Err(ApiError::FromStr(
            "expirationTime cannot be more than 10 years in the future".to_string().into(),
        ));
    }
    Ok(expiration_time)
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(create))
        .routes(routes!(update, delete))
}

/// Searches for user tokens for the given user.
#[utoipa::path(
    get,
    path = "/user-tokens/{username}",
    tag = USER_TOKEN_TAG,
    params(
        ("username" = String, Path, description = "Username"),
        ResourceParams,
    ),
    responses(
        (status = 200, body = UnpagedResponse<UserTokenInfo>),
        (status = 403, description = "Privileges are too low"),
        (status = 404, description = "User does not exist"),
    ),
)]
async fn list(
    Ctx(ctx, connection_pool): Ctx,
    Path(username): Path<SmallString>,
    Query(params): Query<ResourceParams<Field>>,
) -> ApiResult<Json<UnpagedResponse<UserTokenInfo>>> {
    let list_any = ctx.config.privileges()[Action::UserTokenListAny];
    let list_self = ctx.config.privileges()[Action::UserTokenListSelf];

    let (avatar_style, is_self, user_tokens) = connection_pool
        .transaction({
            let username = username.clone();
            move |conn| {
                let (user_id, avatar_style): (i64, AvatarStyle) = user::table
                    .select((user::id, user::avatar_style))
                    .filter(user::name.eq(&username))
                    .first(conn)
                    .optional()?
                    .ok_or(ApiError::NotFound(ResourceType::User))?;

                let is_self = ctx.client.id == Some(user_id);
                let required_rank = if is_self { list_self } else { list_any };
                api::verify_privilege(ctx.client, required_rank)?;

                user_token::table
                    .filter(user_token::user_id.eq(user_id))
                    .order(user_token::creation_time.desc())
                    .load::<UserToken>(conn)
                    .map(|tokens| (avatar_style, is_self, tokens))
                    .map_err(ApiError::from)
            }
        })
        .await?;

    // Only the owner may see the plaintext token: decrypt it from the stored
    // ciphertext. Decrypt failures are logged and the token omitted.
    let visibility = if is_self { Visibility::Full } else { Visibility::PublicOnly };
    let token_key = ctx.config.token_encryption_key();
    let results = user_tokens
        .into_iter()
        .map(|user_token| {
            let token = match visibility {
                Visibility::Full => match token_crypto::decrypt_blob(&token_key, &user_token.token_secret) {
                    Ok(token) => Some(token),
                    Err(err) => {
                        tracing::warn!(
                            "Failed to decrypt user token {} owned by {username}: {err}",
                            user_token.id
                        );
                        None
                    }
                },
                Visibility::PublicOnly => None,
            };
            UserTokenInfo::new(
                MicroUser::new(&ctx.config, username.clone(), avatar_style),
                user_token,
                params.fields,
                visibility,
                token,
            )
        })
        .collect();
    Ok(Json(UnpagedResponse { results }))
}

/// Request body for creating a user token.
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct UserTokenCreateBody {
    /// Whether the token is enabled. Defaults to `true` if not present.
    enabled: Option<bool>,
    /// Optional note describing the token's purpose.
    note: Option<String>,
    /// Optional expiration time for the token.
    expiration_time: Option<DateTime>,
}

/// Creates a new user token for authentication.
///
/// The token can be used for authentication of API endpoints instead of a password.
#[utoipa::path(
    post,
    path = "/user-token/{username}",
    tag = USER_TOKEN_TAG,
    params(
        ("username" = String, Path, description = "Username"),
        ResourceParams,
    ),
    request_body = UserTokenCreateBody,
    responses(
        (status = 200, body = UserTokenInfo),
        (status = 403, description = "Privileges are too low"),
        (status = 404, description = "User does not exist"),
        (status = 422, description = "Expiration time is more than 10 years in the future"),
    ),
)]
async fn create(
    Ctx(ctx, connection_pool): Ctx,
    Path(username): Path<SmallString>,
    Query(params): Query<ResourceParams<Field>>,
    Json(body): Json<UserTokenCreateBody>,
) -> ApiResult<Json<UserTokenInfo>> {
    let create_any = ctx.config.privileges()[Action::UserTokenCreateAny];
    let create_self = ctx.config.privileges()[Action::UserTokenCreateSelf];
    let token_key = ctx.config.token_encryption_key();
    let id = Uuid::new_v4();
    let token = Uuid::new_v4();
    let token_hash = header::hash_token_hex(token.as_bytes());
    let token_secret = token_crypto::encrypt_blob(&token_key, &token.to_string());

    let (user_token, avatar_style) = connection_pool
        .transaction({
            let username = username.clone();
            move |conn| {
                let (user_id, avatar_style): (i64, AvatarStyle) = user::table
                    .select((user::id, user::avatar_style))
                    .filter(user::name.eq(&username))
                    .first(conn)
                    .optional()?
                    .ok_or(ApiError::NotFound(ResourceType::User))?;

                let is_self = ctx.client.id == Some(user_id);
                let required_rank = if is_self { create_self } else { create_any };
                api::verify_privilege(ctx.client, required_rank)?;

                // Delete any expired or disabled tokens owned by user
                let current_time = DateTime::now();
                diesel::delete(user_token::table)
                    .filter(user_token::user_id.eq(user_id))
                    .filter(
                        user_token::enabled
                            .eq(false)
                            .or(user_token::expiration_time.lt(current_time)),
                    )
                    .execute(conn)?;

                // Tokens created without an explicit expiration time get a sane
                // default lifetime instead of never expiring. The internal id
                // and the token are distinct UUIDs; the database stores only
                // the BLAKE3 hash and an AES-256-GCM ciphertext of the token,
                // which is returned to the client once, in the response below.
                let default_expiration = || DateTime::from(time::OffsetDateTime::now_utc() + time::Duration::days(30));
                let expiration_time = validate_expiration_time(body.expiration_time.unwrap_or_else(default_expiration))?;
                let user_token: UserToken = diesel::insert_into(user_token::table)
                    .values((
                        user_token::id.eq(id),
                        user_token::user_id.eq(user_id),
                        user_token::note.eq(body.note.as_deref().map(str::trim).unwrap_or("")),
                        user_token::enabled.eq(body.enabled.unwrap_or(true)),
                        user_token::expiration_time.eq(expiration_time),
                        user_token::token_hash.eq(token_hash.clone()),
                        user_token::token_secret.eq(token_secret),
                    ))
                    .get_result(conn)?;
                snapshot::user_token::creation_snapshot(conn, ctx.client, &user_token)?;
                Ok::<_, ApiError>((user_token, avatar_style))
            }
        })
        .await?;
    Ok(Json(UserTokenInfo::new(
        MicroUser::new(&ctx.config, username, avatar_style),
        user_token,
        params.fields,
        Visibility::Full,
        Some(token.to_string()),
    )))
}

/// Request body for updating a user token.
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct UserTokenUpdateBody {
    /// Resource version. See [versioning](#Versioning).
    version: DateTime,
    /// Whether the token is enabled.
    enabled: Option<bool>,
    /// Optional note describing the token's purpose.
    note: Option<LargeString>,
    /// Optional expiration time for the token. Set to null to remove expiration.
    #[serde(default, deserialize_with = "api::deserialize_some")]
    expiration_time: Option<Option<DateTime>>,
}

/// Updates an existing user token using specified parameters.
///
/// All fields except `version` are optional - update concerns only provided fields.
#[utoipa::path(
    put,
    path = "/user-token/{username}/{id}",
    tag = USER_TOKEN_TAG,
    params(
        ("username" = String, Path, description = "Username"),
        ("id" = Uuid, Path, description = "Internal user token id"),
        ResourceParams,
    ),
    request_body = UserTokenUpdateBody,
    responses(
        (status = 200, body = UserTokenInfo),
        (status = 403, description = "Privileges are too low"),
        (status = 404, description = "User does not exist"),
        (status = 404, description = "User token does not exist"),
        (status = 409, description = "Version is outdated"),
        (status = 422, description = "Expiration time is null or more than 10 years in the future"),
    ),
)]
async fn update(
    Ctx(ctx, connection_pool): Ctx,
    Path((username, id)): Path<(SmallString, Uuid)>,
    Query(params): Query<ResourceParams<Field>>,
    Json(body): Json<UserTokenUpdateBody>,
) -> ApiResult<Json<UserTokenInfo>> {
    let edit_any = ctx.config.privileges()[Action::UserTokenEditAny];
    let edit_self = ctx.config.privileges()[Action::UserTokenEditSelf];
    let token_key = ctx.config.token_encryption_key();

    let (updated_user_token, avatar_style, is_self) = connection_pool
        .transaction({
            let username = username.clone();
            move |conn| {
                let (user_id, avatar_style): (i64, AvatarStyle) = user::table
                    .select((user::id, user::avatar_style))
                    .filter(user::name.eq(&username))
                    .first(conn)
                    .optional()?
                    .ok_or(ApiError::NotFound(ResourceType::User))?;

                let is_self = ctx.client.id == Some(user_id);
                let required_rank = if is_self { edit_self } else { edit_any };
                api::verify_privilege(ctx.client, required_rank)?;

                let mut user_token: UserToken = user_token::table
                    .find(id)
                    .filter(user_token::user_id.eq(user_id))
                    .first(conn)
                    .optional()?
                    .ok_or(ApiError::NotFound(ResourceType::UserToken))?;
                api::verify_version(user_token.last_edit_time, body.version)?;

                let old_data = json!({
                    "note": user_token.note,
                    "enabled": user_token.enabled,
                    "expiration_time": user_token.expiration_time,
                });

                if let Some(enabled) = body.enabled {
                    user_token.enabled = enabled;
                }
                if let Some(note) = body.note {
                    user_token.note = note;
                }
                if let Some(expiration_time) = body.expiration_time {
                    // Reject an explicit null: it would permanently disable expiry.
                    let expiration_time = expiration_time.ok_or_else(|| {
                        ApiError::FromStr("expirationTime cannot be null".to_string().into())
                    })?;
                    user_token.expiration_time = Some(validate_expiration_time(expiration_time)?);
                }
                user_token.last_edit_time = DateTime::now();

                let updated_user_token: UserToken = user_token.save_changes(conn)?;

                let new_data = json!({
                    "note": updated_user_token.note,
                    "enabled": updated_user_token.enabled,
                    "expiration_time": updated_user_token.expiration_time,
                });

                if let Some(data) = snapshot::value_diff(old_data, new_data) {
                    NewSnapshot {
                        user_id: ctx.client.id,
                        operation: ResourceOperation::Modified,
                        resource_type: ResourceType::UserToken,
                        resource_id: id.to_string().into(),
                        data,
                    }
                    .insert(conn)?;
                }
                Ok::<_, ApiError>((updated_user_token, avatar_style, is_self))
            }
        })
        .await?;

    // The owner sees the plaintext token; admins editing someone else's token
    // do not. Decrypt failures are logged and the token omitted.
    let visibility = if is_self { Visibility::Full } else { Visibility::PublicOnly };
    let token = match visibility {
        Visibility::Full => match token_crypto::decrypt_blob(&token_key, &updated_user_token.token_secret) {
            Ok(token) => Some(token),
            Err(err) => {
                tracing::warn!("Failed to decrypt user token {} owned by {username}: {err}", updated_user_token.id);
                None
            }
        },
        Visibility::PublicOnly => None,
    };
    Ok(Json(UserTokenInfo::new(
        MicroUser::new(&ctx.config, username, avatar_style),
        updated_user_token,
        params.fields,
        visibility,
        token,
    )))
}

/// Deletes existing user token.
#[utoipa::path(
    delete,
    path = "/user-token/{username}/{id}",
    tag = USER_TOKEN_TAG,
    params(
        ("username" = String, Path, description = "Username"),
        ("id" = Uuid, Path, description = "Internal user token id"),
    ),
    request_body = Object,
    responses(
        (status = 200, body = Object),
        (status = 403, description = "Privileges are too low"),
        (status = 404, description = "User does not exist"),
        (status = 404, description = "Token does not exist"),
    ),
)]
async fn delete(Ctx(ctx, connection_pool): Ctx, Path((username, id)): Path<(String, Uuid)>) -> ApiResult<Json<()>> {
    connection_pool
        .transaction(move |conn| {
            let user_token_owner: i64 = user::table
                .select(user::id)
                .filter(user::name.eq(username))
                .first(conn)
                .optional()?
                .ok_or(ApiError::NotFound(ResourceType::User))?;

            let action = if ctx.client.id == Some(user_token_owner) {
                Action::UserTokenDeleteSelf
            } else {
                Action::UserTokenDeleteAny
            };
            ctx.verify_privilege(action)?;

            let user_token: UserToken = user_token::table
                .find(id)
                .filter(user_token::user_id.eq(user_token_owner))
                .first(conn)
                .optional()?
                .ok_or(ApiError::NotFound(ResourceType::UserToken))?;

            diesel::delete(user_token::table.find(id))
                .execute(conn)?;

            snapshot::user_token::deletion_snapshot(conn, ctx.client, &user_token)?;
            Ok::<_, ApiError>(Json(()))
        })
        .await
}

#[cfg(test)]
mod test {
    use crate::api;
    use crate::api::error::ApiResult;
    use crate::auth::{header, token_crypto};
    use crate::model::enums::{ResourceOperation, ResourceType, UserRank};
    use crate::model::snapshot::Snapshot;
    use crate::model::user::UserToken;
    use crate::schema::{snapshot, user, user_token};
    use crate::test::*;
    use axum::ServiceExt;
    use axum::extract::Request;
    use axum::http::header::AUTHORIZATION;
    use axum_test::TestServer;
    use diesel::dsl::exists;
    use diesel::{ExpressionMethods, PgConnection, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper};
    use serde_json::json;
    use serial_test::{parallel, serial};
    use tower::layer::Layer;
    use tower_http::normalize_path::NormalizePathLayer;
    use uuid::Uuid;

    // Exclude fields that involve token, creation_time, last_edit_time, or last_usage_time
    const FIELDS: &str = "&fields=user,note,enabled,expirationTime";

    #[tokio::test]
    #[parallel]
    async fn list() -> ApiResult<()> {
        const USER: &str = "administrator";
        verify_response(&format!("GET /user-tokens/{USER}/?{FIELDS}"), "user_token/list").await
    }

    #[tokio::test]
    #[parallel]
    async fn list_visibility() -> ApiResult<()> {
        let app_state = get_state();
        let (router, _) = api::routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");
        let auth = header::basic_credentials_for("administrator", TEST_PASSWORD);

        // The owner sees the internal id and the decrypted plaintext token.
        let response = server
            .get("/user-tokens/administrator")
            .add_header(AUTHORIZATION, &auth)
            .await;
        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_slice(response.as_bytes())?;
        let result = &body["results"][0];
        assert_eq!(result["token"], json!(TEST_TOKEN.to_string()));
        assert_eq!(result["id"], json!(TEST_TOKEN.to_string()));

        // Non-owners see the id and metadata but never the token.
        let response = server
            .get("/user-tokens/regular_user")
            .add_header(AUTHORIZATION, &auth)
            .await;
        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_slice(response.as_bytes())?;
        let result = &body["results"][0];
        assert!(result["id"].is_string());
        assert!(result.get("token").is_none());

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn create() -> ApiResult<()> {
        const USER: &str = "restricted_user";
        verify_response(&format!("POST /user-token/{USER}/?{FIELDS}"), "user_token/create").await?;

        let app_state = get_state();
        let token_key = app_state.config.token_encryption_key();
        let (router, _) = api::routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");
        let auth = header::basic_credentials_for("administrator", TEST_PASSWORD);

        // The create response carries both the internal id and the plaintext token.
        let response = server
            .post(&format!("/user-token/{USER}"))
            .add_header(AUTHORIZATION, &auth)
            .json(&json!({"note": "id and token"}))
            .await;
        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_slice(response.as_bytes())?;
        let id: Uuid = serde_json::from_value(body["id"].clone()).expect("create response must include id");
        let token: Uuid = serde_json::from_value(body["token"].clone()).expect("create response must include token");
        assert_ne!(id, token, "the internal id must not equal the token value");

        // The stored row holds the token's hash and a decryptable ciphertext.
        let mut conn = get_connection()?;
        let (token_hash, token_secret): (String, Vec<u8>) = user_token::table
            .select((user_token::token_hash, user_token::token_secret))
            .find(id)
            .first(&mut conn)?;
        assert_eq!(token_hash, header::hash_token_hex(token.as_bytes()));
        let decrypted =
            token_crypto::decrypt_blob(&token_key, &token_secret).expect("the stored ciphertext must decrypt with the config key");
        assert_eq!(decrypted, token.to_string());

        // Delete by internal id.
        verify_response(&format!("DELETE /user-token/{USER}/{id}"), "user_token/delete").await?;

        let has_token: bool = diesel::select(exists(user_token::table.find(id))).first(&mut conn)?;
        assert!(!has_token);
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn update() -> ApiResult<()> {
        const USER: &str = "administrator";
        let get_user_token = |conn: &mut PgConnection| -> QueryResult<UserToken> {
            user::table
                .inner_join(user_token::table)
                .select(UserToken::as_select())
                .filter(user::name.eq(USER))
                .first(conn)
        };

        let mut conn = get_connection()?;
        let user_token = get_user_token(&mut conn)?;

        verify_response(&format!("PUT /user-token/{USER}/{}/?{FIELDS}", user_token.id), "user_token/edit").await?;

        let edited_user_token = get_user_token(&mut conn)?;
        assert_eq!(edited_user_token.id, user_token.id);
        assert_eq!(edited_user_token.user_id, user_token.user_id);
        assert_ne!(edited_user_token.note, user_token.note);
        assert_ne!(edited_user_token.enabled, user_token.enabled);
        assert_ne!(edited_user_token.expiration_time, user_token.expiration_time);
        assert!(edited_user_token.last_edit_time > user_token.last_edit_time);
        assert_eq!(edited_user_token.last_usage_time, user_token.last_usage_time);

        verify_response(&format!("PUT /user-token/{USER}/{}/?{FIELDS}", user_token.id), "user_token/edit_restore").await?;

        let new_user_token = get_user_token(&mut conn)?;
        assert_eq!(new_user_token.id, user_token.id);
        assert_eq!(new_user_token.user_id, user_token.user_id);
        assert_eq!(new_user_token.note, user_token.note);
        assert_eq!(new_user_token.enabled, user_token.enabled);
        // An explicit null expiration is rejected (permanent non-expiry is not
        // allowed), so the expiration set by the edit is kept.
        assert_eq!(new_user_token.expiration_time, edited_user_token.expiration_time);
        assert!(new_user_token.last_edit_time > user_token.last_edit_time);
        assert_eq!(new_user_token.last_usage_time, user_token.last_usage_time);
        Ok(())
    }

    #[tokio::test]
    #[parallel]
    async fn error() -> ApiResult<()> {
        verify_response("GET /user-tokens/fake_user", "user_token/list_nonexistent_user").await?;
        verify_response("POST /user-token/fake_user", "user_token/create_nonexistent_user").await?;
        verify_response(&format!("PUT /user-token/fake_user/{TEST_TOKEN}"), "user_token/edit_nonexistent_user").await?;
        verify_response(&format!("PUT /user-token/regular_user/{TEST_TOKEN}"), "user_token/edit_nonexistent_token")
            .await?;

        // An explicit null expiration (permanent non-expiry) is rejected.
        verify_response(
            &format!("PUT /user-token/administrator/{TEST_TOKEN}/?{FIELDS}"),
            "user_token/edit_null_expiration",
        )
        .await?;
        // Expiration times beyond 10 years in the future are rejected.
        verify_response(
            &format!("PUT /user-token/administrator/{TEST_TOKEN}/?{FIELDS}"),
            "user_token/edit_far_future_expiration",
        )
        .await?;
        verify_response(&format!("DELETE /user-token/fake_user/{TEST_TOKEN}"), "user_token/delete_nonexistent_user")
            .await?;
        verify_response(
            &format!("DELETE /user-token/regular_user/{TEST_TOKEN}"),
            "user_token/delete_nonexistent_token",
        )
        .await?;

        // User has `self` permissions but not `any` permissions for user_token operations
        verify_response_with_user(UserRank::Regular, "GET /user-tokens/power_user", "user_token/list_another").await?;
        verify_response_with_user(UserRank::Regular, "POST /user-token/power_user", "user_token/create_another")
            .await?;
        verify_response_with_user(
            UserRank::Regular,
            &format!("PUT /user-token/power_user/{TEST_TOKEN}"),
            "user_token/edit_another",
        )
        .await?;
        verify_response_with_user(
            UserRank::Regular,
            &format!("DELETE /user-token/power_user/{TEST_TOKEN}"),
            "user_token/delete_another",
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn webhook_snapshots() -> ApiResult<()> {
        let app_state = get_state();
        let (router, _) = api::routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");
        let auth = header::basic_credentials_for("administrator", TEST_PASSWORD);

        let body = json!({"enabled": true, "note": "wh test"});
        let response = server
            .post("/user-token/administrator")
            .add_header(AUTHORIZATION, &auth)
            .json(&body)
            .await;
        assert_eq!(response.status_code(), 200);

        let mut conn = get_connection()?;
        let snapshot: Snapshot = snapshot::table
            .select(Snapshot::as_select())
            .order(snapshot::id.desc())
            .first(&mut conn)?;
        assert_eq!(snapshot.operation, ResourceOperation::Created);
        assert_eq!(snapshot.resource_type, ResourceType::UserToken);
        assert_eq!(snapshot.data["note"], json!("wh test"));
        assert_eq!(snapshot.data["enabled"], json!(true));
        // Tokens created without an explicit expiration time get a default
        // lifetime, so the snapshot records an RFC 3339 timestamp instead of null.
        assert!(snapshot.data["expiration_time"].is_string());

        let token: Uuid = user_token::table
            .select(user_token::id)
            .order(user_token::creation_time.desc())
            .first(&mut conn)?;

        let response = server
            .delete(&format!("/user-token/administrator/{}", token))
            .add_header(AUTHORIZATION, &auth)
            .await;
        assert_eq!(response.status_code(), 200);

        let snapshot: Snapshot = snapshot::table
            .select(Snapshot::as_select())
            .order(snapshot::id.desc())
            .first(&mut conn)?;
        assert_eq!(snapshot.operation, ResourceOperation::Deleted);
        assert_eq!(snapshot.resource_type, ResourceType::UserToken);

        Ok(())
    }
}
