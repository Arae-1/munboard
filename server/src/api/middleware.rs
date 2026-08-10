use crate::api::error::{ApiError, ApiResult};
use crate::app::AppState;
use crate::auth::{Client as AuthClient, header};
use crate::model::enums::UserRank;
use crate::model::snapshot::Snapshot;
use crate::schema::{snapshot, user, user_token};
use crate::string::SmallString;
use crate::time::DateTime;
use crate::update;
use axum::extract::{Request, State};
use axum::http::Method;
use axum::http::header::{AUTHORIZATION, RETRY_AFTER};
use axum::middleware::Next;
use axum::response::{IntoResponse, Json, Response};
use axum::http::StatusCode;
use diesel::{
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper,
};
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, REFERER};
use reqwest::redirect;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::atomic::{AtomicI64, Ordering};
use tracing::{error, warn};
use url::Url;

/// Logs errors from handlers.
pub async fn log_error(req: Request, next: Next) -> Response {
    let response = next.run(req).await;

    if let Some(err) = response.extensions().get::<Arc<ApiError>>() {
        error!(
            status = err.status_code().as_u16(),
            error = %err.to_string(),
            "request failed with"
        );
    }
    response
}

/// Attempts to authorizes user by either username/password or user token.
pub async fn auth(State(state): State<AppState>, mut request: Request, next: Next) -> ApiResult<Response> {
    let auth_header = request.headers().get(AUTHORIZATION);
    let authenticated = if let Some(auth_value) = auth_header {
        let auth_str = auth_value.to_str()?;
        match header::authenticate_user(&state, auth_str).await {
            Ok(authenticated) => Some(authenticated),
            Err(header::AuthenticateError::RateLimited { retry_after_secs }) => {
                return Ok(rate_limited_response(retry_after_secs));
            }
            Err(header::AuthenticateError::Auth(err)) => return Err(err.into()),
        }
    } else {
        None
    };
    let client = authenticated
        .as_ref()
        .map(|authenticated| authenticated.client)
        .unwrap_or_else(|| AuthClient::new(None, UserRank::Anonymous));

    // If client is not anonymous and the query contains the exact "bump-login"
    // parameter, update the login time (and the token usage time for token auth).
    if let Some(user_id) = client.id
        && has_bump_login_param(request.uri().query())
    {
        let mut conn = state.connection_pool.get().await?;
        update::user::last_login_time(conn.as_mut(), user_id)?;
        if let Some(token_hash) = authenticated.as_ref().and_then(|authenticated| authenticated.token_hash.as_deref()) {
            diesel::update(user_token::table)
                .filter(user_token::token_hash.eq(token_hash))
                .set(user_token::last_usage_time.eq(DateTime::now()))
                .execute(conn.as_mut())?;
        }
    }

    request.extensions_mut().insert(client);
    Ok(next.run(request).await)
}

/// Returns true when the query string contains the exact `bump-login` parameter.
fn has_bump_login_param(query: Option<&str>) -> bool {
    query.is_some_and(|query| url::form_urlencoded::parse(query.as_bytes()).any(|(key, _)| key == "bump-login"))
}

/// Answers 429 with a `Retry-After` header for a rate-limited username.
fn rate_limited_response(retry_after_secs: u64) -> Response {
    let response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({
            "name": "RateLimited",
            "title": "Failed Authentication",
            "description": "Too many failed login attempts. Try again later.",
        })),
    )
        .into_response();
    let mut response = response;
    response
        .headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from_str(&retry_after_secs.to_string()).expect("Retry-After must be a valid header"));
    response
}

/// Sends snapshot data to webhook URLs after modifying requests.
pub async fn post_to_webhooks(State(state): State<AppState>, request: Request, next: Next) -> ApiResult<Response> {
    let can_modify_database = matches!(request.method(), &Method::POST | &Method::PUT | &Method::DELETE);
    let user_ip = if state.config.trusted_proxy {
        request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|v| v.trim().to_string())
            .or_else(|| {
                request
                    .headers()
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_string())
            })
    } else {
        None
    };

    let response = next.run(request).await;

    if can_modify_database {
        let mut conn = state.connection_pool.get().await?;
        let last_posted_snapshot = LAST_POSTED_SNAPSHOT.load(Ordering::SeqCst);
        let new_snapshots = snapshot::table
            .select(Snapshot::as_select())
            .filter(snapshot::id.gt(last_posted_snapshot))
            .order(snapshot::id)
            .load(conn.as_mut())?;

        let user_ids: Vec<i64> = new_snapshots
            .iter()
            .filter_map(|s| s.user_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let user_names: HashMap<i64, String> = if !user_ids.is_empty() {
            user::table
                .select((user::id, user::name))
                .filter(user::id.eq_any(&user_ids))
                .load::<(i64, SmallString)>(conn.as_mut())?
                .into_iter()
                .map(|(id, name)| (id, name.to_string()))
                .collect()
        } else {
            HashMap::new()
        };

        for snapshot in new_snapshots {
            let user_name = snapshot.user_id.and_then(|uid| user_names.get(&uid).cloned());
            post_snapshot(&state, snapshot, user_ip.clone(), user_name);
        }
    }

    Ok(response)
}

