# User Model

## Database Tables

- `user` — Stores user account information.
- `user_token` — Stores API authentication tokens for users.

## Structs

### `NewUser` (Insertable)

Maps to table: `user`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `&'a str` | The username. |
| `password_hash` | `&'a str` | The hashed password string. |
| `password_salt` | `&'a str` | The salt used for password hashing. |
| `email` | `Option<&'a str>` | The user email address. Null if not provided. |
| `rank` | `UserRank` | The privilege rank of the user. |
| `avatar_style` | `AvatarStyle` | The avatar display style (Gravatar or Manual). |

Diesel annotations:
- `#[diesel(table_name = user)]`
- `#[diesel(check_for_backend(Pg))]`

### `User`

Maps to table: `user`

Derives: `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `name` | `SmallString` | The username. |
| `password_hash` | `String` | The hashed password. |
| `password_salt` | `String` | The password salt. |
| `email` | `Option<SmallString>` | The user email address. Null if not set. |
| `rank` | `UserRank` | The privilege rank. |
| `avatar_style` | `AvatarStyle` | The avatar display style. |
| `creation_time` | `DateTime` | Timestamp when the account was created. |
| `last_login_time` | `DateTime` | Timestamp of the most recent login. |
| `last_edit_time` | `DateTime` | Timestamp when the account was last edited. |
| `custom_avatar_size` | `i64` | **Dead code.** File size of the custom avatar. Marked with `#[allow(dead_code)]`. |
| `search_seed` | `f32` | **Dead code.** A seed value for randomized search ordering. Marked with `#[allow(dead_code)]`. |
| `password_reset_token` | `Option<String>` | **Dead code.** A token for password reset. Marked with `#[allow(dead_code)]`. |
| `password_reset_expires` | `Option<DateTime>` | **Dead code.** Expiration time for the password reset token. Marked with `#[allow(dead_code)]`. |

Diesel annotations:
- `#[diesel(table_name = user)]`
- `#[diesel(check_for_backend(Pg))]`

**Note:** `User` does not have `AsChangeset`, `Associations`, `Serialize`, or `Deserialize` derives.

Custom methods:
- `avatar_url(&self, config: &Config) -> String` — Returns the URL for the user avatar. If `avatar_style` is `Gravatar`, calls `hash::gravatar_url` which generates a Gravatar URL from the username. If `avatar_style` is `Manual`, returns a custom avatar URL from the application configuration.

### `NewUserToken` (Insertable)

Maps to table: `user_token`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Primary key. A UUID v4 token identifier. |
| `user_id` | `i64` | Foreign key to `user.id`. |
| `note` | `Option<&'a str>` | An optional description note for the token. |
| `enabled` | `bool` | Whether the token is active. |
| `expiration_time` | `Option<DateTime>` | When the token expires. Null if it never expires. |

Diesel annotations:
- `#[diesel(table_name = user_token)]`
- `#[diesel(check_for_backend(Pg))]`

### `UserToken`

Maps to table: `user_token`

Derives: `AsChangeset`, `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Primary key. UUID v4 token identifier. |
| `user_id` | `i64` | Foreign key to `user.id`. |
| `note` | `LargeString` | A description note for the token. |
| `enabled` | `bool` | Whether the token is active. |
| `expiration_time` | `Option<DateTime>` | When the token expires. Null if it never expires. |
| `creation_time` | `DateTime` | Timestamp when the token was created. |
| `last_edit_time` | `DateTime` | Timestamp when the token was last edited. |
| `last_usage_time` | `DateTime` | Timestamp of the most recent token usage. |

Diesel annotations:
- `#[diesel(table_name = user_token)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(treat_none_as_null = true)]` — Treats SQL `NULL` values as Rust `None` for optional fields.
- `#[diesel(belongs_to(User))]`

Relationships:
- Belongs to `User` via `user_id`.

The `AsChangeset` derive allows using `UserToken` as a Diesel changeset for updates.

## Type References

- `UserRank` — Defined in `enums.rs`. Stored as `SmallInt`: Anonymous (0), Restricted (1), Regular (2), Power (3), Moderator (4), Administrator (5).
- `AvatarStyle` — Defined in `enums.rs`. Stored as `SmallInt`: Gravatar (0), Manual (1).
- `Uuid` — From the `uuid` crate. Used as the primary key for user tokens.
- `SmallString` — A bounded string type from `crate::string`.
- `LargeString` — A string type from `crate::string`.
- `DateTime` — A timestamp type from `crate::time`.
- `Config` — The application configuration from `crate::config`.
