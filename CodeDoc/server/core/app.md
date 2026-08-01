# app.rs — Application State, Startup, and Shutdown

## Purpose

This file defines application-wide state, startup initialization, tracing setup, graceful shutdown, and the HTTP server runner.

## Struct: `AppState`

```rust
pub struct AppState {
    pub connection_pool: Arc<AsyncConnectionPool>,
    pub config: Arc<Config>,
    pub content_cache: Arc<Mutex<RingCache>>,
}
```

Shared application state passed to every request handler. All fields are wrapped in `Arc` for cheap cloning across threads.

| Field | Type | Purpose |
|-------|------|---------|
| `connection_pool` | `Arc<AsyncConnectionPool>` | Shared database connection pool |
| `config` | `Arc<Config>` | Immutable server configuration |
| `content_cache` | `Arc<Mutex<RingCache>>` | Ring buffer cache for uploaded content |

### Method: `AppState::new(connection_pool, config) -> Self`

Creates a new `AppState`. Initializes the content cache with a capacity of 10 elements (`CONTENT_CACHE_SIZE`). This constant matches the expected number of concurrent uploads.

### Method: `AppState::make_context(self, client: Client) -> Ctx`

Creates a per-request `Ctx` value from the app state and authenticated client. The `Ctx` bundles the `Context` (client + config + cache) with an `Arc<AsyncConnectionPool>`.

## Struct: `Context`

```rust
pub struct Context {
    pub client: Client,
    pub config: Arc<Config>,
    pub content_cache: Arc<Mutex<RingCache>>,
}
```

Per-request context. Contains the authenticated client, server config, and content cache.

| Field | Type | Purpose |
|-------|------|---------|
| `client` | `Client` | Authenticated user client information |
| `config` | `Arc<Config>` | Shared server configuration |
| `content_cache` | `Arc<Mutex<RingCache>>` | Content ring cache |

### Method: `Context::has_privilege(&self, action: Action) -> bool`

Checks whether the authenticated client's rank meets or exceeds the rank required for the given `action`. Returns `true` if the client has sufficient privileges.

### Method: `Context::verify_privilege(&self, action: Action) -> ApiResult<()>`

Verifies the privilege. Returns `Ok(())` if the client has the privilege. Returns `Err(ApiError::InsufficientPrivileges)` otherwise.

### Method: `Context::get_content_cache(&self) -> MutexGuard<'_, RingCache>`

Locks the content cache mutex and returns a guard. If the mutex is poisoned (a panic occurred while locked), it recovers by clearing the cache and using the recovered guard. This ensures the server does not crash from a poisoned mutex.

## Function: `num_rayon_threads() -> usize`

Returns the number of threads for the global rayon thread pool. The value is half the available parallelism, with a minimum of 1. Rayon is used for admin commands.

## Function: `enable_tracing(state: &AppState)`

Initializes the `tracing_subscriber` logging subsystem. Reads the log filter string from `state.config.log_filter`. Tries `EnvFilter::try_new` first for strict parsing. Falls back to `EnvFilter::new` if parsing fails, and logs a warning. The layer is configured without timestamps (`without_time()`).

## Function: `load_env() -> dotenvy::Result<()>`

Available only when the `load_env` feature is enabled. Loads environment variables from a `.env` file.

**Logic:**
1. Checks CLI args for `--env-path=<path>`. If found, loads from that specific path.
2. Otherwise, tries `.env` in the current directory, then `../.env` in the parent directory.
3. Returns `Ok(())` on success.

## Function: `initialize(state: &AppState) -> Result<(), Box<dyn Error + Send + Sync>>`

Performs all startup initialization:

1. **Database migrations** — Calls `db::run_database_migrations()`. Returns the range of migration numbers that were applied.
2. **Server migrations** — Calls `db::run_server_migrations()`. Runs post-migration logic like recomputing filenames or checksums.
3. **Admin mode check** — If `admin::enabled()` returns true, runs `admin::command_line_mode()` and exits the process immediately.
4. **Signature version check** — Calls `db::check_signature_version()`. Blocks until the signature version in the database matches the compiled-in version.
5. **Snapshot counter init** — Calls `middleware::initialize_snapshot_counter()`.
6. **Temp file purge** — Calls `filesystem::purge_temporary_uploads()`. Warns on failure, does not abort.
7. **Cleanup task** — Spawns `filesystem::spawn_temporary_uploads_cleanup_task()` to periodically clean stale uploads.

## Function: `run(state: AppState) -> std::io::Result<()>`

Starts the HTTP server:

1. Creates the router via `api::routes(state)`.
2. Wraps the router with `NormalizePathLayer::trim_trailing_slash()` to strip trailing slashes from URLs.
3. Merges Swagger UI at `/docs` with the API documentation at `/apidoc/openapi.json`.
4. Binds a TCP listener on `0.0.0.0:{port}` where `port` comes from `config::port()`.
5. Serves with graceful shutdown via `shutdown_signal()`.

## Function: `shutdown_signal()`

An async function that waits for shutdown signals:

- **Unix**: Listens for both `Ctrl+C` (SIGINT) and `SIGTERM`.
- **Non-Unix**: Listens only for `Ctrl+C`.

When either signal arrives, logs "Stopping server..." and the graceful shutdown begins.
