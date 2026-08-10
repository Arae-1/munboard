# Admin Command-Line Interface

## Individual Files

- [mod.md](mod.md)
- [database.md](database.md)
- [input.md](input.md)
- [post.md](post.md)
- [user.md](user.md)

## Purpose

The admin module provides a command-line interface (CLI) for database and filesystem maintenance tasks. It is activated when the server is started with the `--admin` flag.

## Entry Point (`mod.rs`)

### `enabled`
Returns true if the server was started with the `--admin` argument.

### `command_line_mode`
Starts the admin CLI:
1. Prints a welcome message with thread count.
2. Installs SIGINT and SIGTERM signal handlers for graceful cancellation.
3. Configures the global Rayon thread pool.
4. Creates three editors: post query editor, task editor, user editor.
5. Enters an input loop that prompts for a task name.

### Admin Commands
Special input commands:
- `help`: Displays available tasks and usage info.
- `clear`: Clears the terminal screen.
- `done`: Stops the current task.
- `exit`: Exits the CLI.
- `Ctrl+C`: Gracefully cancels a running task.

### `AdminTask` Enum
Available tasks:
| Task | Description |
|------|-------------|
| `check_integrity` | Checks integrity of post files |
| `recompute_checksums` | Recompute post checksums |
| `recompute_signatures` | Rebuild post signatures |
| `recompute_index` | Rebuild reverse search index |
| `recompute_post_types` | Recompute post types |
| `regenerate_thumbnails` | Regenerate post thumbnails |
| `reset_passwords` | Reset user passwords |
| `reset_filenames` | Rebuild data directory |
| `reset_statistics` | Rebuild table statistics |
| `reset_thumbnail_sizes` | Cache thumbnail sizes |

### `ProgressReporter`
An atomic counter that prints progress at regular intervals. Used in parallel processing loops to report the count of processed items.

### `CANCELLED`
A global atomic boolean set by SIGINT/SIGTERM handlers. All long-running tasks periodically check this flag.

### `get_post_id`
Extracts the post ID from a content file path by splitting on underscore and parsing the first component.

## Database Operations (`database.rs`)

### `reset_filenames`
Renames post files and thumbnails to match the current PostHash format. Walks through the Posts, GeneratedThumbnails, and CustomThumbnails directories. For each file:
1. Extracts the post ID from the filename.
2. Computes the new path using PostHash.
3. Moves the file if the path differs.

### `reset_thumbnail_sizes`
Updates database records for thumbnail file sizes on disk:
1. Walks the Avatars directory, reads file sizes, and updates `user.custom_avatar_size`.
2. Walks the GeneratedThumbnails directory, reads file sizes, and updates `post.generated_thumbnail_size`.
3. Walks the CustomThumbnails directory, reads file sizes, and updates `post.custom_thumbnail_size`.

### `reset_statistics`
Recalculates all cached file sizes and table statistics:
1. Resets `database_statistics.disk_usage` to 0 (triggers will increment it).
2. Walks Posts directory and updates `post.file_size` for each post.
3. Calls `reset_thumbnail_sizes`.
4. Calls `reset_relation_stats`.

### `reset_relation_stats`
Recalculates all denormalized relation statistics:
- **Database statistics**: Counts comments, pools, posts, tags, users.
- **Comment statistics**: Sums scores for each comment.
- **Pool category statistics**: Counts pools in each category.
- **Pool statistics**: Counts posts in each pool.
- **Tag category statistics**: Counts tags in each category.
- **Tag statistics**: Counts usages, implications, suggestions for each tag.
- **User statistics**: Counts comments, favorites, uploads for each user.
- **Post statistics**: Counts tags, pools, notes, comments, relations, scores, favorites, features, and last times for each post.

## Input Handling (`input.rs`)

### `EnumCompleter<E>`
A rustyline completer that provides tab completion for any enum implementing `IntoEnumIterator` and `Into<&str>`. Used for task names, post search tokens, and user search tokens.

### `create_editor` / `create_mock_editor`
Creates rustyline editors with EnumCompleter helpers. Mock editors return empty responses for testing.

### Type Aliases
- `PostEditor`: Editor for post search query input.
- `TaskEditor`: Editor for task name selection.
- `UserEditor`: Editor for username input.

### `read`
Prompts the user for input:
1. Displays a prompt string.
2. Reads a line from the editor.
3. Handles `help` (prints info), `clear` (clears screen), `done`/`exit` (returns cancel state).
4. Supports mocked responses for testing.

### `user_input_loop`
Executes a function repeatedly in a loop. The function can return:
- `Ok(())`: Continue the loop.
- `Err(Cancel(Stop))`: Break the loop (task completed).
- `Err(Cancel(Exit))`: Exit the program.
- `Err(other)`: Print the error and continue.

### `print_info`
Prints help text showing available commands, tasks with descriptions, post selection syntax, and tips.

## Post Maintenance (`post.rs`)

All post maintenance tasks follow the same pattern:
1. Prompt the user for a post selection query (supports ID ranges, tag filters, etc. via the search query builder).
2. Execute the task in parallel across the selected posts using Rayon.

### `check_integrity`
Compares the stored BLAKE3 checksum with the file's current checksum. Warns on mismatch, which may indicate file corruption.

### `recompute_checksums`
Recomputes BLAKE3 and MD5 checksums for each post and updates the database. Also checks for duplicates (other posts with the same checksum).

### `recompute_signatures`
Decodes each post's content, computes the image similarity signature, and updates the `post_signature` table. Also updates the signature version in `database_statistics`.

### `recompute_indexes`
Regenerates the 100 search index words from existing compressed signatures. Does not require reading post content.

### `recompute_post_types`
Redetects the post type (image/animation/video/document) and updates the database.

### `regenerate_thumbnails`
Decodes each post's content and regenerates the thumbnail. Updates the `generated_thumbnail_size`.

## User Management (`user.rs`)

### `reset_password`
Prompts for a username and new password:
1. Checks that the user exists.
2. Validates the password against the configured password regex.
3. Generates a new Argon2id salt and hashes the password.
4. Updates `user.password_hash` and `user.password_salt` in the database.
