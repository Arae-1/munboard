# Core Layer — README

## Overview

The core layer provides the server entry point, application state management, configuration loading, database connectivity, error kind mapping, request extractors, file system operations, schema definitions, custom string types, time utilities, and test infrastructure.

## Module Interaction

The 12 core modules interact in this sequence during startup:

```
main → config::create() → db::create_connection_pool() → AppState::new() → initialize() → run()
```

## Startup Sequence

1. **`main.rs`** — Entry point. Declares all modules, sets clippy allow directives, configures mimalloc on musl targets.
2. **`config::create()`** — Loads `config.toml` (with overlay from `--config-path` CLI arg). Merges with compiled-in default config. Deserializes into `Config` struct.
3. **`db::create_connection_pool()`** — Builds async PostgreSQL pool (`AsyncConnectionPool`) with r2d2 + tokio semaphore. Enables auto_explain on connections if configured.
4. **`AppState::new(pool, config)`** — Creates shared application state with connection pool, config, and content ring cache.
5. **`initialize()`** — Runs database migrations, server migrations, admin mode check, signature version check, snapshot counter init, temp file purge, and spawns upload cleanup task.
6. **`run()`** — Builds the API router via `api::routes(state)`, adds Swagger UI, binds TCP listener, serves with graceful shutdown.

## Key Data Structures

### `AppState`
Shared state passed to every request handler.

| Field | Type | Purpose |
|-------|------|---------|
| `connection_pool` | `Arc<AsyncConnectionPool>` | Shared database connection pool |
| `config` | `Arc<Config>` | Immutable server configuration |
| `content_cache` | `Arc<Mutex<RingCache>>` | Ring buffer cache for uploaded content (10 slots) |

### `Context`
Per-request context.

| Field | Type | Purpose |
|-------|------|---------|
| `client` | `Client` | Authenticated user client |
| `config` | `Arc<Config>` | Shared configuration |
| `content_cache` | `Arc<Mutex<RingCache>>` | Content cache |

Methods: `has_privilege(action)`, `verify_privilege(action)`, `get_content_cache()`.

### `Client` (auth crate)
Authenticated client with `id`, `rank`, and authentication method.

### `Config`
Server configuration with 24 fields including:

| Field | Purpose |
|-------|---------|
| `data_dir` | Root directory for data storage |
| `data_url` | Base URL for serving data files |
| `webhooks` | List of webhook notification URLs |
| `content_secret` | Secret for content URL signing |
| `delete_source_files` | Whether to delete source files after post creation |
| `post_similarity_threshold` | Threshold for reverse image search |
| `trusted_proxy` | Whether to trust proxy headers (X-Forwarded-For) |
| `anonymous_preferences` | Tag/category blacklist and safety hiding for anonymous users |
| `public_info` | Public-facing configuration (name, regexes, privileges, ranks) |

### `PublicConfig`
Exposed to API clients. Contains: site name, default user rank, safety enable flag, username/password/tag name regexes, custom field color default, secure cookie flag, and the privilege configuration mapping each `Action` to a minimum `UserRank`.

### `PrivilegeConfig`
An `ActionTable<UserRank>` — maps every `Action` (e.g., `PostCreate`, `TagEdit`, `UserDeleteAny`) to the minimum `UserRank` required. Covers 60+ privilege actions across users, tokens, posts, tags, pools, comments, and custom fields.

## Error System

`error.rs` defines:

- **`ErrorName`** — Exhaustive enum of 213 named error variants for machine-readable error classification in API responses and OpenAPI docs.
- **`ErrorKind` trait** — Maps library errors (`diesel`, `image`, `lettre`, `axum`, `serde_json`, etc.) to `ErrorName`. Uses a double `.kind().kind()` pattern where outer errors wrap inner errors that also implement `ErrorKind`.

## File System Abstraction

`filesystem.rs` provides:
- **`Directory` enum** — Five data directories: Avatars, Posts, GeneratedThumbnails, CustomThumbnails, TemporaryUploads.
- **File operations** — `save_uploaded_file`, `save_custom_avatar`, `save_post_thumbnail`, `delete_content`, `delete_post`, `swap_posts`, `move_file` (with cross-device fallback).
- **Upload cleanup** — `purge_temporary_uploads` at startup, spawned periodic task (`spawn_temporary_uploads_cleanup_task`) runs every 10 minutes with two-pass stale detection.
- **Size limits** — `max_upload_size` and `max_url_download_size` config keys, default 4 GiB each.

## Time and String Types

### `DateTime` (time.rs)
Wrapper around `time::OffsetDateTime` with RFC 3339 serde, `Timestamptz` Diesel mapping, and OpenAPI schema. Methods: `now()`, `today()`, `yesterday()`, `tomorrow()`, `from_date()`.

