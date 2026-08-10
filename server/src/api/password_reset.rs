use crate::api::doc::PASSWORD_RESET_TAG;
use crate::api::error::{ApiError, ApiResult};
use crate::app::AppState;
use crate::auth::{header, password};
use crate::content::hash;
use crate::extract::{Json, Path};
use crate::schema::{user, user_token};
use crate::string::SmallString;
use crate::time::DateTime;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::extract::State;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use lettre::Address;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use percent_encoding::NON_ALPHANUMERIC;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use subtle::ConstantTimeEq;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(request_reset, reset_password))
}

/// Sends a confirmation email to given user.
///
/// The email contains a link containing a token. The token cannot be guessed,
/// thus using such link proves that the person who requested to reset the
/// password also owns the mailbox, which is a strong indication they are the
/// rightful owner of the account.
///
/// For security, this endpoint always returns 200 regardless of whether the
/// user exists, has an email configured, or already has a pending reset token,
/// to prevent user enumeration. A still-valid pending token is never
/// overwritten and no duplicate email is sent.
#[utoipa::path(
    get,
    path = "/password-reset/{identifier}",
    tag = PASSWORD_RESET_TAG,
    params(
        ("identifier" = SmallString, Path, description = "User email or username"),
    ),
    responses(
        (status = 200, description = "Request processed", body = ()),
    ),
)]
async fn request_reset(State(state): State<AppState>, Path(identifier): Path<SmallString>) -> ApiResult<Json<()>> {
    // SMTP must be configured for a reset email to exist at all. This check is
    // config-level and identical for every identifier, so it cannot be used to
    // enumerate users.
    state.config.smtp().ok_or(ApiError::MissingSmtpInfo)?;

    // Every identifier-dependent step (user lookup, token generation, the
    // database write and the email send) happens in a background task, so the
    // response is an immediate, identical 200 regardless of whether the user
    // exists, has an email, or already has a pending reset token.
    tokio::spawn(async move {
        if let Err(err) = send_reset_email(state, identifier).await {
            tracing::warn!("Password reset request failed: {err}");
        }
    });

    Ok(Json(()))
}

