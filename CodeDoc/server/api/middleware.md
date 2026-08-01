# middleware.rs — API Middleware

## Purpose

This file provides three Axum middleware functions that run on every API request: error logging, authentication, and webhook notification of snapshots.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::app::AppState` | Application state |
| `crate::auth::{Client as AuthClient, header}` | Authentication client and header parsing |
| `crate::model::enums::UserRank` | User rank enum |
| `crate::model::snapshot::Snapshot` | Snapshot database model |
| `crate::schema::{snapshot, user}` | Diesel schemas |
| `crate::string::SmallString` | Compact string |
| `crate::update` | Resource update functions |
| `axum` | Web framework extractors and middleware |
| `diesel` | Database query builder |
| `reqwest::Client` | HTTP client for webhook posting |
| `std::sync::Arc` | Atomic reference counting |
| `std::collections::HashMap` | Hash map for user name lookup |
| `std::sync::atomic::{AtomicI64, Ordering}` | Atomic counter for snapshot tracking |
| `tracing::{error, warn}` | Logging macros |
| `url::Url` | URL type for webhook endpoints |

## Static Data

### `LAST_POSTED_SNAPSHOT`

```
static LAST_POSTED_SNAPSHOT: AtomicI64 = AtomicI64::new(i64::MAX);
```

An atomic counter that tracks the highest snapshot ID already posted to webhooks. Initialized to `i64::MAX` so that on first use, no existing snapshots are posted. Later reset by `initialize_snapshot_counter`.

---

## Middleware: Log Error

### `log_error`

```
pub async fn log_error(req: Request, next: Next) -> Response
```

**Purpose:** Logs errors that occur in downstream handlers.

**Execution order:** Before and after the handler.

### Logic Step by Step

1. Call `next.run(req).await` to process the request through all remaining handlers.
2. Inspect `response.extensions()` for an `Arc<ApiError>`. Errors are stored in response extensions by the `IntoResponse` implementation of `ApiError`.
3. If an error extension is present, log it at `error` level with the status code and error message using `tracing::error!`.
4. Return the response.

The error log format is: `request failed with status=<code> error=<message>`.

---

## Middleware: Authentication

### `auth`

```
pub async fn auth(State(state): State<AppState>, mut request: Request, next: Next) -> ApiResult<Response>
```

**Purpose:** Authenticates the user from the `Authorization` header. Injects an `AuthClient` into request extensions. Optionally updates the user's last login time.

**Execution order:** Before the handler.

### Logic Step by Step

1. Extract the `AUTHORIZATION` header from the request.
2. If the header is present:
   - Convert the header value to a string via `to_str()`.
   - Call `header::authenticate_user(&state, auth_str)`. This function handles both Basic auth (username:password) and Token auth (username:token), returning an `AuthClient`.
3. If no header is present:
   - Create an anonymous `AuthClient` with `AuthClient::new(None, UserRank::Anonymous)`.
4. If the client is not anonymous AND the request URI query string contains `bump-login`:
   - Get a database connection.
   - Call `update::user::last_login_time(conn, user_id)` to update the user's last login timestamp.
5. Insert the `AuthClient` into `request.extensions()`.
6. Run the remaining middleware and handler via `next.run(request).await`.
7. Return the response.

### Error Cases

| Condition | Error |
|-----------|-------|
| `Authorization` header value is not valid UTF-8 | `ApiError::HeaderDeserialization` |
| Authentication fails (wrong password, wrong token, etc.) | `ApiError::FailedAuthentication` |
| Database error during login time update | `ApiError::FailedQuery` or `ApiError::FailedConnection` |

---

## Middleware: Post to Webhooks

### `post_to_webhooks`

```
pub async fn post_to_webhooks(State(state): State<AppState>, request: Request, next: Next) -> ApiResult<Response>
```

**Purpose:** After a modifying request (POST, PUT, DELETE), sends new snapshot data to all configured webhook URLs.

**Execution order:** Before and after the handler.

### Logic Step by Step

1. Determine if the request can modify the database: the method is POST, PUT, or DELETE.
2. Extract the user's IP address:
   - If `state.config.trusted_proxy` is true:
     - Try the `X-Forwarded-For` header (take the first comma-separated value).
     - If not present, try the `X-Real-IP` header.
   - If `trusted_proxy` is false: `user_ip` is `None`.
3. Execute `next.run(request).await` to get the response.
4. If the request was a modifying method:
   a. Get a database connection.
   b. Load `last_posted_snapshot` from the atomic counter.
   c. Query the `snapshot` table for snapshots with `id > last_posted_snapshot`, ordered by `id`.
   d. Collect unique user IDs from the new snapshots.
   e. If there are user IDs, query the `user` table for `(id, name)` pairs and build a `HashMap<i64, String>`.
   f. For each new snapshot, call `post_snapshot` with the state, snapshot, IP, and user name.
5. Return the response.

### Helper: `initialize_snapshot_counter`

```
pub fn initialize_snapshot_counter(conn: &mut PgConnection) -> QueryResult<()>
```

Called during application startup.

1. Query the `snapshot` table for the highest `id`, ordering descending.
2. Use `optional()` — if the table is empty, default to `0`.
3. Store the result in `LAST_POSTED_SNAPSHOT` via `store(Ordering::SeqCst)`.

This ensures that at startup, only snapshots created after the server was started are posted to webhooks.

### Helper: `post_snapshot`

```
fn post_snapshot(state: &AppState, snapshot: Snapshot, user_ip: Option<String>, user_name: Option<String>)
```

**Purpose:** Posts a single snapshot to all configured webhook URLs. Uses a CAS (compare-and-swap) loop to prevent duplicate posting by concurrent threads.

### Logic Step by Step

1. Enter a CAS loop:
   a. Load `last_posted_snapshot`.
   b. If `snapshot.id <= last_posted_snapshot`, return immediately (already posted by another thread).
   c. Try to atomically update `LAST_POSTED_SNAPSHOT` to `snapshot.id` using `compare_exchange`.
   d. If the exchange succeeds, exit the loop. If it fails (another thread updated it), retry.
2. Wrap the snapshot in `Arc`.
3. For each webhook URL in `state.config.webhooks`:
   - Spawn a new `tokio` task calling `post_to_webhook` with the URL, snapshot Arc, IP, and user name.

### Helper: `post_to_webhook`

```
async fn post_to_webhook(url: Url, snapshot: Arc<Snapshot>, user_ip: Option<String>, user_name: Option<String>)
```

**Purpose:** Sends a POST request with the snapshot data to a single webhook URL.

### Logic Step by Step

1. Build HTTP headers:
   - `Referer`: set to the webhook URL.
   - `Content-Type`: `application/json`.
2. Build the `WebhookPayload`:
   - Flatten the `Snapshot` into the payload.
   - Include `user_ip` if `Some`.
   - Include `user_name` if `Some`.
3. Build a `reqwest::Client` with:
   - The headers above.
   - Redirect policy: `Policy::none()` (do not follow redirects).
   - Timeout: 30 seconds.
4. Send a POST request to the webhook URL with the JSON payload.
5. Check the response for HTTP errors via `error_for_status()`.
6. If any error occurs during the POST, log it at `warn` level: `"Could not post snapshot to {url}. Details:\n{err}"`.

### WebhookPayload

| Field | Serialization | Description |
|-------|---------------|-------------|
| (flattened) `Snapshot` fields | Always | All snapshot fields flattened into the payload |
| `user_ip` | Skipped if `None` | IP address of the user who made the request |
| `user_name` | Skipped if `None` | Name of the user who made the request |

---

## Tests

| Test Function | Serial | Description |
|---------------|--------|-------------|
| `unauthorized` | parallel | Tests authentication failures: wrong password, wrong username with basic auth, wrong username with token auth, username-token mismatch, wrong token, expired token, disabled token, missing credentials, unencoded credentials |
