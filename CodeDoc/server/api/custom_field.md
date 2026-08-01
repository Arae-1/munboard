# custom_field.rs — Custom Field API Endpoints

## Purpose

This file provides HTTP API handlers for custom field resources. Custom fields are metadata fields that users can attach to posts.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::CUSTOM_FIELD_TAG` | OpenAPI documentation tag for grouping |
| `crate::api::error::{ApiError, ApiResult}` | Error types for API responses |
| `crate::api::{DeleteBody, ResourceParams, UnpagedResponse, error}` | Shared request/response types |
| `crate::app::AppState` | Application shared state |
| `crate::config::{Action, RegexType}` | Privilege action types and regex validation |
| `crate::extract::{Ctx, Json, Path, Query}` | Axum request extractors |
| `crate::model::custom_field::{CustomField, NewCustomField}` | Database model types |
| `crate::model::enums::{ResourceProperty, ResourceType}` | Enum types for resources |
| `crate::resource::custom_field::{CustomFieldInfo, Field}` | Resource-layer info type and field selection |
| `crate::schema::custom_field` | Diesel schema for the custom_field table |
| `crate::string::SmallString` | Custom efficient string type |
| `crate::time::DateTime` | Date-time wrapper type |
| `crate::{api, snapshot}` | API helpers and snapshot creation |
| `diesel` | Database query builder types |
| `regex::Regex` | Regular expression matching |
| `serde::Deserialize` | Request body deserialization |
| `std::sync::LazyLock` | Lazy static initialization |
| `utoipa::ToSchema` | OpenAPI schema generation |
| `utoipa_axum` | OpenAPI-aware Axum router |

## Static Data

### `COLOR_HEX_REGEX`

A `LazyLock<Regex>` that matches six-digit hexadecimal color codes. The pattern is `^#[0-9a-fA-F]{6}$`.

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with these route groups:
- `list`, `create` — root collection routes
- `get`, `update`, `delete` — single-resource routes by ID

---

## Endpoint: List Custom Fields

- **Method:** GET
- **Path:** `/custom-fields`
- **Authentication:** Required
- **Privilege:** `Action::CustomFieldList`

### Handler: `list`

