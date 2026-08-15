use crate::api::doc::ApiDoc;
use crate::api::error::{ApiError, ApiResult};
use crate::app::AppState;
use crate::auth::Client;
use crate::config::{Config, RegexType};
use crate::model::enums::{Rating, UserRank};
use crate::resource::field::Mask;
use crate::string::SmallString;
use crate::time::DateTime;
use axum::http::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use std::num::NonZeroI64;
use std::ops::Deref;
use std::sync::LazyLock;
use std::str::FromStr;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;

mod comment;
mod custom_field;
mod doc;
pub mod error;
mod info;
mod legacy;
pub mod middleware;
mod password_reset;
mod pool;
mod pool_category;
mod post;
mod snapshot;
mod tag;
mod tag_category;
mod upload;
mod user;
mod user_token;

pub fn routes(state: AppState) -> OpenApiRouter {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(comment::routes())
        .merge(custom_field::routes())
        .merge(info::routes())
        .merge(legacy::routes())
        .merge(password_reset::routes())
        .merge(pool::routes())
        .merge(pool_category::routes())
        .merge(post::routes(&state.config))
        .merge(snapshot::routes())
        .merge(tag::routes())
        .merge(tag_category::routes())
        .merge(upload::routes(&state.config))
        .merge(user::routes(&state.config))
        .merge(user_token::routes())
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_mins(1)),
        ))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth))
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::post_to_webhooks))
        .route_layer(axum::middleware::from_fn(middleware::log_error))
        .with_state(state)
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route not found") })
}

/// Checks if `haystack` matches regex `regex_type`.
/// Returns error if it does not match on the regex.
pub fn verify_matches_regex(config: &Config, haystack: &str, regex_type: RegexType) -> ApiResult<()> {
    config
        .regex(regex_type)
        .is_match(haystack)
        .then_some(())
        .ok_or_else(|| ApiError::ExpressionFailsRegex(SmallString::new(haystack), regex_type))
}

/// Checks if `email` is a valid email.
/// Returns error if `email` is invalid.
pub fn verify_valid_email(email: Option<&str>) -> Result<(), lettre::address::AddressError> {
    match email {
        Some(address) => address.parse::<lettre::Address>().map(|_| ()),
        None => Ok(()),
    }
}

/// Request body to apply/change a score.
#[derive(Deserialize, ToSchema)]
struct RatingBody {
    score: Rating,
}

impl Deref for RatingBody {
    type Target = Rating;
    fn deref(&self) -> &Self::Target {
        &self.score
    }
}

/// Request body for deleting a resource.
#[derive(Deserialize, ToSchema)]
struct DeleteBody {
    /// Resource version. See [versioning](#Versioning).
    version: DateTime,
}

impl Deref for DeleteBody {
    type Target = DateTime;
    fn deref(&self) -> &Self::Target {
        &self.version
    }
}

/// Request body for merging resources.
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct MergeBody<T> {
    /// ID of the source resource to be removed.
    remove: T,
    /// ID of the target resource to merge into.
    merge_to: T,
    /// Version of the source resource. See [versioning](#Versioning).
    remove_version: DateTime,
    /// Version of the target resource. See [versioning](#Versioning).
    merge_to_version: DateTime,
}

/// Represents parameters of a request to retrieve one or more resources.
#[derive(Deserialize, IntoParams)]
#[serde(bound(deserialize = "F: Into<u64> + FromStr"))]
struct ResourceParams<F: Into<u64> + FromStr> {
    /// Query search string
    #[param(example = "anonymous_token")]
    query: Option<String>,
    /// Comma-separated list of fields to include in the response. See [field selection](#Field-Selection) for details.
    #[param(value_type = Option<String>, example = "field1,field2")]
    fields: Mask<F>,
}

impl<F: Into<u64> + FromStr> ResourceParams<F> {
    fn criteria(&self) -> &str {
        self.query.as_deref().unwrap_or("")
    }
}