/// Performs the actual password-reset request work in the background: looks up
/// the user, stores a fresh reset token and sends the confirmation email.
///
/// A still-valid pending token is never overwritten and no duplicate email is
/// sent. The no-overwrite guard and the token insert share one transaction, so
/// two rapid requests cannot both win: PostgreSQL rechecks the WHERE clause of
/// a waiting UPDATE against the committed row, only the first insert matches,
/// and the second skips the email. Errors are logged by the caller and never
/// surface in the response.
async fn send_reset_email(state: AppState, identifier: SmallString) -> ApiResult<()> {
    let smtp_info = state.config.smtp().ok_or(ApiError::MissingSmtpInfo)?;

    let mut conn = state.connection_pool.get().await?;

    let user_info = user::table
        .select((user::id, user::name, user::email, user::password_reset_token, user::password_reset_expires))
        .filter(user::name.eq(&identifier).or(user::email.eq(&identifier)))
        .first::<(i64, SmallString, Option<SmallString>, Option<String>, Option<DateTime>)>(conn.as_mut())
        .optional()?;

    let (user_id, username, user_email) = match user_info {
        Some((user_id, username, Some(email), _, _)) => (user_id, username, email),
        _ => return Ok(()),
    };

    let user_mailbox = match Address::from_str(&user_email) {
        Ok(address) => Mailbox::new(None, address),
        Err(_) => return Ok(()),
    };

    let domain = if let Some(domain) = state.config.domain.as_deref() {
        domain.to_string()
    } else if let Ok(domain) = std::env::var("HTTP_ORIGIN") {
        domain
    } else if let Ok(domain) = std::env::var("HTTP_REFERER") {
        domain
    } else if let Ok(port) = std::env::var("PORT") {
        format!("http://localhost:{port}")
    } else {
        String::new()
    };
    let domain = domain.trim_end_matches('/');

    let mut random_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut random_bytes);
    let reset_token = hash::compute_url_safe_hash(&random_bytes);
    let expires = DateTime::from(time::OffsetDateTime::now_utc() + time::Duration::hours(1));

    // Store only the hash of the token, never the token itself, so a database
    // leak cannot yield live reset links.
    let stored_token_hash = header::hash_token_hex(reset_token.as_bytes());

    let inserted = state
        .connection_pool
        .transaction({
            let stored_token_hash = stored_token_hash.clone();
            move |conn| {
                diesel::update(
                    user::table.filter(
                        user::id.eq(user_id).and(
                            user::password_reset_token
                                .is_null()
                                .or(user::password_reset_expires.lt(DateTime::now())),
                        ),
                    ),
                )
                .set((user::password_reset_token.eq(&stored_token_hash), user::password_reset_expires.eq(expires)))
                .execute(conn)
            }
        })
        .await?;
    if inserted == 0 {
        // A still-valid pending token already exists; do not overwrite it and
        // do not send another email.
        return Ok(());
    }

    let site_name = &state.config.public_info.name;
    let username_encoded = percent_encoding::utf8_percent_encode(&username, NON_ALPHANUMERIC);
    let separator = percent_encoding::percent_encode_byte(b':');
    let url = format!("{domain}/password-reset/{username_encoded}{separator}{reset_token}");

    let email = match Message::builder()
        .from(smtp_info.from.clone())
        .to(user_mailbox.clone())
        .subject(format!("Password reset for {site_name}"))
        .header(ContentType::TEXT_HTML)
        .body(format!(
            "<html>
                <body>
                    <p>Hello,</p>
                    <p>You (or someone else) requested to reset your password on {site_name}.<br>
                    If you wish to proceed, click this link: <a href=\"{url}\">{url}</a></p>
                    <p>Otherwise, please ignore this email.</p>
                </body>
            </html>"
        )) {
        Ok(email) => email,
        Err(_) => return Ok(()),
    };

    // Send the email in a blocking task; a relay or send failure must not
    // affect the already-sent response. The token is stored in the database
    // before the send, so a failed delivery can simply be retried.
    let smtp_host = smtp_info.host.clone();
    let smtp_username = smtp_info.username.clone();
    let smtp_password = smtp_info.password.clone();
    let smtp_port = smtp_info.port;
    let _ = tokio::task::spawn_blocking(move || {
        let mut smtp_builder = match SmtpTransport::relay(&smtp_host) {
            Ok(builder) => builder,
            Err(err) => {
                tracing::warn!("Unable to build SMTP transport for password reset email: {err}");
                return;
            }
        };
        if let (Some(username), Some(password)) = (smtp_username.as_ref(), smtp_password.as_ref()) {
            smtp_builder = smtp_builder.credentials(Credentials::new(username.to_string(), password.to_string()));
        }
        if let Some(port) = smtp_port {
            smtp_builder = smtp_builder.port(port);
        }
        let mailer = smtp_builder.build();

        if let Err(err) = mailer.send(&email) {
            tracing::warn!("Unable to send password reset email: {err}");
        }
    });

    Ok(())
}

/// Token from password reset email.
#[derive(Deserialize, ToSchema)]
struct ResetToken {
    token: String,
}

/// Response containing the new temporary password. Sent as plain-text.
#[derive(Serialize, ToSchema)]
struct NewPassword {
    password: String,
}

/// Creates a random sequence of printable ASCII characters of the given `length`.
fn generate_temporary_password(length: u8) -> String {
    const NUM_CHARACTERS: u8 = b'~' - b'!';

    let rng = &mut OsRng;
    (0..length)
        .map(|_| b'!' + u8::try_from(rng.next_u32() % u32::from(NUM_CHARACTERS)).expect("NUM_CHARACTERS is <= u8::MAX"))
        .map(char::from)
        .collect()
}