```
async fn list(Ctx(ctx, connection_pool): Ctx, Query(resource): Query<ResourceParams<Field>>) -> ApiResult<Json<UnpagedResponse<CustomFieldInfo>>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `fields` | query string | Query | Comma-separated field names for field selection |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UnpagedResponse<CustomFieldInfo>` | List of all custom fields |
| 403 | `ErrorResponse` | Insufficient privileges |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::CustomFieldList` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Query all rows from the `custom_field` table, ordered by `name`.
   b. Convert each `CustomField` to a `CustomFieldInfo` using `CustomFieldInfo::new_batch`, applying the `fields` filter from the query parameters.
   c. Wrap the results in an `UnpagedResponse`.
4. Return the response as JSON.

---

## Endpoint: Get Custom Field

- **Method:** GET
- **Path:** `/custom-field/{id}`
- **Authentication:** Required
- **Privilege:** `Action::CustomFieldView`

### Handler: `get`

```
async fn get(Ctx(ctx, connection_pool): Ctx, Path(custom_field_id): Path<i64>, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<CustomFieldInfo>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `id` | `i64` | Path | Custom field ID |
| `fields` | query string | Query | Comma-separated field names for field selection |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `CustomFieldInfo` | The requested custom field |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Custom field does not exist |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::CustomFieldView` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the `CustomField` by primary key `custom_field_id`.
   b. If no row is found, return `ApiError::NotFound(ResourceType::CustomField)`.
   c. Convert the `CustomField` to `CustomFieldInfo` using `CustomFieldInfo::new`, applying the `fields` filter.
4. Return the response as JSON.

---

## Endpoint: Create Custom Field

- **Method:** POST
- **Path:** `/custom-fields`
- **Authentication:** Required
- **Privilege:** `Action::CustomFieldCreate`

### Handler: `create`

```
async fn create(Ctx(ctx, connection_pool): Ctx, Query(params): Query<ResourceParams<Field>>, Json(body): Json<CustomFieldCreateBody>) -> ApiResult<Json<CustomFieldInfo>>
```

### Request Body: `CustomFieldCreateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `SmallString` | Yes | Name of the custom field |
| `description` | `Option<SmallString>` | No | Description text |
| `color` | `Option<SmallString>` | No | Hex color code (e.g., `#FF0000`) |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `fields` | query string | Query | Comma-separated field names for field selection |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `CustomFieldInfo` | The created custom field |
| 403 | `ErrorResponse` | Insufficient privileges |
| 409 | `ErrorResponse` | A custom field with this name already exists |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::CustomFieldCreate` privilege.
2. Verify the `name` matches the `CustomField` regex from the server configuration.
3. If `color` is `Some` and is not empty, check it against `COLOR_HEX_REGEX`. If it does not match, return `ApiError::ExpressionFailsRegex`.
4. Open a database transaction.
5. Inside the transaction:
   a. Insert a new `NewCustomField` row with the provided `name`, `description` (empty string if not provided), and `color` (empty string if not provided).
   b. If the insert fails with a database unique violation, return `ApiError::AlreadyExists(ResourceProperty::CustomFieldName)`.
   c. Create a creation snapshot via `snapshot::custom_field::creation_snapshot`.
   d. Return the inserted `CustomField`.
6. Open a second database transaction.
7. Convert the new `CustomField` to `CustomFieldInfo` using `CustomFieldInfo::new`, applying the `fields` filter.
8. Return the response as JSON.

---

## Endpoint: Update Custom Field

- **Method:** PUT
- **Path:** `/custom-field/{id}`
- **Authentication:** Required
- **Privileges:** `Action::CustomFieldEditName` (if name, description, or color changes without name change). Version verification always applies.

### Handler: `update`

```
async fn update(Ctx(ctx, connection_pool): Ctx, Path(custom_field_id): Path<i64>, Query(params): Query<ResourceParams<Field>>, Json(body): Json<CustomFieldUpdateBody>) -> ApiResult<Json<CustomFieldInfo>>
```

### Request Body: `CustomFieldUpdateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | `DateTime` | Yes | Resource version for optimistic locking |
| `name` | `Option<SmallString>` | No | New name of the custom field |
| `description` | `Option<SmallString>` | No | New description text |
| `color` | `Option<SmallString>` | No | New hex color code |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `id` | `i64` | Path | Custom field ID |
| `fields` | query string | Query | Comma-separated field names for field selection |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `CustomFieldInfo` | The updated custom field |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Custom field does not exist |
| 409 | `ErrorResponse` | Version is outdated |
| 409 | `ErrorResponse` | Name already exists (unique violation) |

### Handler Logic Step by Step

1. Open a database transaction.
2. Inside the transaction:
   a. Find the existing `CustomField` by primary key `custom_field_id`.
   b. If not found, return `ApiError::NotFound(ResourceType::CustomField)`.
   c. Verify the `body.version` matches `old_cf.last_edit_time`. If not, return `ApiError::ResourceModified`.
   d. Clone the old custom field into a mutable `new_cf`.
   e. If `body.name` is `Some`:
      - Verify it matches the `CustomField` regex.
      - Verify the user has `Action::CustomFieldEditName` privilege.
      - Set `new_cf.name` to the new value.
   f. If `body.description` is `Some`:
      - If `body.name` was `None`, verify the user has `Action::CustomFieldEditName` privilege.
      - Set `new_cf.description` to the new value.
   g. If `body.color` is `Some`:
      - If not empty, verify it matches `COLOR_HEX_REGEX`. If not, return `ApiError::ExpressionFailsRegex`.
      - If `body.name` was `None`, verify the user has `Action::CustomFieldEditName` privilege.
      - Set `new_cf.color` to the new value.
   h. Update `new_cf.last_edit_time` to the current time.
   i. Save the changes. If a unique violation occurs, return `ApiError::AlreadyExists(ResourceProperty::CustomFieldName)`.
   j. Create a modification snapshot via `snapshot::custom_field::modification_snapshot`.
   k. Return the updated `CustomField`.
3. Open a second database transaction.
4. Convert the updated `CustomField` to `CustomFieldInfo` using `CustomFieldInfo::new`, applying the `fields` filter.
5. Return the response as JSON.

---

## Endpoint: Delete Custom Field

- **Method:** DELETE
- **Path:** `/custom-field/{id}`
- **Authentication:** Required
- **Privilege:** `Action::CustomFieldDelete`

### Handler: `delete`

```
async fn delete(Ctx(ctx, connection_pool): Ctx, Path(custom_field_id): Path<i64>, Json(client_version): Json<DeleteBody>) -> ApiResult<Json<()>>
```

### Request Body: `DeleteBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | `DateTime` | Yes | Resource version for optimistic locking |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `id` | `i64` | Path | Custom field ID |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | Empty JSON `()` | Custom field deleted |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | Custom field does not exist |
| 409 | `ErrorResponse` | Version is outdated |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::CustomFieldDelete` privilege.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the `CustomField` by primary key `custom_field_id`.
   b. If not found, return `ApiError::NotFound(ResourceType::CustomField)`.
   c. Verify `cf.last_edit_time` matches `*client_version`. The `DeleteBody` dereferences to `DateTime` via its `Deref` impl.
   d. Create a deletion snapshot via `snapshot::custom_field::deletion_snapshot`.
   e. Execute a `DELETE` query on the `custom_field` table by the given ID.
   f. Return `Json(())`.
4. Return the response.
