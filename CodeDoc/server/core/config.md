# config.rs — Configuration Loading and Types

## Purpose

This file defines the server configuration types, deserialization logic, privilege system, regex rules, and database URL construction.

## Enum: `RegexType`

| Variant | String Representation | Purpose |
|---------|----------------------|---------|
| `Pool` | `pool` | Pool name regex |
| `PoolCategory` | `pool category` | Pool category name regex |
| `Tag` | `tag` | Tag name regex |
| `TagCategory` | `tag category` | Tag category name regex |
| `Username` | `username` | Username regex |
| `Password` | `password` | Password regex |
| `CustomField` | `custom field` | Custom field name regex |

Derives `Debug`, `Display`, `Clone`, `Copy`. Display serializes to lowercase except for `PoolCategory`, `TagCategory`, `CustomField`.

## Struct: `ThumbnailConfig`

| Field | Type | Purpose |
|-------|------|---------|
| `avatar_width` | `u32` | Width of avatar thumbnails |
| `avatar_height` | `u32` | Height of avatar thumbnails |
| `post_width` | `u32` | Width of post thumbnails |
| `post_height` | `u32` | Height of post thumbnails |

### Method: `ThumbnailConfig::avatar_dimensions() -> (u32, u32)`

Returns `(avatar_width, avatar_height)`.

### Method: `ThumbnailConfig::post_dimensions() -> (u32, u32)`

Returns `(post_width, post_height)`.

## Struct: `SmtpConfig`

| Field | Type | Purpose |
|-------|------|---------|
| `host` | `SmallString` | SMTP server hostname |
| `port` | `Option<u16>` | SMTP port (optional) |
| `username` | `Option<SmallString>` | SMTP username |
| `password` | `Option<SmallString>` | SMTP password |
| `from` | `Mailbox` | From address for emails |

## Struct: `AnonymousPreferences`

| Field | Type | Purpose |
|-------|------|---------|
| `tag_blacklist` | `Vec<SmallString>` | Tags hidden from anonymous users |
| `tag_category_blacklist` | `Vec<SmallString>` | Tag categories hidden from anonymous users |
| `hide_unsafe` | `bool` | Hide unsafe posts from anonymous |
| `hide_sketchy` | `bool` | Hide sketchy posts from anonymous |
| `hide_untagged` | `bool` | Hide untagged posts from anonymous |

### Method: `AnonymousPreferences::is_empty() -> bool`

Returns `true` if all blacklists are empty and all hide flags are false.

## Enum: `Action`

An exhaustive enum of all privilege-controlled actions in the system. Categories:

**User actions:** CreateSelf, CreateAny, List, View, EditAny/EditSelf (Name, Pass, Email, Avatar, Rank), DeleteAny, DeleteSelf

**Token actions:** ListAny/ListSelf, CreateAny/CreateSelf, EditAny/EditSelf, DeleteAny/DeleteSelf

**Post actions:** CreateAnonymous, CreateIdentified, List, ReverseSearch, View, ViewFeatured, Edit (Content, Description, Flag, Note, Relation, Safety, Source, InfoDates, Tag, Thumbnail, CustomFields), Feature, Delete, Score, Merge, Favorite, BulkEdit (Tag, Safety, Delete)

**Tag actions:** Create, Edit (Name, Category, Description, Implication, Suggestion), List, View, Merge, Delete

**TagCategory actions:** Create, Edit (Name, Color, Order), List, View, Delete, SetDefault

**Pool actions:** Create, Edit (Name, Category, Description, Post, LaunchDate), List, View, Merge, Delete

**PoolCategory actions:** Create, Edit (Name, Color), List, View, Delete, SetDefault

**Comment actions:** Create, Delete (Any, Own), Edit (Any, Own), List, View, Score

**CustomField actions:** Create, EditName, Delete, List, View

**Other:** SnapshotList, UploadCreate, UploadUseDownloader

Derives `Clone`, `Copy`, `EnumCount`, `EnumIter`, `EnumTable`, `IntoStaticStr`.

## Type: `PrivilegeConfig`

Alias for `ActionTable<UserRank>` (an enum-indexed table mapping each `Action` to a `UserRank`).

### Serialize Implementation

Serializes each action to its string name with the corresponding `UserRank` value as a JSON number.

### Deserialize Implementation

Deserializes from a map of action names to `UserRank` values. Starts with all actions set to `Administrator`. For each action, removes the corresponding key from the map. Reports a missing field error if an action key is absent.

### `PartialSchema` Implementation

Generates an OpenAPI object schema where each action is a required property of type `UserRank`.

## Struct: `PublicConfig`

