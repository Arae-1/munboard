use crate::api::doc::PASSWORD_RESET_TAG;
use crate::api::error::{ApiError, ApiResult};
use crate::app::AppState;
use crate::auth::password;
use crate::content::hash;
use crate::extract::{Json, Path};
use crate::schema::user;
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
/// user exists or has an email configured, to prevent user enumeration.
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
    let smtp_info = state.config.smtp().ok_or(ApiError::MissingSmtpInfo)?;

    let mut conn = state.connection_pool.get().await?;

    let user_info = user::table
        .select((user::id, user::name, user::email, user::password_reset_token, user::password_reset_expires))
        .filter(user::name.eq(&identifier).or(user::email.eq(&identifier)))
        .first::<(i64, SmallString, Option<SmallString>, Option<String>, Option<DateTime>)>(conn.as_mut())
        .optional()?;

    let (user_id, username, user_email, existing_token, existing_expiry) = match user_info {
        Some(info) => info,
        None => return Ok(Json(())),
    };

    let user_email = match user_email {
        Some(email) => email,
        None => return Ok(Json(())),
    };

    if let (Some(_), Some(expires)) = (&existing_token, &existing_expiry) {
        if *expires > DateTime::now() {
            return Err(ApiError::ResetAlreadyRequested);
        }
    }

    let user_mailbox = match Address::from_str(&user_email) {
        Ok(address) => Mailbox::new(None, address),
        Err(_) => return Ok(Json(())),
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

    let site_name = &state.config.public_info.name;
    let username_encoded = percent_encoding::utf8_percent_encode(&username, NON_ALPHANUMERIC);
    let separator = percent_encoding::percent_encode_byte(b':');

    let mut random_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut random_bytes);
    let reset_token = hash::compute_url_safe_hash(&random_bytes);
    let expires = DateTime::from(time::OffsetDateTime::now_utc() + time::Duration::hours(1));

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
        Err(_) => return Ok(Json(())),
    };

    diesel::update(user::table.find(user_id))
        .set((user::password_reset_token.eq(&reset_token), user::password_reset_expires.eq(expires)))
        .execute(conn.as_mut())?;

    let mut smtp_builder = SmtpTransport::relay(&smtp_info.host)?;
    if let (Some(smtp_username), Some(smtp_password)) = (smtp_info.username.as_ref(), smtp_info.password.as_ref()) {
        let credentials = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
        smtp_builder = smtp_builder.credentials(credentials);
    }
    if let Some(port) = smtp_info.port {
        smtp_builder = smtp_builder.port(port);
    }
    let mailer = smtp_builder.build();

    let _ = mailer.send(&email);

    Ok(Json(()))
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
        ("identifier" = String, Path, description = "User email or username"),
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
            let (user_id, _name, _email, stored_token, expires) = user::table
                .select((user::id, user::name, user::email, user::password_reset_token, user::password_reset_expires))
                .filter(user::name.eq(&username))
                .first::<(i64, SmallString, Option<SmallString>, Option<String>, Option<DateTime>)>(conn)
                .optional()?
                .ok_or(ApiError::UnauthorizedPasswordReset)?;

            let stored_token = stored_token.ok_or(ApiError::UnauthorizedPasswordReset)?;
            let expires = expires.ok_or(ApiError::UnauthorizedPasswordReset)?;

            if !bool::from(stored_token.as_bytes().ct_eq(confirmation.token.as_bytes())) {
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

            Ok(Json(NewPassword {
                password: temporary_password,
            }))
        })
        .await
}

#[cfg(test)]
mod test {
    use crate::api::error::ApiResult;
    use crate::test::*;
    use serial_test::parallel;

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
}
