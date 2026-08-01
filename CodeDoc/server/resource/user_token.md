# User Token Resource — `resource/user_token.rs`

**Source:** `server/src/resource/user_token.rs` (82 lines)

## Purpose

This file serializes `UserToken` model entities into `UserTokenInfo` API response structs. User tokens are API authentication keys (UUID values) that belong to a user. The token value is only exposed on creation and only when the client has full visibility.

## Dependencies

- `crate::model::user::UserToken` — database model
- `crate::resource::field::Mask` — field selection
- `crate::resource::user::MicroUser` — lightweight user representation
- `crate::string::LargeString` — string type for notes
- `crate::time::DateTime` — timestamp type
- `serde::Serialize` — serialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Field Enum

```rust
pub enum Field {
    Version, User, Note, Enabled,
    ExpirationTime, CreationTime, LastEditTime, LastUsageTime,
    Token,
}
```

## Visibility Enum

```rust
pub enum Visibility {
    Full,        // Token value is visible
    PublicOnly,  // Token value is never visible
}
```

This is a separate enum from `user.rs:Visibility`, with the same name. Both exist in the same crate but different modules. This `Visibility` controls token exposure, not general user data.

## Struct

### `UserTokenInfo`

The API response shape for a user token.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `user_token.last_edit_time` | Resource version |
| `user` | `Option<MicroUser>` | Provided by caller | The token owner |
| `note` | `Option<LargeString>` | `user_token.note` | Description of the token's purpose |
| `enabled` | `Option<bool>` | `user_token.enabled` | Whether the token is still valid |
| `expiration_time` | `Option<Option<DateTime>>` | `user_token.expiration_time` | When the token expires, or `null` if no expiry |
| `creation_time` | `Option<DateTime>` | `user_token.creation_time` | When the token was created |
| `last_edit_time` | `Option<DateTime>` | `user_token.last_edit_time` | When the token was last edited |
| `last_usage_time` | `Option<DateTime>` | `user_token.last_usage_time` | Last time used with `?bump-login` |
| `token` | `Option<Option<String>>` | `user_token.id.to_string()` | The UUID token value |

**Token visibility logic:**
```rust
token: match (fields[Field::Token], visibility == Visibility::Full) {
    (true, true) => Some(Some(user_token.id.to_string())),
    _ => None,
}
```

The token value is only shown when both conditions are true:
1. The `Token` field is requested in the field mask
2. The visibility is `Full`

In all other cases, `token` is `None` (which `skip_serializing_none` omits from the JSON output).

## Constructor

### `UserTokenInfo::new(user, user_token, fields, visibility) -> Self`

The only constructor. Unlike other resource types, this does not use a database connection. The `MicroUser` is provided by the caller. There is no batch construction, no `Batcher`, and no database queries.

**Parameters:**
- `user: MicroUser` — pre-constructed lightweight user (the token owner)
- `user_token: UserToken` — the model entity
- `fields: Mask<Field>` — field selection mask
- `visibility: Visibility` — controls token value exposure

**Processing:** Direct field mapping from the model. No conditional database queries. All data is available from the model or the caller-provided MicroUser.
