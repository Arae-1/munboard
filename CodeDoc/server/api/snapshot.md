# snapshot.rs — Snapshot API Endpoints

## Purpose

This file provides the HTTP API handler for listing resource snapshots. Snapshots record every creation, modification, deletion, and merge of resources.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::SNAPSHOT_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::api::{PageParams, PagedResponse, ResourceParams}` | Paginated response types |
| `crate::app::AppState` | Application state |
| `crate::config::Action` | Privilege action |
| `crate::extract::{Ctx, Json, Query}` | Axum extractors |
| `crate::resource::snapshot::{Field, SnapshotInfo}` | Resource-layer types |
| `crate::search::Builder` | Search query builder trait |
| `crate::search::snapshot::QueryBuilder` | Snapshot-specific query builder |
| `utoipa_axum` | OpenAPI router |

## Constants

### `MAX_SNAPSHOTS_PER_PAGE`

Set to `1000`. The maximum number of snapshots returned in a single page.

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with only the `list` route.

---

## Endpoint: List Snapshots

- **Method:** GET
- **Path:** `/snapshots`
- **Authentication:** Required
- **Privilege:** `Action::SnapshotList`

### Handler: `list`

```
async fn list(Ctx(ctx, connection_pool): Ctx, Query(resource): Query<ResourceParams<Field>>, Query(page): Query<PageParams>) -> ApiResult<Json<PagedResponse<SnapshotInfo>>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `fields` | query string | Query | Comma-separated field names for field selection |
| `query` | query string | Query | Search query string for filtering snapshots |
| `offset` | `Option<i64>` | Query | Pagination offset (default: 0) |
| `limit` | page params | Query | Pagination limit (capped at 1000) |

### Search Query Tokens (Named Tokens)

| Key | Description |
|-----|-------------|
| `type` | Filter by resource type |
| `id` | Filter by resource ID |
| `date`, `time` | Filter by creation date |
| `operation` | Filter by operation: `modified`, `created`, `deleted`, or `merged` |
| `user` | Filter by user name (accepts wildcards) |

### Sort Style Tokens

None. Snapshots are sorted by creation time only. Sort style tokens are not supported.

### Special Tokens

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `PagedResponse<SnapshotInfo>` | Paginated list of snapshots |
| 403 | `ErrorResponse` | Insufficient privileges |

### Response Schema: `PagedResponse<SnapshotInfo>`

The response includes:
- `query`: The search query string that was used
- `offset`: The offset applied
- `limit`: The limit applied
- `total`: Total matching snapshots
- `results`: Array of `SnapshotInfo` objects

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::SnapshotList` privilege.
2. Compute `offset`: from `page.offset`, defaulting to `0`.
3. Compute `limit`: the minimum of `page.limit.get()` and `MAX_SNAPSHOTS_PER_PAGE` (1000).
4. Open a database transaction.
5. Inside the transaction:
   a. Create a new `QueryBuilder` from the request context and the resource criteria (extracted from the query string via `resource.criteria()`).
   b. Set the offset and limit on the query builder.
   c. Call `query_builder.list(conn)` to execute the search. This returns a tuple of `(total, selected_snapshots)` where `selected_snapshots` is a `Vec` of snapshot IDs.
   d. Convert the snapshot IDs to `SnapshotInfo` objects using `SnapshotInfo::new_batch_from_ids`, passing the config and field selection.
   e. Build the `PagedResponse` with `query`, `offset`, `limit`, `total`, and `results`.
6. Return the response as JSON.

## Anonymous Tokens

Anonymous user tokens are not supported for snapshots.