/// Initializes last posted snapshot counter using highest ID snapshot
pub fn initialize_snapshot_counter(conn: &mut PgConnection) -> QueryResult<()> {
    let latest_snapshot_id = snapshot::table
        .select(snapshot::id)
        .order(snapshot::id.desc())
        .first(conn)
        .optional()
        .map(Option::unwrap_or_default)?;
    LAST_POSTED_SNAPSHOT.store(latest_snapshot_id, Ordering::SeqCst);
    Ok(())
}

static LAST_POSTED_SNAPSHOT: AtomicI64 = AtomicI64::new(i64::MAX);

/// Sends `snapshot` data to webhooks if it hasn't already been posted by another thread.
fn post_snapshot(state: &AppState, snapshot: Snapshot, user_ip: Option<String>, user_name: Option<String>) {
    loop {
        let last_posted_snapshot = LAST_POSTED_SNAPSHOT.load(Ordering::SeqCst);
        if snapshot.id <= last_posted_snapshot {
            return;
        }

        if LAST_POSTED_SNAPSHOT
            .compare_exchange(last_posted_snapshot, snapshot.id, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            break;
        }
    }

    let snapshot = Arc::new(snapshot);
    for url in &state.config.webhooks {
        tokio::spawn(post_to_webhook(url.clone(), Arc::clone(&snapshot), user_ip.clone(), user_name.clone()));
    }
}

#[derive(serde::Serialize)]
struct WebhookPayload<'a> {
    #[serde(flatten)]
    snapshot: &'a Snapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_name: Option<String>,
}

/// Sends `snapshot` data to given `url`.
async fn post_to_webhook(url: Url, snapshot: Arc<Snapshot>, user_ip: Option<String>, user_name: Option<String>) {
    const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");
    let post = async || {
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_str(url.as_str())?);
        headers.insert(CONTENT_TYPE, APPLICATION_JSON);

        let payload = WebhookPayload {
            snapshot: &snapshot,
            user_ip: user_ip.clone(),
            user_name: user_name.clone(),
        };

        let client = Client::builder()
            .default_headers(headers)
            .redirect(redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()?;
        let response = client.post(url.clone()).json(&payload).send().await?;
        response.error_for_status().map(|_| ()).map_err(ApiError::from)
    };

    if let Err(err) = post().await {
        warn!("Could not post snapshot to {url}. Details:\n{err}");
    }
}

#[cfg(test)]
mod test {
    use crate::api;
    use crate::api::error::ApiResult;
    use crate::auth::{header, token_crypto};
    use crate::schema::{user, user_token};
    use crate::test::*;
    use crate::time::DateTime;
    use axum::ServiceExt;
    use axum::extract::Request;
    use axum::http::header::AUTHORIZATION;
    use axum_test::TestServer;
    use diesel::{ExpressionMethods, PgConnection, QueryDsl, QueryResult, RunQueryDsl};
    use serde_json::json;
    use serial_test::{parallel, serial};
    use tower::layer::Layer;
    use tower_http::normalize_path::NormalizePathLayer;
    use uuid::{Uuid, uuid};

