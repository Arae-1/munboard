# Admin CLI (`admin/mod.rs`)

## Summary

This module is the entry point for the admin command-line interface (CLI). It starts the CLI when the server runs with `--admin`. It parses user commands, runs admin tasks, and manages progress reporting.

## Imports

- `admin::input::CancelType`, `PostEditor`, `TaskEditor`, `UserEditor`: Types for interactive input and cancellation.
- `api::error::ApiError`: API error type.
- `app::{self, AppState}`: Application state and configuration.
- `auth::Client`: Authentication client.
- `model::enums::UserRank`: User privilege levels.
- `diesel::r2d2::PoolError`: Database connection pool errors.
- `rayon::ThreadPoolBuilder`: Parallel task thread pool.
- `signal_hook::consts::{SIGINT, SIGTERM}`: OS signal constants.
- `std::path::Path`: File path handling.
- `std::str::FromStr`: String parsing trait.
- `std::sync::atomic::{AtomicBool, AtomicU64, Ordering}`: Atomic primitives for cross-thread communication.
- `std::sync::{Arc, LazyLock}`: Shared ownership and lazy initialization.
- `strum::{EnumIter, EnumMessage, EnumString, IntoEnumIterator, IntoStaticStr}`: Enum utilities for iteration, messages, and string conversion.
- `thiserror::Error`: Error derive macro.
- `tracing::{error, info}`: Structured logging.

## Module Declarations

- `pub mod database`: Database admin commands (public).
- `mod input`: Admin input parsing utilities (private).
- `pub mod post`: Post admin operations (public).
- `mod user`: User admin operations (private).

## Type Alias: `AdminResult<T>`

```rust
pub type AdminResult<T> = Result<T, AdminError>;
```

A result type that uses `AdminError` as the error variant. Used throughout the admin module.

## Enum: `AdminError`

```rust
#[derive(Debug, Error)]
#[error(transparent)]
pub enum AdminError {
    Cancel(#[from] CancelType),
    #[error("{0}")]
    Error(String),
}
```

Represents errors that occur during admin operations.

**Variants:**
- `Cancel(CancelType)`: The user cancelled the current operation. Uses `#[from]` to auto-convert from `CancelType`.
- `Error(String)`: A string-based error message. Displays the inner string via `#[error("{0}")]`.

**Trait Implementations:**

### `From<&str> for AdminError`
Converts a string slice to `AdminError::Error`. Clones the string into an owned `String`.

### `From<String> for AdminError`
Converts an owned `String` to `AdminError::Error`.

### `From<PoolError> for AdminError`
Converts a database connection pool error to `AdminError::Error`. Uses the error's `to_string()` representation.

### `From<diesel::result::Error> for AdminError`
Converts a Diesel query error to `AdminError::Error`.

### `From<std::io::Error> for AdminError`
Converts an I/O error to `AdminError::Error`.

### `From<walkdir::Error> for AdminError`
Converts a directory walk error to `AdminError::Error`.

### `From<ApiError> for AdminError`
Converts an API error to `AdminError::Error`.

## Enum: `AdminTask`

```rust
#[derive(Clone, Copy, EnumIter, EnumString, EnumMessage, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum AdminTask { ... }
```

Defines all available admin tasks. Each variant has a human-readable message via `strum`'s `EnumMessage`.

**Variants:**
| Variant | Message |
|---------|---------|
| `CheckIntegrity` | Checks integrity of post files |
| `RecomputeChecksums` | Recompute post checksums |
| `RecomputeSignatures` | Rebuild post signatures |
| `RecomputeIndex` | Rebuild reverse search index |
| `RecomputePostTypes` | Recompute post types |
| `RegenerateThumbnails` | Regenerate post thumbnails |
| `ResetPasswords` | Reset user passwords |
| `ResetFilenames` | Rebuild data directory |
| `ResetStatistics` | Rebuild table statistics |
| `ResetThumbnailSizes` | Cache thumbnail sizes |

## Function: `enabled()`

```rust
pub fn enabled() -> bool
```

Checks if the server was started in admin mode. Returns `true` if the command-line argument `--admin` is present. Searches all arguments.

## Function: `command_line_mode(state: &AppState)`

```rust
pub fn command_line_mode(state: &AppState)
```

Starts the admin CLI.

**Algorithm:**
1. Print a welcome message with the thread count.
2. Print help instructions: enter `help` for commands, `done` to escape a task, `exit` to quit.
3. Install OS signal handlers for SIGINT and SIGTERM via `install_signal_handlers()`.
4. Build a global Rayon thread pool with the configured number of threads. Panics on failure.
5. Create three editors: `post_editor` (for post selection), `task_editor` (for task selection), `user_editor` (for username input). Each uses `input::create_editor()`.
6. Enter `input::user_input_loop()` with the `task_editor`.
7. The loop closure:
   - Read user input with prompt "Please select a task: ".
   - Parse the input into an `AdminTask` using `AdminTask::from_str()`. On failure, show valid options via `AdminTask::iter()`.
   - Call `run_task()` with the parsed task and all three editors.
   - If `CANCELLED` is true after the task, log "Task aborted."
   - Return `Ok(())`.