### `Timer`
Debug utility that logs elapsed time on drop. Automatically selects units (ns, μs, ms, s).

### `SmallString` (string.rs)
Wrapper around `CompactString` with Small String Optimization (SSO). No heap allocation for strings ≤ 24 bytes. Used for tags, usernames, passwords.

### `LargeString` (string.rs)
Wrapper around `Arc<str>` for immutable string sharing. Cheap to clone. Used for descriptions, source URLs.

## Database

### `AsyncConnectionPool`
Async-safe pool combining r2d2 + tokio `Semaphore`. Methods: `get()` (async with semaphore), `get_blocking()` (direct, no semaphore), `transaction()` (spawn_blocking with transactional closure).

### `ConnectionInitializer`
Customizes connections on acquire. If `auto_explain` is enabled, loads PostgreSQL auto_explain extension and configures log settings.

### Migrations
- `run_database_migrations()` — Runs embedded Diesel migrations.
- `run_server_migrations()` — Runs post-migration logic (filename recomputation, checksum recomputation, statistics reset).
- `check_signature_version()` — Blocks startup until database signature version matches compiled version.

### Schema (schema.rs)
Auto-generated by Diesel CLI. Defines all 31 database tables with column types, primary keys, join relationships, and `allow_tables_to_appear_in_same_query!` declarations.

## Request Extractors

`extract.rs` wraps Axum extractors to convert rejections into `ApiError`:

| Extractor | Wraps | Purpose |
|-----------|-------|---------|
| `Ctx` | `(Context, Arc<AsyncConnectionPool>)` | Per-request context with auth and DB pool |
| `Json<T>` | `axum::Json<T>` | JSON body extraction |
| `JsonOrMultipart<T>` | JSON or `axum::Multipart` | Content-type dispatch for upload endpoints |
| `Path<T>` | `axum::Path<T>` | Path parameter extraction |
| `Query<T>` | `axum::Query<T>` | Query string extraction |

## Test Infrastructure

`test.rs` (only in `cfg(test)` builds) provides:

| Function | Purpose |
|----------|---------|
| `get_state()` | Lazily creates singleton test `AppState` with populated database |
| `get_connection()` | Gets a blocking connection from the test pool |
| `reset_database()` | Resets the singleton state for test isolation |
| `verify_response(request, path)` | Sends a test request and compares response JSON to expected file |
| `verify_response_with_user(rank, request, path)` | Same as above with specific user rank |
| `simulate_upload(media, temp_name)` | Copies a test media file to temp upload directory |
| `media_path(filename)` | Returns path to test media file |

Test data includes: 5 users (Restricted through Administrator), 3 tokens (valid/expired/disabled), 2 pool categories, 5 pools, 5 tag categories, 31 tags with implications/suggestions, 5 posts, post relations/favorites/features/scores, 4 comments with scores.

## Configuration Details

### `RegexType` enum
Maps named regex types (`Pool`, `PoolCategory`, `Tag`, `TagCategory`, `Username`, `Password`, `CustomField`) to their corresponding compiled regex patterns in `Config`.

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `SERVER_PORT` | HTTP server port (default: 6666) |
| `POSTGRES_USER` | Database user |
| `POSTGRES_PASSWORD` | Database password |
| `POSTGRES_HOST` | Database host (default: localhost) |
| `POSTGRES_PORT` | Database port (default: 5432) |
| `POSTGRES_DB` | Database name |

## Documentation Files

| File | Content |
|------|---------|
| [main.md](main.md) | Entry point, module declarations, allocator config |
| [build.md](build.md) | Build script — recompile triggers for migrations and config |
| [app.md](app.md) | `AppState`, `Context`, startup sequence, graceful shutdown |
| [config.md](config.md) | `Config`, `PublicConfig`, `PrivilegeConfig`, regex types, DB URL |
| [db.md](db.md) | `AsyncConnectionPool`, migrations, auto_explain |
| [error.md](error.md) | `ErrorName` enum, `ErrorKind` trait implementations |
| [extract.md](extract.md) | Custom Axum extractors (`Ctx`, `Json`, `JsonOrMultipart`, `Path`, `Query`) |
| [filesystem.md](filesystem.md) | Data directories, file operations, upload cleanup |
| [schema.md](schema.md) | All 31 database table definitions and join relationships |
| [string.md](string.md) | `SmallString` (SSO) and `LargeString` (Arc<str>) types |
| [time.md](time.md) | `DateTime` wrapper and `Timer` utility |
| [test.md](test.md) | Test database setup, fixtures, response verification helpers |
