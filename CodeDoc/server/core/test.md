# test.rs — Test Helpers and Database Fixtures

## Purpose

This file provides the test infrastructure for the entire server codebase. It creates a test database, populates it with known data, and provides helpers for testing API endpoints. Available only in test builds (`#[cfg(test)]`).

## Constants

### Authentication Constants

| Constant | Type | Value |
|----------|------|-------|
| `TEST_PASSWORD` | `&str` | `"test_password"` |
| `TEST_SALT` | `&str` | `"test_salt"` |
| `TEST_HASH` | `&str` | Pre-computed Argon2id hash for test_password with test_salt |
| `TEST_TOKEN` | `Uuid` | `67e55044-10b1-426f-9247-bb680e5fe0c8` — Valid, enabled token |
| `EXPIRED_TOKEN` | `Uuid` | `b7188ca3-1391-4abf-bfc1-7d7dfad7d161` — Expired token |
| `DISABLED_TOKEN` | `Uuid` | `86c5b652-7c9c-4846-8ae3-5ec236f57c4e` — Disabled token |

### Size Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `MB` | `1024_i64.pow(2)` | Megabytes |
| `GB` | `1024_i64.pow(3)` | Gigabytes |

## Public Functions

### `get_connection() -> Result<Connection, PoolError>`

Gets a blocking database connection from the test pool. Delegates to `get_state().connection_pool.get_blocking()`.

### `get_state() -> AppState`

Gets the singleton test `AppState`. Lazily creates the test database on first call. Uses a static `Mutex<Option<AppState>>` so the database is created once and reused across tests.

### `reset_database()`

Resets the singleton test state by setting it to `None`. The next call to `get_state()` will recreate the database and re-populate it.

### `reset_sequence(table: ResourceType) -> ApiResult<()>`

Resets the auto-increment sequence for a table. Uses `SELECT setval(pg_get_serial_sequence(...))` to set the sequence to the maximum existing ID. This is needed when an insert fails due to a constraint violation (e.g., unique violation) because the sequence may have advanced past the actual data.

### `media_path(filename: &str) -> PathBuf`

Returns the path to a test media file in `test/media/`.

### `simulate_upload(media_filename: &str, temp_name: &str) -> std::io::Result<u64>`

Copies a test media file to the temporary uploads directory under the given `temp_name`. Returns the number of bytes copied. Used to simulate file uploads for test requests.

### `verify_response(request: &str, relative_path: &str) -> ApiResult<()>`

Verifies an API response against expected JSON files. Runs with administrator credentials.

**Process:**
1. Calls `verify_response_with_user(UserRank::Administrator, request, relative_path)`.

### `verify_response_with_user(user: UserRank, request: &str, relative_path: &str) -> ApiResult<()>`

Like `verify_response` but authenticates as the given user rank.

**Process:**
1. If the user is not anonymous, creates basic auth credentials using `header::basic_credentials_for`.
2. Calls `verify_response_with_credentials`.

### `verify_response_with_credentials(credentials: Option<String>, request: &str, relative_path: &str) -> ApiResult<()>`

The core response verification function.

**Process:**
1. Reads files from `test/request/{relative_path}/`:
   - `response.json` — Expected JSON response body (required)
   - `snapshot.json` — Expected snapshot data (optional)
   - `body.json` — Request body to send (optional)
   - `config.toml` — Test-specific config override (optional)
2. Creates a test `AppState`, optionally overriding the config (but not `data_dir`).
3. Builds an Axum router using `api::routes(app_state)` with `NormalizePathLayer`.
4. Parses the request string (`"METHOD /path"`) and creates a test request.
5. Adds `Authorization` header if credentials are provided.
6. Sends the request with optional JSON body.
7. Compares the actual response JSON to `response.json` using `verify_json`.
8. If `snapshot.json` exists, queries the latest snapshot from the database and compares.

**Panics if:**
- `response.json` is missing from the test directory.
- The response body is not valid JSON.
- An unexpected file is found in the test directory.

## Test Data Constants

### `USERS: &[NewUser]`

Five test users with different ranks:
1. `restricted_user` — `UserRank::Restricted` (email: `google.com`)
2. `regular_user` — `UserRank::Regular` (email: `email@domain.com`)
3. `power_user` — `UserRank::Power` (email: `example@hotmail.com`)
4. `moderator` — `UserRank::Moderator` (no email)
5. `administrator` — `UserRank::Administrator` (no email)