/// Generates a new password for given user.
///
/// Password is sent as plain-text, so it is recommended to connect through HTTPS.
#[utoipa::path(
    post,
    path = "/password-reset/{identifier}",
    tag = PASSWORD_RESET_TAG,
    params(
        // The confirm endpoint resolves the user by name only (unlike
        // request_reset, which also accepts an email address).
        ("identifier" = String, Path, description = "Username"),
    ),
    request_body = ResetToken,
    responses(
        (status = 200, description = "New password generated", body = NewPassword),
        (status = 400, description = "Token is missing or invalid"),
        (status = 404, description = "User does not exist"),
    ),
)]
async fn reset_password(
    State(state): State<AppState>,
    Path(username): Path<SmallString>,
    Json(confirmation): Json<ResetToken>,
) -> ApiResult<Json<NewPassword>> {
    const TEMPORARY_PASSWORD_LENGTH: u8 = 16;

    state
        .connection_pool
        .transaction(move |conn| {
            // Lock the user row for the transaction: a concurrent confirm with
            // the same token then blocks on this read, re-reads the committed
            // row after the first one commits, sees the cleared token, and
            // fails with UnauthorizedPasswordReset instead of both succeeding
            // (TOCTOU, I-2).
            let (user_id, _name, _email, stored_token, expires) = user::table
                .select((user::id, user::name, user::email, user::password_reset_token, user::password_reset_expires))
                .filter(user::name.eq(&username))
                .for_update()
                .first::<(i64, SmallString, Option<SmallString>, Option<String>, Option<DateTime>)>(conn)
                .optional()?
                .ok_or(ApiError::UnauthorizedPasswordReset)?;

            let stored_token = stored_token.ok_or(ApiError::UnauthorizedPasswordReset)?;
            let expires = expires.ok_or(ApiError::UnauthorizedPasswordReset)?;

            // The stored value is the token's hash; compare in constant time.
            let submitted_hash = header::hash_token_hex(confirmation.token.as_bytes());
            if !bool::from(stored_token.as_bytes().ct_eq(submitted_hash.as_bytes())) {
                return Err(ApiError::UnauthorizedPasswordReset);
            }

            if DateTime::now() > expires {
                return Err(ApiError::UnauthorizedPasswordReset);
            }

            diesel::update(user::table.find(user_id))
                .set((user::password_reset_token.eq(None::<String>), user::password_reset_expires.eq(None::<DateTime>)))
                .execute(conn)?;

            let temporary_password = generate_temporary_password(TEMPORARY_PASSWORD_LENGTH);

            let salt = SaltString::generate(&mut OsRng);
            let hash = password::hash_password(&state.config, &temporary_password, &salt)?;
            diesel::update(user::table.find(user_id))
                .set((user::password_salt.eq(salt.as_str()), user::password_hash.eq(hash)))
                .execute(conn)?;

            // Revoke all of the user's API tokens so that a stolen token does
            // not survive a password reset.
            diesel::delete(user_token::table.filter(user_token::user_id.eq(user_id))).execute(conn)?;

            Ok(Json(NewPassword {
                password: temporary_password,
            }))
        })
        .await
}

#[cfg(test)]
mod test {
    use crate::api::error::ApiResult;
    use crate::api::routes;
    use crate::auth::header;
    use crate::config::{self, SmtpConfig};
    use crate::model::user::NewUserToken;
    use crate::schema::{user, user_token};
    use crate::string::SmallString;
    use crate::test::*;
    use crate::time::DateTime;
    use axum::ServiceExt;
    use axum::extract::Request;
    use axum::http::header::AUTHORIZATION;
    use axum_test::TestServer;
    use diesel::dsl::exists;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use lettre::Address;
    use lettre::message::Mailbox;
    use serde_json::{Value, json};
    use serial_test::{parallel, serial};
    use std::str::FromStr;
    use tower::layer::Layer;
    use tower_http::normalize_path::NormalizePathLayer;
    use uuid::Uuid;