/// Represents parameters of a request to retrieve multiple resources, paged.
#[derive(Deserialize, IntoParams)]
struct PageParams {
    /// Starting position in the result set
    #[param(example = 0)]
    #[serde(default, deserialize_with = "deserialize_non_negative_i64")]
    offset: Option<i64>,
    /// Maximum number of results to return
    #[param(value_type = i64, minimum = 1, example = 40)]
    #[serde(deserialize_with = "deserialize_positive_i64")]
    limit: NonZeroI64,
}

/// Deserializes `limit` as a positive integer. `NonZeroI64` alone would accept
/// negative values, which fail later with an internal database error.
fn deserialize_positive_i64<'de, D>(deserializer: D) -> Result<NonZeroI64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;
    NonZeroI64::new(value)
        .filter(|&value| value.get() > 0)
        .ok_or_else(|| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Signed(value), &"a nonzero i64")
        })
}

/// Upper bound on the `offset` pagination parameter. A request cannot force a
/// full table scan through an arbitrarily large SQL OFFSET, so values above
/// this are silently clamped (mirroring how `limit` is clamped per endpoint).
const MAX_OFFSET: i64 = 1_000_000;

/// Deserializes `offset` as a non-negative integer. Absent values remain `None`.
fn deserialize_non_negative_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;
    if value < 0 {
        return Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Signed(value),
            &"a non-negative i64",
        ));
    }
    Ok(Some(value.min(MAX_OFFSET)))
}

/// A result of search operation that doesn't involve paging.
#[derive(Serialize, ToSchema)]
struct UnpagedResponse<T> {
    results: Vec<T>,
}

/// A result of search operation that involves paging.
#[derive(Serialize, ToSchema)]
struct PagedResponse<T> {
    /// The query passed in the original request that contains a standard [search query](#Search).
    #[schema(examples("anonymous_token named_token:value1,value2,value3 sort:sort_token"))]
    query: Option<String>,
    /// The record starting offset, passed in the original request.
    #[schema(examples(0))]
    offset: i64,
    /// Number of records on one page.
    #[schema(examples(40))]
    limit: i64,
    /// How many resources were found. To get the page count, divide this number by `limit`.
    #[schema(examples(1729))]
    total: i64,
    results: Vec<T>,
}

/// Checks if the `client` is at least `required_rank`.
/// Returns error if client is lower rank than `required_rank`.
fn verify_privilege(client: Client, required_rank: UserRank) -> ApiResult<()> {
    (client.rank >= required_rank)
        .then_some(())
        .ok_or(ApiError::InsufficientPrivileges)
}

/// Environment variable that disables the concurrent-edit (version) check.
///
/// The check is enabled by default. Setting this variable to any value (e.g.
/// `OXIBOORU_DISABLE_VERSION_CHECK=1`) disables it at runtime. This is intended for
/// automated tests and one-off maintenance scripts that deliberately operate on stale
/// versions.
pub(crate) const DISABLE_VERSION_CHECK_ENV: &str = "OXIBOORU_DISABLE_VERSION_CHECK";

/// Checks if `current_version` matches `client_version`.
/// Returns error if they do not match.
fn verify_version(current_version: DateTime, client_version: DateTime) -> ApiResult<()> {
    // Disabled at runtime when `OXIBOORU_DISABLE_VERSION_CHECK` is set
    if std::env::var_os(DISABLE_VERSION_CHECK_ENV).is_some() {
        // Warn once so a production misconfiguration is visible in logs. The
        // test harness sets this variable deliberately, so skip test builds.
        static WARNED: LazyLock<()> = LazyLock::new(|| {
            tracing::warn!("{DISABLE_VERSION_CHECK_ENV} is set; the optimistic concurrency check is disabled");
        });
        if cfg!(not(test)) {
            LazyLock::force(&WARNED);
        }
        return Ok(());
    }

    (current_version == client_version)
        .then_some(())
        .ok_or(ApiError::ResourceModified)
}

// Any value that is present is considered Some value, including null.
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
