# pool_category.rs — Pool Category API Endpoints

## Purpose

This file provides HTTP API handlers for pool category resources. Pool categories organize pools into groups. Each pool belongs to exactly one category.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::POOL_CATEGORY_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::api::{DeleteBody, ResourceParams, UnpagedResponse, error}` | Shared request/response types |
| `crate::app::AppState` | Application state |
| `crate::config::{Action, RegexType}` | Privilege actions and regex validation |
| `crate::extract::{Ctx, Json, Path, Query}` | Axum extractors |
| `crate::model::enums::{ResourceProperty, ResourceType}` | Resource enum types |
| `crate::model::pool_category::{NewPoolCategory, PoolCategory}` | Database model types |
| `crate::resource::pool_category::{Field, PoolCategoryInfo}` | Resource-layer types |
| `crate::schema::{pool, pool_category}` | Diesel schemas |
| `crate::string::SmallString` | Compact string type |
| `crate::time::DateTime` | Date-time type |
| `crate::{api, snapshot}` | API helpers and snapshot functions |
| `diesel` | Database query builder |
| `serde::Deserialize` | Deserialization |
| `utoipa::ToSchema` | OpenAPI schema |
| `utoipa_axum` | OpenAPI router |

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with these route groups:
- `list`, `create` — collection-level routes
- `get`, `update`, `delete` — single-resource routes by name
- `set_default` — special action route

---

## Endpoint: List Pool Categories

- **Method:** GET
- **Path:** `/pool-categories`
- **Authentication:** Required
- **Privilege:** `Action::PoolCategoryList`

### Handler: `list`

```
async fn list(Ctx(ctx, connection_pool): Ctx, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<UnpagedResponse<PoolCategoryInfo>>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `fields` | query string | Query | Comma-separated field names |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UnpagedResponse<PoolCategoryInfo>` | List of all pool categories |
| 403 | `ErrorResponse` | Insufficient privileges |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::PoolCategoryList` privilege.
2. Open a database transaction.
3. Inside the transaction, call `PoolCategoryInfo::all(conn, params.fields)` to fetch and convert all pool categories.
4. Wrap results in `UnpagedResponse` and return as JSON.

---

## Endpoint: Get Pool Category

- **Method:** GET
- **Path:** `/pool-category/{name}`
- **Authentication:** Required
- **Privilege:** `Action::PoolCategoryView`

### Handler: `get`

```
async fn get(Ctx(ctx, connection_pool): Ctx, Path(name): Path<SmallString>, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<PoolCategoryInfo>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `name` | `String` | Path | Pool category name |
| `fields` | query string | Query | Comma-separated field names |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `PoolCategoryInfo` | The found pool category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Pool category does not exist |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::PoolCategoryView` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Query `pool_category::table` filtered by `name` matching the path parameter.
   b. If no row is found, return `ApiError::NotFound(ResourceType::PoolCategory)`.
   c. Convert the `PoolCategory` to `PoolCategoryInfo` using `PoolCategoryInfo::new`.
4. Return the response as JSON.

---

## Endpoint: Create Pool Category

- **Method:** POST
- **Path:** `/pool-categories`
- **Authentication:** Required
- **Privilege:** `Action::PoolCategoryCreate`

### Handler: `create`

```
async fn create(Ctx(ctx, connection_pool): Ctx, Query(params): Query<ResourceParams<Field>>, Json(body): Json<PoolCategoryCreateBody>) -> ApiResult<Json<PoolCategoryInfo>>
```

### Request Body: `PoolCategoryCreateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `SmallString` | Yes | Category name. Must match `pool_category_name_regex`. |
| `color` | `SmallString` | Yes | Category color. |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `PoolCategoryInfo` | The created pool category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 409 | `ErrorResponse` | Name is used by an existing pool category |
| 422 | `ErrorResponse` | Name is invalid or missing |
| 422 | `ErrorResponse` | Color is invalid or missing |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::PoolCategoryCreate` privilege.
2. Verify the `name` matches `RegexType::PoolCategory` from the server configuration.
3. Open a database transaction.
4. Inside the transaction:
   a. Insert a new `NewPoolCategory` with the provided `name` and `color`.
   b. Use `ON CONFLICT (name) DO NOTHING`. Pool category names are case insensitive.
   c. If the row already exists and `do_nothing()` returns no result, return `ApiError::AlreadyExists(ResourceProperty::PoolCategoryName)`.
   d. Create a creation snapshot via `snapshot::pool_category::creation_snapshot`.
   e. Return the inserted `PoolCategory`.
5. Open a second database transaction.
6. Convert the new `PoolCategory` to `PoolCategoryInfo` using `PoolCategoryInfo::new`.
7. Return the response as JSON.

---

## Endpoint: Update Pool Category

- **Method:** PUT
- **Path:** `/pool-category/{name}`
- **Authentication:** Required
- **Privileges:** `Action::PoolCategoryEditName` (if name changes), `Action::PoolCategoryEditColor` (if color changes)

### Handler: `update`

```
async fn update(Ctx(ctx, connection_pool): Ctx, Path(name): Path<SmallString>, Query(params): Query<ResourceParams<Field>>, Json(body): Json<PoolCategoryUpdateBody>) -> ApiResult<Json<PoolCategoryInfo>>
```

### Request Body: `PoolCategoryUpdateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | `DateTime` | Yes | Resource version for optimistic locking |
| `name` | `Option<SmallString>` | No | New category name |
| `color` | `Option<SmallString>` | No | New category color |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `name` | `String` | Path | Current pool category name |
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `PoolCategoryInfo` | The updated pool category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Pool category not found |
| 409 | `ErrorResponse` | Version is outdated |
| 409 | `ErrorResponse` | Name already exists |
| 422 | `ErrorResponse` | Name is invalid |
| 422 | `ErrorResponse` | Color is invalid |

### Handler Logic Step by Step

1. Open a database transaction.
2. Inside the transaction:
   a. Find the existing `PoolCategory` filtered by `name` matching the path parameter.
   b. If not found, return `ApiError::NotFound(ResourceType::PoolCategory)`.
   c. Verify `old_category.last_edit_time` matches `body.version`. If not, return `ApiError::ResourceModified`.
   d. Clone the old category into a mutable `new_category`.
   e. If `body.name` is `Some`:
      - Verify `Action::PoolCategoryEditName` privilege.
      - Verify the name matches `RegexType::PoolCategory`.
      - Set `new_category.name` to the new value.
   f. If `body.color` is `Some`:
      - Verify `Action::PoolCategoryEditColor` privilege.
      - Set `new_category.color` to the new value.
   g. Update `new_category.last_edit_time` to the current time.
   h. Save changes. If a unique violation occurs, map to `ApiError::AlreadyExists(ResourceProperty::PoolCategoryName)`.
   i. Create a modification snapshot via `snapshot::pool_category::modification_snapshot`.
   j. Return the updated `PoolCategory`.
3. Open a second database transaction.
4. Convert the updated `PoolCategory` to `PoolCategoryInfo` using `PoolCategoryInfo::new`.
5. Return the response as JSON.

---

## Endpoint: Set Default Pool Category

- **Method:** PUT
- **Path:** `/pool-category/{name}/default`
- **Authentication:** Required
- **Privilege:** `Action::PoolCategorySetDefault`

### Handler: `set_default`

```
async fn set_default(Ctx(ctx, connection_pool): Ctx, Path(name): Path<SmallString>, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<PoolCategoryInfo>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `name` | `String` | Path | Pool category name to become default |
| `fields` | query string | Query | Comma-separated field names |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `PoolCategoryInfo` | The new default pool category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Pool category does not exist |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::PoolCategorySetDefault` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the target `PoolCategory` by `name` from the path. If not found, return `ApiError::NotFound(ResourceType::PoolCategory)`.
   b. Find the current default `PoolCategory` using `PoolCategory::default()` filter (where `id = 0`).
   c. Update all pools that currently belong to the target category: set their `category_id` to `0` (default). Collect their IDs.
   d. Update all pools that currently have `category_id = 0` but are NOT the pools that were moved: set their `category_id` to the target category's ID. This handles pools already in the default category.
   e. Swap the IDs: `std::mem::swap(&mut category.id, &mut old_default_category.id)`. Now `category` has ID 0 and `old_default_category` has the former target ID.
   f. Temporarily set the new default category name to an empty string to avoid unique constraint violation while saving.
   g. Save the new default category (`category.save_changes`).
   h. Save the old default category (`old_default_category.save_changes`) with its new ID.
   i. Restore the proper name to the new default category and save again.
   j. Create a set-default snapshot via `snapshot::pool_category::set_default_snapshot`.
   k. Return the new default `PoolCategory`.
4. Open a second database transaction.
5. Convert the new default `PoolCategory` to `PoolCategoryInfo` using `PoolCategoryInfo::new`.
6. Return the response as JSON.

---

## Endpoint: Delete Pool Category

- **Method:** DELETE
- **Path:** `/pool-category/{name}`
- **Authentication:** Required
- **Privilege:** `Action::PoolCategoryDelete`

### Handler: `delete`

```
async fn delete(Ctx(ctx, connection_pool): Ctx, Path(name): Path<SmallString>, Json(client_version): Json<DeleteBody>) -> ApiResult<Json<()>>
```

### Request Body: `DeleteBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | `DateTime` | Yes | Resource version |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `name` | `String` | Path | Pool category name |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | Empty JSON `()` | Pool category deleted |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Pool category does not exist |
| 409 | `ErrorResponse` | Version is outdated |
| 422 | `ErrorResponse` | Category is the default category |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::PoolCategoryDelete` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the `PoolCategory` by `name`. If not found, return `ApiError::NotFound(ResourceType::PoolCategory)`.
   b. Verify the version matches `*client_version`. If not, return `ApiError::ResourceModified`.
   c. If `category.id == 0`, this is the default category. Return `ApiError::DeleteDefault(ResourceType::PoolCategory)`.
   d. Execute a `DELETE` query on the table by the category's ID.
   e. Create a deletion snapshot via `snapshot::pool_category::deletion_snapshot`.
   f. Return `Json(())`.
4. Return the response.

---

## Tests

The module contains integration tests using the `verify_response` helper. The test constant `FIELDS` excludes `creation_time` and `last_edit_time` to avoid non-deterministic failures.

| Test Function | Serial | Description |
|---------------|--------|-------------|
| `list` | parallel | Verifies GET `/pool-categories/` response matches `pool_category/list` |
| `get` | parallel | Verifies GET `/pool-category/Setting/` and checks `last_edit_time` is unchanged |
| `create` | serial | Creates a category, verifies count increases by 1, checks usage is 0, then deletes it |
| `update` | serial | Updates "Style" category, verifies name and color changed, `last_edit_time` increased, then restores |
| `set_default` | serial | Makes "Setting" the default, verifies `id == 0`, then restores original default |
| `error` | parallel | Tests nonexistent get, edit, set_default, delete; tests invalid create, name clash, edit invalid, delete default |
