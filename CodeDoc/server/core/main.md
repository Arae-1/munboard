# main.rs — Server Entry Point

## Purpose

This file is the binary entry point of the Oxibooru server. It declares all modules, sets global compiler directives, configures the allocator, and runs the startup sequence.

## Module Declarations

| Module | Purpose |
|--------|---------|
| `admin` | Admin CLI commands |
| `api` | HTTP API routing and error handling |
| `app` | Application state, startup, and shutdown |
| `auth` | Authentication (headers, tokens) |
| `config` | Configuration loading |
| `content` | Media content processing |
| `db` | Database connection pool and migrations |
| `embed` | OpenGraph embed pages for post links |
| `error` | Error kind mapping trait |
| `extract` | Axum request extractor wrappers |
| `filesystem` | File operations for media storage |
| `math` | Mathematical utilities |
| `model` | Data models |
| `resource` | Resource CRUD operations |
| `schema` | Diesel schema definitions |
| `search` | Search query parsing |
| `snapshot` | Change tracking/snapshots |
| `string` | Custom string types (Small/Large) |
| `test` | Test database helpers (available only with `cfg(test)`) |
| `time` | DateTime wrapper and Timer utility |
| `update` | Resource update logic |

## Clippy Allow Directives

| Directive | Reason |
|-----------|--------|
| `needless_for_each` | Disagrees with manual `for_each` usage |
| `large_stack_arrays` | Warns on large derives |
| `too_many_arguments` | API handlers often need multiple parameters |
| `cast_possible_truncation` | Warns on integer casts in const context |
| `cast_possible_wrap` | Same as above |
| `option_option` | `Option<Option<T>>` is useful for nullable JSON fields |
| `float_cmp` | Buggy: fires on `assert_eq!(0.0)` |
| `unreadable_literal` | Too subjective |
| `too_many_lines` | Too subjective |

## Allocator

On `musl` targets, the global allocator is set to `mimalloc::MiMalloc`. The default musl allocator has poor performance.

## Main Function Flow

```
main() -> async fn main()
```

1. If the `load_env` feature is enabled, calls `app::load_env()` to load the `.env` file.
2. Creates `AppState` via `app::AppState::new()`, which takes:
   - `db::create_connection_pool()` — builds the async PostgreSQL connection pool
   - `config::create()` — deserializes the configuration
3. Calls `app::enable_tracing(&state)` to set up the tracing/logging subscriber.
4. Calls `app::initialize(&state)`. On error, logs the error and exits with code 1.
5. Calls `app::run(state).await`. On error, logs the error and exits with code 1.

## Error Handling

Both `initialize` and `run` are fallible. If either fails, the process exits with code 1 after logging the error via `tracing::error!`.
