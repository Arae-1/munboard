# Enums Model

This file defines all enum types, error types, and bitmask types used across the data model. These types are stored in the database as `SmallInt` (`i16`) values. Change the variant order only by appending new variants at the end, because the integer representation maps to database values.

## Error Types

### `ParseExtensionError`

Derives: `Debug`, `Error`, `PartialEq`, `Eq`

A tuple struct wrapping the unrecognized file extension string. Returned by `MimeType::from_extension()` when the extension is not supported.

### `ParseMimeTypeError`

Derives: `Debug`, `Error`, `PartialEq`, `Eq`

A tuple struct wrapping the unrecognized MIME type string. Returned by `MimeType::from_str()` when the MIME type string is not supported.

### `FromRatingError`

Derives: `Debug`, `Error`

A unit struct error. Returned when a `Rating::None` value is converted to `Score` via `TryFrom`.

---

## Database-Backed Enums

All of the enums below implement `ToSql<SmallInt, Pg>` and `FromSql<SmallInt, Pg>` for Diesel. They also implement `AsExpression` and `FromSqlRow` with `#[diesel(sql_type = SmallInt)]` and use `#[repr(i16)]`.

### `AvatarStyle`

Derives: `Debug`, `Default`, `Clone`, `Copy`, `PartialEq`, `Eq`, `FromRepr`, `AsExpression`, `FromSqlRow`, `Serialize`, `Deserialize`, `ToSchema`

Defines how a user avatar is displayed.

| Variant | Repr | Description |
|---------|------|-------------|
| `Gravatar` (default) | 0 | Uses a Gravatar image based on the user name. |
| `Manual` | 1 | Uses a custom uploaded avatar image. |

Serde: `#[serde(rename_all = "lowercase")]` — serializes as `"gravatar"` or `"manual"`.

### `PostType`

Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `EnumString`, `FromRepr`, `AsExpression`, `FromSqlRow`, `Serialize`, `Deserialize`, `ToSchema`

Defines the type of media content of a post.

| Variant | Repr | Description |
|---------|------|-------------|
| `Image` | 0 | A static image file. |
| `Animation` | 1 | An animated image (e.g. animated GIF). |
| `Video` | 2 | A video file. |
| `Document` | 3 | A document file (e.g. PDF). |

Serde: `#[serde(rename_all = "lowercase")]` — serializes as `"image"`, `"animation"`, etc.

Strum: `#[strum(serialize_all = "lowercase")]` — string conversions use lowercase.

### `MimeType`

Derives: `Debug`, `Display`, `Copy`, `Clone`, `PartialEq`, `Eq`, `FromRepr`, `AsExpression`, `FromSqlRow`, `Serialize`, `ToSchema`

Defines a supported MIME type for post content.

| Variant | Repr | Serde Name | File Extensions |
|---------|------|------------|-----------------|
| `Bmp` | 0 | `"image/bmp"` | `bmp`, `dib` |
| `Gif` | 1 | `"image/gif"` | `gif` |
| `Jpeg` | 2 | `"image/jpeg"` | `jpg`, `jpeg`, `jpe`, `jif`, `jfif`, `jfi` |
| `Png` | 3 | `"image/png"` | `png` |
| `Webp` | 4 | `"image/webp"` | `webp` |
| `Mp4` | 5 | `"video/mp4"` | `mp4`, `m4a`, `m4b`, `m4p`, `m4r`, `m4v` |
| `Mov` | 6 | `"video/quicktime"` | `mov`, `movie`, `qt` |
| `Webm` | 7 | `"video/webm"` | `webm` |
| `Avif` | 8 | `"image/avif"` | `avif` |
| `Pdf` | 9 | `"application/pdf"` | `pdf` |

Serde: Each variant has `#[serde(rename = "...")]` to use the full MIME type string.

`Display` uses the `strum` derived display which uses the variant name (e.g. `Bmp`).

Custom methods:
- `from_extension(extension: &str) -> Result<Self, ParseExtensionError>` — Converts a file extension (case-insensitive) to a MIME type.
- `from_path(path: &Path) -> Option<Self>` — Extracts the extension from a path and converts it.
- `extension(self) -> &'static str` — Returns the default file extension for the MIME type.