| Field | Type | Purpose |
|-------|------|---------|
| `name` | `SmallString` | Site name |
| `default_user_rank` | `UserRank` | Default rank for new users |
| `enable_safety` | `bool` | Whether safety filtering is enabled |
| `contact_email` | `Option<SmallString>` | Site contact email |
| `can_send_mails` | `bool` | Whether SMTP is configured (set at runtime) |
| `username_regex` | `Regex` | Regex for valid usernames |
| `password_regex` | `Regex` | Regex for valid passwords |
| `tag_name_regex` | `Regex` | Regex for valid tag names |
| `tag_category_name_regex` | `Regex` | Regex for valid tag category names |
| `custom_field_name_regex` | `Regex` | Regex for valid custom field names |
| `custom_field_default_color` | `SmallString` | Default color for custom fields |
| `secure_cookie` | `bool` | Whether to use secure cookies |
| `privileges` | `PrivilegeConfig` | Privilege rank mapping |

Serialized as `camelCase`. Unknown fields are denied.

## Struct: `Config`

| Field | Type | Purpose |
|-------|------|---------|
| `data_dir` | `PathBuf` | Root directory for data files |
| `data_url` | `String` | Base URL for data file serving |
| `webhooks` | `Vec<Url>` | Webhook URLs for notifications |
| `password_secret` | `SmallString` | Secret for password hashing |
| `content_secret` | `SmallString` | Secret for content URLs |
| `domain` | `Option<SmallString>` | Server domain |
| `delete_source_files` | `bool` | Whether to delete source files after post creation |
| `append_tag_implications_on_post_edit` | `bool` | Whether to auto-append tag implications |
| `post_similarity_threshold` | `f64` | Threshold for reverse image search similarity |
| `pool_name_regex` | `Regex` | Regex for pool names |
| `pool_category_regex` | `Regex` | Regex for pool category names |
| `log_filter` | `String` | Tracing filter directive string |
| `auto_explain` | `bool` | Whether to enable PostgreSQL auto_explain |
| `trusted_proxy` | `bool` | Whether to trust proxy headers |
| `thumbnails` | `ThumbnailConfig` | Thumbnail dimensions |
| `smtp` | `Option<SmtpConfig>` | SMTP configuration |
| `anonymous_preferences` | `AnonymousPreferences` | Anonymous user preferences |
| `public_info` | `PublicConfig` | Public-facing configuration |

### Method: `Config::smtp() -> Option<&SmtpConfig>`

Returns a reference to the SMTP config if present.

### Method: `Config::default_rank() -> UserRank`

Returns the default user rank, clamped to at least `UserRank::Restricted`. Anonymous users cannot be the default.

### Method: `Config::privileges() -> &PrivilegeConfig`

Returns a reference to the privilege configuration from `public_info`.

### Method: `Config::regex(regex_type: RegexType) -> &Regex`

Returns the regex for the given type. Maps each `RegexType` variant to the corresponding field on `Config` or `PublicConfig`.

### Method: `Config::path(directory: Directory) -> PathBuf`

Returns the filesystem path for a data directory. Joins `data_dir` with the directory name from the `Directory` enum.

### Method: `Config::custom_avatar_url(username: &str) -> String`

Returns the URL path to a user's custom avatar, formatted as `{data_url}/avatars/{lowercase_username}.png`.

### Method: `Config::custom_avatar_path(username: &str) -> PathBuf`

Returns the filesystem path to a user's custom avatar, formatted as `{data_dir}/avatars/{lowercase_username}.png`.

## Function: `create() -> Config`

Deserializes the production configuration. Panics if called in a test build. Looks for `--config-path=<path>` in CLI arguments. Defaults to `"config"` (the file name `config.toml`).

## Function: `test_config(override_relative_path: Option<&str>) -> Config`

Available only in test builds. Creates a test configuration with an optional override path relative to `test/request/`.

## Function: `port() -> u16`

Reads the `SERVER_PORT` environment variable. Defaults to `6666` if not set or not parseable.

## Function: `database_url(database_override: Option<&str>) -> String`

Constructs a PostgreSQL connection URL from environment variables:

| Variable | Default |
|----------|---------|
| `POSTGRES_USER` | Required |
| `POSTGRES_PASSWORD` | Required |
| `POSTGRES_HOST` | `localhost` |
| `POSTGRES_PORT` | `5432` |
| `POSTGRES_DB` | Required |

Returns `postgres://{user}:{password}@{hostname}:{port}/{database}`.

## Constant: `DEFAULT_CONFIG`

The content of `config.toml.dist`, embedded at compile time via `include_str!`.

## Function: `create_config(config_path: Option<&str>) -> Config`

Internal function that builds the configuration:

1. Starts with `DEFAULT_CONFIG` as a base source.
2. If a config path is given, adds it as a second source (overrides defaults).
3. Deserializes into `Config`.
4. On parse error, prints to stderr and exits with code 1.
5. Sets `public_info.can_send_mails` to `true` if `smtp` is `Some`.
