# Admin Input Parsing (`admin/input.rs`)

## Summary

This module provides the interactive input layer for the admin CLI. It uses the `rustyline` library for line editing, tab completion, history, and hints. It defines editor types for post queries, task selection, and user input.

## Imports

- `admin::{AdminError, AdminResult, AdminTask}`: Admin error types and task enum.
- `app::AppState`: Application state.
- `search::post::Token as PostToken`: Post search tokens for field completion.
- `search::user::Token as UserToken`: User search tokens for field completion.
- `rustyline::*`: Line editor, completion, history, configuration types.
- `std::marker::PhantomData`: Zero-sized type for generic type markers.
- `strum::{EnumMessage, IntoEnumIterator}`: Enum iteration and messages.
- `thiserror::Error`: Error derive.
- `tracing::error`: Logging.

## Type Aliases

```rust
pub type PostEditor = Editor<EnumCompleter<PostToken>, DefaultHistory>;
pub type TaskEditor = Editor<EnumCompleter<AdminTask>, DefaultHistory>;
pub type UserEditor = Editor<EnumCompleter<UserToken>, DefaultHistory>;
```

Three specialized editor types:
- `PostEditor`: Auto-completes `PostToken` values (search field names like `id`, `tag`, `pool`).
- `TaskEditor`: Auto-completes `AdminTask` values (task names like `check_integrity`).
- `UserEditor`: Auto-completes `UserToken` values (user search fields).

All use `DefaultHistory` for command history storage.

## Enum: `CancelType`

```rust
#[derive(Debug, Error)]
pub enum CancelType {
    #[error("User has cancelled task")]
    Stop,
    #[error("User has exited program")]
    Exit,
}
```

Represents two cancellation states:
- `Stop`: The user typed `done` to cancel the current task (returns to task selection).
- `Exit`: The user typed `exit` to quit the program entirely.

### `TryFrom<&str> for CancelType`

Parses a string into a `CancelType`:
- `"done"` maps to `CancelType::Stop`.
- `"exit"` maps to `CancelType::Exit`.
- Any other string returns `Err(())`.

## Struct: `EnumCompleter<E>`

```rust
pub struct EnumCompleter<E> {
    response_count: Option<usize>,
    _phantom_data: PhantomData<E>,
}
```

A generic tab-completion helper for enums. Uses `PhantomData<E>` because `E` is only used at the type level.

**Fields:**
- `response_count: Option<usize>`: When `Some`, the completer is in mock mode. It returns a set number of pre-determined responses. When `None`, it reads from stdin as normal.

### Method: `EnumCompleter::new() -> Self`

Creates a standard completer. Sets `response_count` to `None` (real interactive mode).

### Method: `EnumCompleter::mocked() -> Self`

Creates a mock completer for testing. Sets `response_count` to `Some(0)`.

## Trait Implementation: `Completer for EnumCompleter<E>`

Where `E: IntoEnumIterator + Into<&'static str>`.

**Method: `complete(&self, line: &str, pos: usize, _ctx: &Context) -> Result<(usize, Vec<Pair>)>`**

Provides tab completion for the enum variants.

**Algorithm:**
1. Find the start of the current word: search left from `pos` for the last space. If none, start is 0.
2. Extract `prefix` from `line[start..pos]`.
3. Iterate over all variants of `E` via `E::iter()`.
4. Convert each variant to a `&'static str` via `.into()`.
5. Filter: keep variants whose string representation starts with `prefix`.
6. Map each match to a `Pair { display, replacement }` (both set to the variant name).
7. Return `(start, matches)` where `start` is the word-start position.

## Trait Implementations (Stubs)

These are minimal implementations that satisfy `rustyline::Helper` trait bounds but add no behavior:

- `impl<E> Hinter for EnumCompleter<E>`: Type `Hint = String`. No method implementations.
- `impl<E> Highlighter for EnumCompleter<E>`: No method implementations (default highlight).
- `impl<E> Validator for EnumCompleter<E>`: No method implementations (all input is valid).
- `impl<E: IntoEnumIterator + Into<&'static str>> Helper for EnumCompleter<E>`: Combines Completer, Hinter, Highlighter, Validator.