Custom trait implementations:
- `FromStr` — Parses a MIME type string (e.g. `"image/png"`) to `MimeType`.

### `PostSafety`

Derives: `Debug`, `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `EnumString`, `FromRepr`, `AsExpression`, `FromSqlRow`, `Serialize`, `Deserialize`, `ToSchema`

Defines the safety rating of a post.

| Variant | Repr | Description |
|---------|------|-------------|
| `Safe` | 0 | Safe for all audiences. |
| `Sketchy` | 1 | Questionable content. |
| `Unsafe` | 2 | Explicit content. |

The variants are ordered by severity: `Safe < Sketchy < Unsafe`.

Serde: `#[serde(rename_all = "lowercase")]` — serializes as `"safe"`, `"sketchy"`, `"unsafe"`.

Strum: `#[strum(serialize_all = "lowercase")]`.

### `PostFlag`

Derives: `Clone`, `Copy`, `EnumCount`, `EnumIter`, `EnumString`, `FromRepr`, `IntoStaticStr`, `Deserialize`, `ToSchema`

Defines individual boolean flags that can be set on a post. These are combined into a bitmask (see `PostFlags`).

| Variant | Repr | Description |
|---------|------|-------------|
| `Loop` | 0 | The post content loops (e.g. looping video). |
| `Sound` | 1 | The post content has sound (e.g. video with audio). |

Serde: `#[serde(rename_all = "lowercase")]`.

Strum: `#[strum(serialize_all = "lowercase")]`.

Custom implementations:
- `From<PostFlag> for u16` — Converts a flag to its bit position value (`1 << variant as u16`).

### `PostFlags`

Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Default`, `AsExpression`, `FromSqlRow`, `ToSchema`

**Not an enum.** A newtype wrapper around `u16` that acts as a bitmask of `PostFlag` values. Stored in the database as `SmallInt`.

| Field | Type | Description |
|-------|------|-------------|
| `0` | `u16` | The internal bitmask value. |

Diesel annotations:
- `#[diesel(sql_type = SmallInt)]`

Utoipa annotations:
- `#[schema(value_type = Vec<PostFlag>)]` — The OpenAPI schema serializes this as an array of flags.

Custom methods:
- `new() -> Self` — Creates a `PostFlags` with no flags set (0).
- `new_with(flag: PostFlag) -> Self` — Creates a `PostFlags` with a single flag set.
- `from_slice(flags: &[PostFlag]) -> Self` — Creates a `PostFlags` with multiple flags set.

Custom trait implementations:
- `From<PostFlags> for u16` — Extracts the raw bitmask.
- `BitOr<T: Into<u16>> for PostFlags` — Combines flags with bitwise OR.
- `BitOrAssign<T: Into<u16>> for PostFlags` — Assigns flags with bitwise OR.
- `ToSql<SmallInt, Pg>` — Serializes the bitmask as a signed 16-bit integer.
- `FromSql<SmallInt, Pg>` — Deserializes from a 16-bit integer, casting to unsigned.
- `Serialize` — Serializes as a JSON array of flag name strings. Iterates all `PostFlag` variants and includes those whose bits are set. Asserts at compile time that there are 16 or fewer flags.

### `UserRank`

Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `EnumString`, `FromRepr`, `AsExpression`, `FromSqlRow`, `Serialize`, `Deserialize`, `ToSchema`

Defines the privilege level of a user account.

| Variant | Repr | Description |
|---------|------|-------------|
| `Anonymous` | 0 | Not logged in. |
| `Restricted` | 1 | Limited access account. |
| `Regular` | 2 | Normal user account. |
| `Power` | 3 | Elevated privileges. |
| `Moderator` | 4 | Content moderation privileges. |
| `Administrator` | 5 | Full administrative access. |

The variants are ordered by authority: `Anonymous < Restricted < Regular < Power < Moderator < Administrator`.

Serde: `#[serde(rename_all = "lowercase")]`.

Strum: `#[strum(serialize_all = "lowercase")]`.

### `Rating`

Derives: `Debug`, `Default`, `Clone`, `Copy`, `Serialize_repr`, `Deserialize_repr`

**Not a database type.** A three-valued representation of user sentiment. Used in API responses.