All users share `TEST_HASH`, `TEST_SALT`, and `AvatarStyle::Gravatar`.

### `USER_TOKENS: &[NewUserToken]`

Three test tokens:
1. `TEST_TOKEN` — Enabled, belongs to administrator (user_id: 5), no expiration
2. `EXPIRED_TOKEN` — Enabled, belongs to regular_user (user_id: 2), expired at test_date
3. `DISABLED_TOKEN` — Disabled, belongs to regular_user (user_id: 2), no expiration

### Pool Data

| Constant | Value |
|----------|-------|
| `POOL_CATEGORY_NAMES` | `["Setting", "Style"]` |
| `DEFAULT_POOLS` | `[["favs"]]` |
| `SETTINGS_POOLS` | `[["fantasy"], ["steampunk"], ["cyberpunk"]]` |
| `STYLES_POOLS` | `[["abstract"], ["realistic"]]` |
| `POOL_GROUPS` | `[DEFAULT_POOLS, SETTINGS_POOLS, STYLES_POOLS]` |

### Tag Data

| Constant | Value |
|----------|-------|
| `TAG_CATEGORY_NAMES` | `["Artist", "Source", "Character", "Surroundings", "Meta"]` |
| `DEFAULT_TAGS` | `[["tagme", "tag_me"]]` |
| `ARTIST_TAGS` | `[["shakespeare"], ["george_lucas"], ["hidetaka_miyazaki"]]` |
| `SOURCE_TAGS` | `[["classic_literature"], ["star_wars"], ["sekiro"]]` |
| `CHARACTER_TAGS` | 11 tags including `claudius`, `luke_skywalker`, `isshin_ashina`, etc. |
| `SURROUNDINGS_TAGS` | 11 tags including `plant`/`foliage`, `forest`/`woods`, `river`/`stream`/`creek`, etc. |
| `META_TAGS` | `[["high_resolution", "high_res"], ["16:9_aspect_ratio"]]` |

### `TAG_IMPLICATIONS`

| Parent | Child |
|--------|-------|
| `tree` | `plant` |
| `forest` | `plant` |
| `forest` | `tree` |
| `river` | `water` |
| `desert` | `sand` |
| `night_sky` | `night` |
| `night_sky` | `sky` |

### `TAG_SUGGESTIONS`

| Parent | Child |
|--------|-------|
| `river` | `sand` |
| `river` | `plant` |

### `POSTS: &[NewPost]`

Five test posts:
1. `starry_night.png` — 1 MB, 1000x2000, Safe, JPEG, user 1
2. `gif.gif` — 5 MB, 1980x1080, Sketchy, GIF animation, user 2
3. `bmp.bmp` — 92 MB, 11146x7479, Safe, BMP, user 2
4. `1_pixel.png` — 1 byte, 1x1, Safe, PNG, user 2
5. `mp4.mp4` — 100 GB, 1980x1080, Unsafe, MP4 video with Sound flag, anonymous

### `POST_TAGS`

Maps tags to posts:
- Post 1: `shakespeare`, `classic_literature`, `claudius`, `laertes`, `plant`, `rock`
- Post 2: `george_lucas`, `star_wars`, `luke_skywalker`, `darth_vader`, `princess_leia`, `admiral_ackbar`, `forest`, `tree`, `plant`, `rock`, `river`, `water`, `16:9_aspect_ratio`
- Post 3: `high_resolution`, `tagme`
- Post 4: (none)
- Post 5: `hidetaka_miyazaki`, `sekiro`, `isshin_ashina`, `black_hat_badger`, `sekiro_(sekiro)`, `night_sky`, `night`, `sky`, `16:9_aspect_ratio`

### `POOL_POSTS`

| Pool | Post |
|------|------|
| `fantasy` | 1 |
| `fantasy` | 2 |
| `cyberpunk` | 2 |
| `abstract` | 4 |
| `fantasy` | 5 |
| `realistic` | 5 |

### `POST_RELATIONS`

| Parent | Child |
|--------|-------|
| 1 | 2 |
| 1 | 3 |
| 4 | 5 |

### `POST_FAVORITES`

| User | Post |
|------|------|
| 1 | 1 |
| 2 | 2 |
| 2 | 3 |
| 2 | 4 |
| 5 | 5 |

### `POST_FEATURES`