## Function: `mock_editor()`

```rust
pub fn mock_editor() -> PostEditor
```

Creates a mock editor instance for testing. Returns a `PostEditor` with mocked completion. Used in test contexts to avoid blocking on real user input.

## Constants

### `PRINT_INTERVAL`
```rust
const PRINT_INTERVAL: Option<u64> = Some(1000);
```
The interval (in count increments) at which `ProgressReporter` prints status messages. Set to every 1000 iterations.

### `CANCELLED`
```rust
static CANCELLED: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));
```
A lazily-initialized, thread-safe atomic boolean. Set to `true` when SIGINT or SIGTERM is received. All tasks check this flag to support cancellation.

## Struct: `ProgressReporter`

```rust
struct ProgressReporter {
    message: &'static str,
    print_interval: Option<u64>,
    count: AtomicU64,
}
```

An atomic counter that prints a message with the current count at regular intervals.

**Fields:**
- `message`: The message prefix to print (e.g., "Posts checked").
- `print_interval`: How often to print in increments. `None` disables automatic printing.
- `count`: Thread-safe counter using `AtomicU64`.

### Method: `ProgressReporter::new(message, print_interval)`

Creates a new `ProgressReporter`. Sets the count to 0. The message is a static string. The print interval is an optional `u64`.

### Method: `ProgressReporter::increment(&self)`

Atomically increments the count by 1. After incrementing, if `print_interval` is `Some` and the new count is a multiple of it, calls `self.report()`. Uses `Ordering::SeqCst` for the fetch-add.

### Method: `ProgressReporter::report(&self)`

Logs the current state: `"{message}: {count}"` using `tracing::info!`. Loads the count with `Ordering::SeqCst`.

## Function: `is_cancelled()`

```rust
fn is_cancelled() -> Result<(), CancelType>
```

Checks the global `CANCELLED` atomic bool. Returns `Err(CancelType::Stop)` if cancelled, `Ok(())` otherwise. Uses `Ordering::SeqCst`.

## Function: `client()`

```rust
fn client() -> Client
```

Creates an `admin::Client` with:
- `id: None` (no specific user ID)
- `rank: UserRank::Administrator`

This client is used by admin tasks that need authenticated context.

## Function: `run_task(state, task, post_editor, user_editor)`

```rust
fn run_task(state: &AppState, task: AdminTask, post_editor: &mut PostEditor, user_editor: &mut UserEditor)
```

Dispatches a task to the appropriate handler. First resets `CANCELLED` to `false`.

**Dispatch table:**
| `AdminTask` | Handler | Editor |
|-------------|---------|--------|
| `CheckIntegrity` | `post::check_integrity` | `post_editor` |
| `RecomputeChecksums` | `post::recompute_checksums` | `post_editor` |
| `RecomputeSignatures` | `post::recompute_signatures` | `post_editor` |
| `RecomputeIndex` | `post::recompute_indexes` | `post_editor` |
| `RecomputePostTypes` | `post::recompute_post_types` | `post_editor` |
| `RegenerateThumbnails` | `post::regenerate_thumbnails` | `post_editor` |
| `ResetPasswords` | `user::reset_password` | `user_editor` |
| `ResetFilenames` | `database::reset_filenames` | (none) |
| `ResetStatistics` | `database::reset_statistics` | (none) |
| `ResetThumbnailSizes` | `database::reset_thumbnail_sizes` | (none) |

Database tasks do not use an editor because they operate on all records without user selection.

## Function: `get_post_id(path: &Path)`

```rust
fn get_post_id(path: &Path) -> Option<i64>
```

Extracts the post ID from a file path.

**Algorithm:**
1. Get the file name (last path component) as a string. Return `None` if absent.
2. Split the file name on `_` and take the first segment.
3. Parse the segment as `i64`. Return `Some(id)` on success, `None` on failure.

This works because post files are named as `{post_id}_{hash}.{extension}`.

## Function: `install_signal_handlers()`

```rust
fn install_signal_handlers()
```

Registers two signal handlers using `signal_hook::flag::register`:
- `SIGINT` (Ctrl+C): Sets `CANCELLED` to `true`.
- `SIGTERM`: Sets `CANCELLED` to `true`.

Both use a clone of the global `CANCELLED` `Arc<AtomicBool>`. This enables graceful task cancellation. Panics on registration failure (the expect message says "Must be able to register signal handler").