    #[tokio::test]
    #[parallel]
    async fn unauthorized() -> ApiResult<()> {
        const QUERY: &str = "GET /comment/1";
        const FAKE_TOKEN: Uuid = uuid!("ef94b18f-3244-4394-a529-5ea86b6fde93");

        let wrong_password = Some(header::basic_credentials_for("regular_user", "password123"));
        verify_response_with_credentials(wrong_password, QUERY, "middleware/wrong_password").await?;

        let wrong_username_with_basic = Some(header::basic_credentials_for("mystery_man29", TEST_PASSWORD));
        verify_response_with_credentials(wrong_username_with_basic, QUERY, "middleware/wrong_username_with_basic")
            .await?;

        let wrong_username_with_token = Some(header::token_credentials_for("fake_user", TEST_TOKEN));
        verify_response_with_credentials(wrong_username_with_token, QUERY, "middleware/wrong_username_with_token")
            .await?;

        let username_token_mismatch = Some(header::token_credentials_for("regular_user", TEST_TOKEN));
        verify_response_with_credentials(username_token_mismatch, QUERY, "middleware/username_token_mismatch").await?;

        let wrong_token = Some(header::token_credentials_for("regular_user", FAKE_TOKEN));
        verify_response_with_credentials(wrong_token, QUERY, "middleware/wrong_token").await?;

        // Insert expired and disabled tokens with their hashes so the lookup
        // succeeds and the expiry/disabled checks are exercised.
        let mut conn = get_connection()?;
        let token_key = get_state().config.token_encryption_key();
        let expired_token = Uuid::new_v4();
        diesel::insert_into(user_token::table)
            .values((
                user_token::id.eq(expired_token),
                user_token::user_id.eq(2),
                user_token::note.eq("expired token"),
                user_token::enabled.eq(true),
                user_token::expiration_time.eq(Some(DateTime::test_date())),
                user_token::token_hash.eq(header::hash_token_hex(expired_token.as_bytes())),
                user_token::token_secret.eq(token_crypto::encrypt_blob(&token_key, &expired_token.to_string())),
            ))
            .execute(&mut conn)?;
        let disabled_token = Uuid::new_v4();
        diesel::insert_into(user_token::table)
            .values((
                user_token::id.eq(disabled_token),
                user_token::user_id.eq(2),
                user_token::note.eq("disabled token"),
                user_token::enabled.eq(false),
                user_token::expiration_time.eq(None::<DateTime>),
                user_token::token_hash.eq(header::hash_token_hex(disabled_token.as_bytes())),
                user_token::token_secret.eq(token_crypto::encrypt_blob(&token_key, &disabled_token.to_string())),
            ))
            .execute(&mut conn)?;

        let expired_token = Some(header::token_credentials_for("regular_user", expired_token));
        verify_response_with_credentials(expired_token, QUERY, "middleware/expired_token").await?;

        let disabled_token = Some(header::token_credentials_for("regular_user", disabled_token));
        verify_response_with_credentials(disabled_token, QUERY, "middleware/disabled_token").await?;

        let missing_credentials = Some(String::new());
        verify_response_with_credentials(missing_credentials, QUERY, "middleware/missing_credentials").await?;

        let unencoded_credentials = Some(format!("Basic regular_user:{TEST_PASSWORD}"));
        verify_response_with_credentials(unencoded_credentials, QUERY, "middleware/unencoded_credentials").await?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn token_auth_bumps_last_usage_time() -> ApiResult<()> {
        let app_state = get_state();
        let (router, _) = api::routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");
        let auth = header::basic_credentials_for("regular_user", TEST_PASSWORD);

        // Mint a token through the API: the create response carries the token
        // and its internal id.
        let response = server
            .post("/user-token/regular_user/?fields=token,id")
            .add_header(AUTHORIZATION, &auth)
            .json(&json!({"note": "bump test"}))
            .await;
        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_slice(response.as_bytes())?;
        let token: Uuid = serde_json::from_value(body["token"].clone()).expect("create response must include the token");
        let id: Uuid = serde_json::from_value(body["id"].clone()).expect("create response must include the token id");

        let get_usage_time = |conn: &mut PgConnection| -> QueryResult<DateTime> {
            user_token::table
                .select(user_token::last_usage_time)
                .find(id)
                .first(conn)
        };
        let mut conn = get_connection()?;
        let initial_usage = get_usage_time(&mut conn)?;

        // Login without bump-login: the token usage time is not touched.
        let token_auth = header::token_credentials_for("regular_user", token);
        let response = server.get("/comment/1").add_header(AUTHORIZATION, &token_auth).await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(get_usage_time(&mut conn)?, initial_usage);

        // Login with bump-login: the token usage time is updated.
        let response = server
            .get("/comment/1?bump-login=true")
            .add_header(AUTHORIZATION, &token_auth)
            .await;
        assert_eq!(response.status_code(), 200);
        assert!(get_usage_time(&mut conn)? > initial_usage);

        // A value merely containing "bump-login" must not trigger the update.
        let response = server
            .get("/comment/1?note=bump-login")
            .add_header(AUTHORIZATION, &token_auth)
            .await;
        assert_eq!(response.status_code(), 200);
        let usage = get_usage_time(&mut conn)?;
        let response = server
            .get("/comment/1?bump-login=true")
            .add_header(AUTHORIZATION, &token_auth)
            .await;
        assert_eq!(response.status_code(), 200);
        assert!(get_usage_time(&mut conn)? > usage);

        // Cleanup
        let response = server
            .delete(&format!("/user-token/regular_user/{id}"))
            .add_header(AUTHORIZATION, &auth)
            .await;
        assert_eq!(response.status_code(), 200);
        Ok(())
    }

    /// Creates a throwaway user via anonymous self-registration. A unique name
    /// keeps the shared failed-login map off the seeded users' keys.
    async fn create_limiter_victim(server: &TestServer) -> (String, &'static str) {
        let username = format!("limiter_victim_{}", &Uuid::new_v4().simple().to_string()[..12]);
        let password = TEST_PASSWORD;
        let response = server
            .post("/users")
            .json(&json!({"name": &username, "password": password, "rank": "regular"}))
            .await;
        assert_eq!(response.status_code(), 200);
        (username, password)
    }

    /// Deletes a throwaway user authenticated as itself (user_delete_self = regular).
    async fn delete_limiter_victim(server: &TestServer, username: &str, auth: &str) -> ApiResult<()> {
        let mut conn = get_connection()?;
        let dt: DateTime = user::table
            .select(user::last_edit_time)
            .filter(user::name.eq(username))
            .first(&mut conn)?;
        let response = server
            .delete(&format!("/user/{username}"))
            .add_header(AUTHORIZATION, auth)
            .json(&json!({"version": dt}))
            .await;
        assert_eq!(response.status_code(), 200);
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn basic_auth_correct_password_works_during_rate_limit() -> ApiResult<()> {
        let app_state = get_state();
        let (router, _) = api::routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");

        let (username, password) = create_limiter_victim(&server).await;

        // Exhaust the budget with wrong passwords: `max_failures` (20) failed
        // attempts answer 401, the next one is rate-limited with 429.
        let wrong = header::basic_credentials_for(&username, "wrong_password");
        for _ in 0..20 {
            let response = server.get("/comment/1").add_header(AUTHORIZATION, &wrong).await;
            assert_eq!(response.status_code(), 401);
        }
        let response = server.get("/comment/1").add_header(AUTHORIZATION, &wrong).await;
        assert_eq!(response.status_code(), 429);
        let body: serde_json::Value = serde_json::from_slice(response.as_bytes())?;
        assert_eq!(body["name"], "RateLimited");

        // F-1 regression: the correct password must still authenticate during
        // the lockout instead of being rejected with 429.
        let correct = header::basic_credentials_for(&username, password);
        let response = server.get("/comment/1").add_header(AUTHORIZATION, &correct).await;
        assert_eq!(response.status_code(), 200);

        delete_limiter_victim(&server, &username, &correct).await
    }

    #[tokio::test]
    #[serial]
    async fn basic_auth_case_variant_shares_rate_limit() -> ApiResult<()> {
        let app_state = get_state();
        let (router, _) = api::routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");

        let (username, password) = create_limiter_victim(&server).await;

        // Lock the username out with 21 wrong-password attempts.
        let wrong = header::basic_credentials_for(&username, "wrong_password");
        for _ in 0..20 {
            let response = server.get("/comment/1").add_header(AUTHORIZATION, &wrong).await;
            assert_eq!(response.status_code(), 401);
        }
        let response = server.get("/comment/1").add_header(AUTHORIZATION, &wrong).await;
        assert_eq!(response.status_code(), 429);

        // F-2: an ALL-CAPS variant of the username shares the budget (the key
        // is normalized to lowercase). Pre-fix this would answer 401 because
        // the map key was case-sensitive.
        let caps_username = username.to_ascii_uppercase();
        let wrong_caps = header::basic_credentials_for(&caps_username, "wrong_password");
        let response = server.get("/comment/1").add_header(AUTHORIZATION, &wrong_caps).await;
        assert_eq!(response.status_code(), 429);
        let body: serde_json::Value = serde_json::from_slice(response.as_bytes())?;
        assert_eq!(body["name"], "RateLimited");

        // The correct password also authenticates through the normalized key
        // (F-1), clearing the shared history.
        let correct_caps = header::basic_credentials_for(&caps_username, password);
        let response = server.get("/comment/1").add_header(AUTHORIZATION, &correct_caps).await;
        assert_eq!(response.status_code(), 200);

        delete_limiter_victim(&server, &username, &correct_caps).await
    }
}