| Variant | Repr | Description |
|---------|------|-------------|
| `Dislike` | -1 | Negative rating. |
| `None` (default) | 0 | No rating given. |
| `Like` | 1 | Positive rating. |

Serde: Uses `Serialize_repr` and `Deserialize_repr` to serialize as the raw integer -1, 0, or 1.

Custom implementations:
- `From<Score> for Rating` — Converts `Score::Dislike` to `Rating::Dislike` and `Score::Like` to `Rating::Like`.
- `ToSchema` — OpenAPI schema returns name `"Rating"`.
- `PartialSchema` — OpenAPI schema describes an integer type with minimum -1, maximum 1, and examples [-1, 0, 1].

### `Score`

Derives: `Debug`, `Clone`, `Copy`, `FromRepr`, `AsExpression`, `FromSqlRow`

Stored in the database as `SmallInt`. Represents a like or dislike. Unlike `Rating`, there is no neutral/zero value.

| Variant | Repr | Description |
|---------|------|-------------|
| `Dislike` | -1 | Negative score. |
| `Like` | 1 | Positive score. |

Diesel annotations:
- `#[diesel(sql_type = SmallInt)]`

Custom implementations:
- `TryFrom<Rating> for Score` — Converts `Rating::Like` to `Score::Like` and `Rating::Dislike` to `Score::Dislike`. Returns `FromRatingError` for `Rating::None` because `None` has no `Score` equivalent.
- `ToSql<SmallInt, Pg>` — Serializes as an `i16`.
- `FromSql<SmallInt, Pg>` — Deserializes from an `i16` using `FromRepr`.

### `ResourceOperation`

Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `EnumString`, `FromRepr`, `AsExpression`, `FromSqlRow`, `Serialize`, `ToSchema`

Defines the type of change recorded in a snapshot.

| Variant | Repr | Description |
|---------|------|-------------|
| `Created` | 0 | A new resource was created. |
| `Modified` | 1 | An existing resource was changed. |
| `Merged` | 2 | Two resources were merged into one. |
| `Deleted` | 3 | A resource was removed. |

Serde: `#[serde(rename_all = "snake_case")]`.

Diesel annotations:
- `#[diesel(sql_type = SmallInt)]`

### `ResourceType`

Derives: `Debug`, `Display`, `Clone`, `Copy`, `PartialEq`, `Eq`, `EnumString`, `FromRepr`, `AsExpression`, `FromSqlRow`, `Serialize`, `ToSchema`

Defines the type of resource that a snapshot tracks.

| Variant | Repr | Description |
|---------|------|-------------|
| `Comment` | 0 | A comment. |
| `CustomField` | 1 | A custom field definition. |
| `Pool` | 2 | A pool. |
| `PoolCategory` | 3 | A pool category. |
| `Post` | 4 | A post. |
| `Tag` | 5 | A tag. |
| `TagCategory` | 6 | A tag category. |
| `TagImplication` | 7 | A tag implication. |
| `TagSuggestion` | 8 | A tag suggestion. |
| `User` | 9 | A user account. |
| `UserToken` | 10 | A user API token. |

Serde: `#[serde(rename_all = "snake_case")]`.

Strum: `#[strum(serialize_all = "snake_case")]`.

Diesel annotations:
- `#[diesel(sql_type = SmallInt)]`

---

## Non-Database Enums

### `ResourceProperty`

Derives: `Debug`, `Display`

Does not map to a database column. Describes a specific property of a resource that was changed. Used in snapshot data context.

Strum: `#[strum(serialize_all = "snake_case")]`.

| Variant | Description |
|---------|-------------|
| `CustomFieldName` | The name of a custom field. |
| `PoolName` | The name of a pool. |
| `PoolPost` | A post within a pool. |
| `PoolCategoryName` | The name of a pool category. |
| `PostContent` | The content file of a post. |
| `PostFeature` | A featured post. |
| `PostRelation` | A relationship between posts. |
| `TagName` | The name of a tag. |
| `TagCategoryName` | The name of a tag category. |
| `UserName` | The name of a user. |
| `UserEmail` | The email address of a user. |

---

## Tests

The module contains two unit tests:

- `safety_ordering` — Verifies that `PostSafety::Safe < Sketchy < Unsafe`.
- `rank_ordering` — Verifies that `UserRank` variants are ordered by increasing privilege.