| User | Post |
|------|------|
| 5 | 5 |
| 4 | 4 |
| 3 | 1 |
| 3 | 3 |
| 3 | 1 |

### `POST_SCORES`

| User | Post | Score |
|------|------|-------|
| 1 | 5 | Dislike |
| 2 | 1 | Like |
| 2 | 2 | Like |
| 2 | 3 | Like |
| 4 | 4 | Like |
| 5 | 4 | Dislike |
| 5 | 5 | Like |

### `COMMENTS`

| User | Post | Text |
|------|------|------|
| 2 | 1 | "Cool post!" |
| 5 | 1 | "how did you post this" |
| 2 | 4 | "I don't think this uploaded correctly" |
| Anonymous | 5 | "Lorem ipsum..." |

### `COMMENT_SCORES`

| Comment | User | Score |
|---------|------|-------|
| 1 | 1 | Like |
| 1 | 3 | Like |
| 2 | 1 | Dislike |
| 2 | 4 | Like |
| 3 | 1 | Dislike |
| 3 | 3 | Dislike |
| 3 | 4 | Dislike |
| 3 | 5 | Dislike |

## Static State

```rust
static TEST_STATE: Mutex<Option<AppState>> = Mutex::new(None);
```

A global mutex holding the optional test state. A single test database is created and reused across tests.

### `get_state_guard() -> MutexGuard<'static, Option<AppState>>`

Locks the `TEST_STATE` mutex. If the mutex is poisoned (a test panicked while holding the lock), the state is reset to `None` to reconstruct the database on the next access.

## Private Functions

### `recreate_database() -> AdminResult<AppState>`

Creates a fresh test database:

1. Loads `.env` (if `load_env` feature is enabled).
2. Generates a random test data directory in the system temp directory.
3. Connects to the `postgres` database and drops/recreates `__test` database.
4. Creates a test connection pool and runs migrations.
5. Calls `populate_database` to fill the database with fixture data.
6. Returns `AppState` with the test pool and config.

### `new_user(name, email, rank) -> NewUser<'static>`

Const function to create a `NewUser` with `TEST_HASH`, `TEST_SALT`, and `AvatarStyle::Gravatar`.

### `create_tag_categories(conn)` / `create_tags(conn)`

Insert tag categories and tags from the constant arrays. Each tag may have multiple aliases. Tag implications and suggestions are also inserted.

### `create_pool_categories(conn)` / `create_pools(conn)`

Insert pool categories and pools from the constant arrays. Each pool may have multiple name aliases.

### `create_posts(conn, config)`

Inserts the test posts and sets up their filesystem data:
- Copies the test media file to the content path.
- Generates a post signature using `signature::compute` and `signature::generate_indexes`.
- Copies a placeholder image to the generated thumbnail path.

### `populate_database(conn, config)`

Orchestrates all database population in order:
1. Users → 2. User tokens → 3. Tag categories → 4. Tags → 5. Pool categories → 6. Pools → 7. Posts → 8. Post relations → 9. Post tags → 10. Pool posts → 11. Post favorites → 12. Post features → 13. Post scores → 14. Post notes → 15. Comments → 16. Comment scores

### `asset_path(folder_path, relative_path) -> PathBuf`

Builds a path relative to `$CARGO_MANIFEST_DIR/test/{folder_path}/{relative_path}`.

### `verify_json(test_name, json_type, expected, actual)`

Compares two JSON values. On mismatch, computes a diff via `crate::snapshot::value_diff` and panics with a descriptive message including the expected, actual, and diff.

## Test Module: `statistics_tests`

Contains tests that verify the statistics tables are populated correctly:

| Test | What It Verifies |
|------|-----------------|
| `database_statistics` | Global counts for posts, tags, users, pools, comments, disk usage |
| `comment_statistics` | Comment score totals match expected scores |
| `pool_category_statistics` | Pool category usage counts |
| `pool_statistics` | Pool post counts |
| `post_statistics` | Per-post tag count, pool count, note count, comment count, relation count, score, favorite count, feature count |
| `tag_category_statistics` | Tag category usage counts |
| `tag_statistics` | Tag usage counts |
| `user_statistics` | Per-user comment count, favorite count, upload count |
| `reset_statistics` | Serial test that resets all statistics via `database::reset_relation_stats` and then verifies all statistics are still correct |

All tests use `#[serial_test::parallel]` except `reset_statistics` which uses `#[serial]` because it modifies shared state.
