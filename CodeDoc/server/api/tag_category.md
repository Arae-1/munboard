# tag_category.rs — Tag Category API Endpoints

## Purpose

This file provides HTTP API handlers for tag category resources. Tag categories organize tags into groups. The default category has ID `0`.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::TAG_CATEGORY_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::api::{DeleteBody, ResourceParams, UnpagedResponse, error}` | Shared types |
| `crate::app::{AppState, Context}` | Application state and request context |
| `crate::config::{Action, RegexType}` | Privilege actions and regex validation |
| `crate::extract::{Ctx, Json, Path, Query}` | Axum extractors |
| `crate::model::enums::{ResourceProperty, ResourceType, UserRank}` | Enum types |
| `crate::model::tag_category::{NewTagCategory, TagCategory}` | Database model types |
| `crate::resource::tag_category::{Field, TagCategoryInfo}` | Resource-layer types |
| `crate::schema::{tag, tag_category}` | Diesel schemas |
| `crate::string::SmallString` | Compact string |
| `crate::time::DateTime` | Date-time |
| `crate::{api, snapshot}` | API helpers and snapshots |
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

## Helper Function

### `verify_visibility`

```
fn verify_visibility(conn: &mut PgConnection, ctx: &Context, category_name: &SmallString) -> ApiResult<TagCategory>
```

This function handles two concerns:
1. If the client is anonymous (`UserRank::Anonymous`) and the category name is in `ctx.config.anonymous_preferences.tag_category_blacklist`, return `ApiError::Hidden(ResourceType::TagCategory)`.
2. Otherwise, find the `TagCategory` by name. If not found, return `ApiError::NotFound(ResourceType::TagCategory)`.

---

## Endpoint: List Tag Categories

- **Method:** GET
- **Path:** `/tag-categories`
- **Authentication:** Required
- **Privilege:** `Action::TagCategoryList`

### Handler: `list`

```
async fn list(Ctx(ctx, connection_pool): Ctx, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<UnpagedResponse<TagCategoryInfo>>>
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
| 200 | `UnpagedResponse<TagCategoryInfo>` | All tag categories |
| 403 | `ErrorResponse` | Insufficient privileges |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::TagCategoryList` privilege.
2. Open a database transaction.
3. Inside the transaction, call `TagCategoryInfo::all(conn, &ctx, params.fields)`. This internally handles visibility checks per category.
4. Wrap results in `UnpagedResponse` and return as JSON.

---

## Endpoint: Get Tag Category

- **Method:** GET
- **Path:** `/tag-category/{name}`
- **Authentication:** Required
- **Privilege:** `Action::TagCategoryView`

### Handler: `get`

```
async fn get(Ctx(ctx, connection_pool): Ctx, Path(name): Path<SmallString>, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<TagCategoryInfo>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `name` | `String` | Path | Tag category name |
| `fields` | query string | Query | Comma-separated field names |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `TagCategoryInfo` | The tag category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 403 | `ErrorResponse` | Tag category is hidden (anonymous user in blacklist) |
| 404 | `ErrorResponse` | Tag category not found |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::TagCategoryView` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Call `verify_visibility(conn, &ctx, &name)` to check blacklist and find the category.
   b. Convert the `TagCategory` to `TagCategoryInfo` using `TagCategoryInfo::new`.
4. Return the response as JSON.

---

## Endpoint: Create Tag Category

- **Method:** POST
- **Path:** `/tag-categories`
- **Authentication:** Required
- **Privilege:** `Action::TagCategoryCreate`

### Handler: `create`

```
async fn create(Ctx(ctx, connection_pool): Ctx, Query(params): Query<ResourceParams<Field>>, Json(body): Json<TagCategoryCreateBody>) -> ApiResult<Json<TagCategoryInfo>>
```

### Request Body: `TagCategoryCreateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `order` | `i32` | Yes | Display order for the category |
| `name` | `SmallString` | Yes | Category name. Must match `tag_category_name_regex`. |
| `color` | `SmallString` | Yes | Category color |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `TagCategoryInfo` | The created tag category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 409 | `ErrorResponse` | Name already exists |
| 422 | `ErrorResponse` | Name is invalid or missing |
| 422 | `ErrorResponse` | Color is invalid or missing |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::TagCategoryCreate` privilege.
2. Verify the `name` matches `RegexType::TagCategory` from the server configuration.
3. Open a database transaction.
4. Inside the transaction:
   a. Insert a new `NewTagCategory` with the provided `order`, `name`, and `color`.
   b. Use `ON CONFLICT (name) DO NOTHING`.
   c. If no row is inserted (name conflict), return `ApiError::AlreadyExists(ResourceProperty::TagCategoryName)`.
   d. Create a creation snapshot via `snapshot::tag_category::creation_snapshot`.
   e. Return the inserted `TagCategory`.
5. Open a second database transaction.
6. Convert the new `TagCategory` to `TagCategoryInfo` using `TagCategoryInfo::new`.
7. Return the response as JSON.

---

## Endpoint: Update Tag Category

- **Method:** PUT
- **Path:** `/tag-category/{name}`
- **Authentication:** Required
- **Privileges:** `Action::TagCategoryEditOrder` (if order changes), `Action::TagCategoryEditName` (if name changes), `Action::TagCategoryEditColor` (if color changes)

### Handler: `update`

```
async fn update(Ctx(ctx, connection_pool): Ctx, Path(name): Path<SmallString>, Query(params): Query<ResourceParams<Field>>, Json(body): Json<TagCategoryUpdateBody>) -> ApiResult<Json<TagCategoryInfo>>
```

### Request Body: `TagCategoryUpdateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | `DateTime` | Yes | Resource version |
| `order` | `Option<i32>` | No | New display order |
| `name` | `Option<SmallString>` | No | New category name |
| `color` | `Option<SmallString>` | No | New category color |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `name` | `String` | Path | Current tag category name |
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `TagCategoryInfo` | The updated tag category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Tag category not found |
| 409 | `ErrorResponse` | Version is outdated |
| 409 | `ErrorResponse` | Name already exists |
| 422 | `ErrorResponse` | Name is invalid |
| 422 | `ErrorResponse` | Color is invalid |

### Handler Logic Step by Step

1. Open a database transaction.
2. Inside the transaction:
   a. Find the existing `TagCategory` filtered by `name`. If not found, return `ApiError::NotFound(ResourceType::TagCategory)`.
   b. Verify `old_category.last_edit_time` matches `body.version`. If not, return `ApiError::ResourceModified`.
   c. Clone the old category into a mutable `new_category`.
   d. If `body.order` is `Some`:
      - Verify `Action::TagCategoryEditOrder` privilege.
      - Set `new_category.order`.
   e. If `body.name` is `Some`:
      - Verify `Action::TagCategoryEditName` privilege.
      - Verify name matches `RegexType::TagCategory`.
      - Set `new_category.name`.
   f. If `body.color` is `Some`:
      - Verify `Action::TagCategoryEditColor` privilege.
      - Set `new_category.color`.
   g. Update `new_category.last_edit_time` to the current time.
   h. Save changes. Map unique violation to `ApiError::AlreadyExists(ResourceProperty::TagCategoryName)`.
   i. Create a modification snapshot via `snapshot::tag_category::modification_snapshot`.
   j. Return the updated `TagCategory`.
3. Open a second database transaction.
4. Convert the updated `TagCategory` to `TagCategoryInfo` using `TagCategoryInfo::new`.
5. Return the response as JSON.

---

## Endpoint: Set Default Tag Category

- **Method:** PUT
- **Path:** `/tag-category/{name}/default`
- **Authentication:** Required
- **Privilege:** `Action::TagCategorySetDefault`

### Handler: `set_default`

```
async fn set_default(Ctx(ctx, connection_pool): Ctx, Path(name): Path<SmallString>, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<TagCategoryInfo>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `name` | `String` | Path | Tag category name to become default |
| `fields` | query string | Query | Comma-separated field names |

### Request Body

None. The OpenAPI doc declares `request_body = Object`, but the handler takes no body.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `TagCategoryInfo` | The new default tag category |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Tag category not found |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::TagCategorySetDefault` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the target `TagCategory` by `name`. If not found, return `ApiError::NotFound(ResourceType::TagCategory)`.
   b. Find the current default `TagCategory` using `TagCategory::default()` filter (`id == 0`).
   c. Update all tags that belong to the target category: set their `category_id` to `0` (default). Collect their IDs.
   d. Update all tags with `category_id = 0` that are NOT the ones that were moved: set their `category_id` to the target category's ID.
   e. Swap the IDs of the two categories via `std::mem::swap`.
   f. Temporarily set the new default category name to an empty string to avoid unique constraint violation.
   g. Save the new default category.
   h. Save the old default category.
   i. Restore the proper name to the new default category and save again.
   j. Create a set-default snapshot via `snapshot::tag_category::set_default_snapshot`.
   k. Return the new default `TagCategory`.
4. Open a second database transaction.
5. Convert the new default `TagCategory` to `TagCategoryInfo` using `TagCategoryInfo::new`.
6. Return the response as JSON.

---

## Endpoint: Delete Tag Category

- **Method:** DELETE
- **Path:** `/tag-category/{name}`
- **Authentication:** Required
- **Privilege:** `Action::TagCategoryDelete`

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
| `name` | `String` | Path | Tag category name |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | Empty JSON `()` | Tag category deleted |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Tag category not found |
| 409 | `ErrorResponse` | Version is outdated |
| 422 | `ErrorResponse` | Category is the default category |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::TagCategoryDelete` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the `TagCategory` by `name`. If not found, return `ApiError::NotFound(ResourceType::TagCategory)`.
   b. Verify the version matches `*client_version`.
   c. If `category.id == 0`, return `ApiError::DeleteDefault(ResourceType::TagCategory)`.
   d. Execute a `DELETE` query on the table by the category's ID.
   e. Create a deletion snapshot via `snapshot::tag_category::deletion_snapshot`.
   f. Return `Json(())`.
4. Return the response.

---

## Tests

The test constant `FIELDS` excludes `creation_time` and `last_edit_time`.

| Test Function | Serial | Description |
|---------------|--------|-------------|
| `list` | parallel | Verifies GET `/tag-categories/` response |
| `get` | parallel | Verifies GET `/tag-category/Source/` and checks `last_edit_time` is unchanged |
| `create` | serial | Creates a category, verifies count increase, checks usage is 0, then deletes |
| `update` | serial | Updates "Character" category, verifies changed fields, then restores |
| `set_default` | serial | Makes "Surroundings" the default, checks `id == 0`, then restores |
| `preferences` | parallel | Tests anonymous user with tag category blacklist preferences (list and get with `meta` category) |
| `error` | parallel | Tests nonexistent and invalid operations, name clashes, delete default |