## Function: `create_editor<E>() -> Editor<EnumCompleter<E>, DefaultHistory>`

Where `E: IntoEnumIterator + Into<&'static str>`.

Creates a new rustyline `Editor` with tab completion for enum `E`.

**Algorithm:**
1. Build a `Config` with `completion_type(CompletionType::List)`.
2. Create an `Editor` with this config. Panics on failure.
3. Set the helper to a new `EnumCompleter::new()`.
4. Return the editor.

## Function: `create_mock_editor<E>() -> Editor<EnumCompleter<E>, DefaultHistory>`

Where `E: IntoEnumIterator + Into<&'static str>`.

Same as `create_editor` but uses `EnumCompleter::mocked()` as the helper. Used in tests.

## Function: `read<E>(prompt, editor) -> Result<String, CancelType>`

```rust
pub fn read<E>(prompt: &str, editor: &mut Editor<EnumCompleter<E>, DefaultHistory>) -> Result<String, CancelType>
```

Where `E: IntoEnumIterator + Into<&'static str>`.

Prompts the user for input and returns the trimmed string.

**Algorithm:**
1. Enter an infinite loop.
2. **Mock mode check:** If `editor.helper_mut()` has a `response_count` field:
   - If `response_count == 0`: return `Ok(String::new())` (empty input on first call).
   - Otherwise: return `Err(CancelType::Stop)`.
   - Increment `response_count` each call.
3. **Real mode:** Call `editor.readline(prompt)`.
   - On `Ok(line)`: Trim the line, add to history. Handle special commands:
     - `"help"`: Print help info via `print_info()`, then continue the loop.
     - `"clear"`: Call `editor.clear_screen()`, then continue the loop.
     - Other: Try `CancelType::try_from(trimmed)`. If it matches `"done"` or `"exit"`, return the corresponding `Err(CancelType)`. Otherwise return `Ok(trimmed.to_owned())`.
   - On `Err(ReadlineError::Interrupted)` (Ctrl+C): Print `^C` and return `Err(CancelType::Exit)`.
   - On other errors: Print the error and continue the loop.

## Function: `user_input_loop<F, E>(state, editor, function)`

```rust
pub fn user_input_loop<F, E>(
    state: &AppState,
    editor: &mut Editor<EnumCompleter<E>, DefaultHistory>,
    mut function: F,
)
```

Where `F: FnMut(&AppState, &mut Editor<EnumCompleter<E>, DefaultHistory>) -> AdminResult<()>` and `E: IntoEnumIterator + Into<&'static str>`.

Runs a loop that repeatedly calls `function` until the user requests stop or exit.

**Algorithm:**
1. Enter an infinite loop.
2. Call `function(state, editor)`.
3. On `Ok(())`: Continue the loop.
4. On `Err(AdminError::Cancel(CancelType::Stop))`: Break the loop (user typed `done`).
5. On `Err(AdminError::Cancel(CancelType::Exit))`: Call `std::process::exit(0)` (user typed `exit`).
6. On other `Err(err)`: Log the error and continue the loop.

This allows a task to prompt the user multiple times (e.g., for different users/posts) before returning.

## Function: `print_info()`

Prints help information to the console.

**Output sections:**

1. **Commands:**
   - `help`: Show this help message.
   - `clear`: Clear the screen.
   - `exit`: Exit the CLI.

2. **Tasks:**
   - Lists every `AdminTask` variant with its name and message.
   - Formats with dynamic spacing: the longest task name + 4 characters.

3. **Post Selection:**
   - Instructions for selecting posts via search query.
   - Leave blank to select all posts.
   - Supports anonymous ID filters and named filters.
   - Example: `100..500 -tag::tagme` (posts with ID 100-500 not tagged "tagme").

4. **Tip:**
   - Press Ctrl+C to cancel a running task.