    #[tokio::test]
    #[parallel]
    async fn error() -> ApiResult<()> {
        verify_response("GET /password-reset/no_one", "password_reset/request_for_nonexistent_user").await?;
        verify_response("POST /password-reset/nobody", "password_reset/confirm_for_nonexistent_user").await?;

        verify_response("GET /password-reset/moderator", "password_reset/no_email").await?;
        verify_response("GET /password-reset/restricted_user", "password_reset/invalid_email").await?;
        verify_response("GET /password-reset/regular_user", "password_reset/reset_disabled").await?;
        verify_response("POST /password-reset/regular_user", "password_reset/invalid_token").await?;
        verify_response("POST /password-reset/regular_user", "password_reset/missing_token").await?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn reset_password_revokes_api_tokens() -> ApiResult<()> {
        let app_state = get_state();
        let (router, _) = routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");
        let auth = header::basic_credentials_for("administrator", TEST_PASSWORD);

        // Create a throwaway user to reset the password for
        let body = json!({"name": "reset_user", "password": "test_password", "rank": "regular"});
        let response = server.post("/users").add_header(AUTHORIZATION, &auth).json(&body).await;
        assert_eq!(response.status_code(), 200);
        let name: String = response.json::<Value>()["name"].as_str().unwrap().to_string();

        let mut conn = get_connection()?;
        let user_id: i64 = user::table
            .select(user::id)
            .filter(user::name.eq(&name))
            .first(&mut conn)?;

        // Seed a valid pending reset token (stored as its hash) and an API
        // token for the user
        diesel::update(user::table.find(user_id))
            .set((
                user::password_reset_token.eq(Some(header::hash_token_hex(b"reset_token"))),
                user::password_reset_expires.eq(Some(DateTime::from(
                    time::OffsetDateTime::now_utc() + time::Duration::hours(1),
                ))),
            ))
            .execute(&mut conn)?;
        let api_token = Uuid::new_v4();
        diesel::insert_into(user_token::table)
            .values(NewUserToken {
                id: api_token,
                user_id,
                note: Some("reset test token"),
                enabled: true,
                expiration_time: None,
            })
            .execute(&mut conn)?;

        // Confirm the reset through the API
        let response = server
            .post(&format!("/password-reset/{name}"))
            .json(&json!({"token": "reset_token"}))
            .await;
        assert_eq!(response.status_code(), 200);
        assert!(response.json::<Value>()["password"].is_string());

        // The API token must have been revoked (M-03 regression test)
        let token_exists: bool = diesel::select(exists(user_token::table.find(api_token))).first(&mut conn)?;
        assert!(!token_exists, "API tokens must be revoked on password reset");

        // Clean up the throwaway user
        let dt: DateTime = user::table
            .select(user::last_edit_time)
            .filter(user::name.eq(&name))
            .first(&mut conn)?;
        let response = server
            .delete(&format!("/user/{name}"))
            .add_header(AUTHORIZATION, &auth)
            .json(&json!({"version": dt}))
            .await;
        assert_eq!(response.status_code(), 200);
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn request_reset_writes_token_in_background() -> ApiResult<()> {
        let mut app_state = get_state();
        // Point SMTP at a port that fails immediately: the test only observes
        // the database write, never the email delivery.
        app_state.config = std::sync::Arc::new({
            let mut config = config::test_config(Some("password_reset/request_for_nonexistent_user"));
            config.smtp = Some(SmtpConfig {
                host: SmallString::new("127.0.0.1"),
                port: Some(1),
                username: None,
                password: None,
                from: Mailbox::new(None, Address::from_str("noreply@example.com").expect("valid test address")),
            });
            config
        });
        let (router, _) = routes(app_state).split_for_parts();
        let app = NormalizePathLayer::trim_trailing_slash().layer(router);
        let server = TestServer::new(ServiceExt::<Request>::into_make_service(app))
            .expect("Test server must be constructible");

        let response = server.get("/password-reset/regular_user").await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.json::<Value>(), Value::Null);

        // The token is written asynchronously, so poll briefly for it
        // (L-08 regression test).
        let mut conn = get_connection()?;
        let mut written = false;
        for _ in 0..200 {
            let pending: Option<String> = user::table
                .select(user::password_reset_token)
                .filter(user::name.eq("regular_user"))
                .first(&mut conn)?;
            if pending.is_some() {
                written = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        assert!(written, "reset token was not written by the background task");
        Ok(())
    }
}
