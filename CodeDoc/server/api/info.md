# info.rs — Server Information API Endpoint

## Purpose

This file provides a single HTTP API handler that returns server statistics and configuration. No authentication is required.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::ResourceParams` | Field selection parameters |
| `crate::api::doc::INFO_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::app::AppState` | Application state |
| `crate::config::PublicConfig` | Public server configuration |
| `crate::extract::{Ctx, Json, Query}` | Axum extractors |
| `crate::model::post::PostFeature` | Database model for featured posts |
| `crate::resource::post::{Field, PostInfo}` | Post resource types |
| `crate::schema::{database_statistics, post_feature, user}` | Diesel schemas |
| `crate::string::SmallString` | Compact string type |
| `crate::time::DateTime` | Date-time type |
| `diesel` | Database query builder |
| `serde::Serialize` | Serialization |
| `utoipa::ToSchema` | OpenAPI schema |
| `utoipa_axum` | OpenAPI router |

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with only the `get` route.

## Response Type

### `InfoResponse`

| Field | Type | Description |
|-------|------|-------------|
| `postCount` | `i64` | Total number of posts on the server |
| `diskUsage` | `i64` | Total disk usage in bytes |
| `featuredPost` | `Option<PostInfo>` | The currently featured post, or `null` |
| `featuringTime` | `Option<DateTime>` | Time when the post was featured |
| `featuringUser` | `Option<SmallString>` | Username of the user who featured the post |
| `serverTime` | `DateTime` | Current server time |
| `config` | `PublicConfig` | Public server configuration settings |

---

## Endpoint: Get Server Information

- **Method:** GET
- **Path:** `/info`
- **Authentication:** Not required (no privilege check)

### Handler: `get`

```
async fn get(Ctx(ctx, connection_pool): Ctx, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<InfoResponse>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `fields` | query string | Query | Comma-separated field names for the `featuredPost` field selection |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `InfoResponse` | Server information |

### Handler Logic Step by Step

1. Open a database transaction. No privilege check is performed.
2. Inside the transaction:
   a. Query the `database_statistics` table for `post_count` and `disk_usage`. These are the first row.
   b. Query the `post_feature` table, ordered by `time` descending, to get the most recent feature. Use `optional()` — it may be `None`.
   c. If a featured post exists (`latest_feature` is `Some`):
      - Construct a `PostInfo` from the featured post's `post_id` using `PostInfo::new_from_id`.
      - Query the `user` table by `user_id` from the feature to get the featuring user's name. Handle the case where the user may have been deleted (use `optional()` and `flatten()`).
   d. If no featured post exists, `featuredPost`, `featuringTime`, and `featuringUser` are all `None`.
   e. Build the `InfoResponse`:
      - `post_count` and `disk_usage` from the statistics table
      - `featured_post`: the `PostInfo` or `None`
      - `featuring_time`: the feature's time or `None`
      - `featuring_user`: the fetched username or `None`
      - `server_time`: `DateTime::now()`
      - `config`: `ctx.config.public_info.clone()` (the public configuration)
3. Return the response as JSON.
