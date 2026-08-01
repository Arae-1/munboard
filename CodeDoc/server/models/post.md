# Post Model

## Database Tables

- `post` — Stores the main post entity with metadata about uploaded media.
- `post_relation` — Stores bidirectional relationships between posts (e.g. parent-child).
- `post_tag` — Junction table linking posts to tags.
- `post_favorite` — Junction table tracking which users favorited which posts.
- `post_feature` — Tracks which posts were featured and by whom.
- `post_note` — Stores annotation notes on posts with polygon coordinates.
- `post_score` — Stores a score (like/dislike) that a user gave to a post.
- `post_signature` — Stores perceptual signature data for duplicate detection.

## Structs

### `NewPost` (Insertable)

Maps to table: `post`

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Option<i64>` | Foreign key to `user.id`. Null if uploaded anonymously. |
| `file_size` | `i64` | Size of the uploaded file in bytes. |
| `width` | `i32` | Width of the media in pixels. |
| `height` | `i32` | Height of the media in pixels. |
| `safety` | `PostSafety` | Safety rating of the post. |
| `type_` | `PostType` | Type of media content. |
| `mime_type` | `MimeType` | MIME type of the media file. |
| `checksum` | `Checksum` | A hash checksum of the file content. |
| `checksum_md5` | `Md5Checksum` | An MD5 hash checksum of the file content. |
| `flags` | `PostFlags` | Bitmask of boolean flags (loop, sound). |
| `source` | `&'a str` | The source URL or description for the post. |
| `description` | `&'a str` | A text description of the post. |
| `launch_date` | `Option<DateTime>` | An optional scheduled publish date. |
| `media_created_date` | `Option<DateTime>` | An optional date when the media was originally created. |

Diesel annotations:
- `#[diesel(table_name = post)]`
- `#[diesel(check_for_backend(Pg))]`

### `Post`

Maps to table: `post`

Derives: `Clone`, `AsChangeset`, `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `user_id` | `Option<i64>` | Foreign key to `user.id`. Null if uploaded anonymously. |
| `file_size` | `i64` | Size of the uploaded file in bytes. |
| `width` | `i32` | Width of the media in pixels. |
| `height` | `i32` | Height of the media in pixels. |
| `safety` | `PostSafety` | Safety rating of the post. |
| `type_` | `PostType` | Type of media content. |
| `mime_type` | `MimeType` | MIME type of the media file. |
| `checksum` | `Checksum` | Hash checksum of the file content. |
| `checksum_md5` | `Md5Checksum` | MD5 hash checksum of the file content. |
| `flags` | `PostFlags` | Bitmask of boolean flags. |
| `source` | `LargeString` | The source URL or description. |
| `creation_time` | `DateTime` | Timestamp when the post was created. |
| `last_edit_time` | `DateTime` | Timestamp when the post was last edited. |
| `generated_thumbnail_size` | `i64` | File size of the auto-generated thumbnail. |
| `custom_thumbnail_size` | `i64` | File size of a custom uploaded thumbnail. |
| `launch_date` | `Option<DateTime>` | Scheduled publish date. |
| `media_created_date` | `Option<DateTime>` | Original media creation date. |
| `description` | `LargeString` | Text description of the post. |

Diesel annotations:
- `#[diesel(table_name = post)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(treat_none_as_null = true)]` — Treats SQL `NULL` values as Rust `None` for optional fields.
- `#[diesel(belongs_to(User))]`

Relationships:
- Belongs to `User` via `user_id`.

The `AsChangeset` derive allows using `Post` as a Diesel changeset for updates.

### `PostRelation`

Maps to table: `post_relation`

Derives: `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `parent_id` | `i64` | Part of composite primary key. Foreign key to `post.id` (parent post). |
| `child_id` | `i64` | Part of composite primary key. Foreign key to `post.id` (child post). |

Diesel annotations:
- `#[diesel(table_name = post_relation)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(parent_id, child_id))]`
- `#[diesel(belongs_to(Post, foreign_key = parent_id))]`

This struct implements `Insertable` directly (no separate `New*` struct).

Relationships:
- Belongs to `Post` via `parent_id`.
- Explicit joinable declaration: `diesel::joinable!(post_relation -> post (parent_id))` allows joining on the `post` table through `parent_id`.

Composite primary key: `(parent_id, child_id)`.

Custom methods:
- `new_pair(id_1: i64, id_2: i64) -> [Self; 2]` — Creates a bidirectional pair of relations: `(id_1, id_2)` and `(id_2, id_1)`.

### `PostTag`

Maps to table: `post_tag`

Derives: `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Part of composite primary key. Foreign key to `post.id`. |
| `tag_id` | `i64` | Part of composite primary key. Foreign key to `tag.id`. |

Diesel annotations:
- `#[diesel(table_name = post_tag)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(post_id, tag_id))]`
- `#[diesel(belongs_to(Post), belongs_to(Tag))]`

Relationships:
- Belongs to `Post` via `post_id`.
- Belongs to `Tag` via `tag_id`.

Composite primary key: `(post_id, tag_id)`.

This struct implements `Insertable` directly.

### `PostFavorite`

Maps to table: `post_favorite`

Derives: `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Part of composite primary key. Foreign key to `post.id`. |
| `user_id` | `i64` | Part of composite primary key. Foreign key to `user.id`. |
| `time` | `DateTime` | Timestamp when the favorite was recorded. |

Diesel annotations:
- `#[diesel(table_name = post_favorite)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(post_id, user_id))]`
- `#[diesel(belongs_to(Post), belongs_to(User))]`

Relationships:
- Belongs to `Post` via `post_id`.
- Belongs to `User` via `user_id`.

Composite primary key: `(post_id, user_id)`.

This struct implements `Insertable` directly.

### `NewPostFeature` (Insertable)

Maps to table: `post_feature`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Foreign key to `post.id`. |
| `user_id` | `i64` | Foreign key to `user.id` (the user who featured the post). |
| `time` | `DateTime` | Timestamp when the post was featured. |

Diesel annotations:
- `#[diesel(table_name = post_feature)]`
- `#[diesel(check_for_backend(Pg))]`

### `PostFeature`

Maps to table: `post_feature`

Derives: `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `post_id` | `i64` | Foreign key to `post.id`. |
| `user_id` | `i64` | Foreign key to `user.id`. |
| `time` | `DateTime` | Timestamp when the post was featured. |

Diesel annotations:
- `#[diesel(table_name = post_feature)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(belongs_to(Post), belongs_to(User))]`

Relationships:
- Belongs to `Post` via `post_id`.
- Belongs to `User` via `user_id`.

### `NewPostNote` (Insertable)

Maps to table: `post_note`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Foreign key to `post.id`. |
| `polygon` | `&'a [f32]` | A slice of f32 values representing polygon vertex coordinates. |
| `text` | `&'a str` | The note text content. |

Diesel annotations:
- `#[diesel(table_name = post_note)]`
- `#[diesel(check_for_backend(Pg))]`

### `PostNote`

Maps to table: `post_note`

Derives: `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `post_id` | `i64` | Foreign key to `post.id`. |
| `polygon` | `Vec<Option<f32>>` | A vector of optional f32 values representing polygon coordinates. Individual coordinates can be null. |
| `text` | `LargeString` | The note text content. |

Diesel annotations:
- `#[diesel(table_name = post_note)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(belongs_to(Post))]`

Relationships:
- Belongs to `Post` via `post_id`.

### `PostScore`

Maps to table: `post_score`

Derives: `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Part of composite primary key. Foreign key to `post.id`. |
| `user_id` | `i64` | Part of composite primary key. Foreign key to `user.id`. |
| `score` | `Score` | The score value (`Like` or `Dislike`). |
| `time` | `DateTime` | Timestamp when the score was recorded. |

Diesel annotations:
- `#[diesel(table_name = post_score)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(post_id, user_id))]`
- `#[diesel(belongs_to(Post), belongs_to(User))]`

Relationships:
- Belongs to `Post` via `post_id`.
- Belongs to `User` via `user_id`.

Composite primary key: `(post_id, user_id)`.

This struct implements `Insertable` directly.

### `CompressedSignature`

Derives: `Debug`, `AsExpression`, `FromSqlRow`

**Not a table struct.** A newtype wrapper around `[i64; COMPRESSED_SIGNATURE_LEN]` that represents a perceptual signature for image duplicate detection.

Diesel annotations:
- `#[diesel(sql_type = Array<Nullable<BigInt>>)]` — Stored as a PostgreSQL array of nullable `BIGINT` values.

Custom trait implementations:
- `Deref<Target = [i64; COMPRESSED_SIGNATURE_LEN]>` — Allows dereferencing to the inner array.
- `From<[i64; COMPRESSED_SIGNATURE_LEN]>` — Allows construction from a fixed-size array.
- `ToSql<Array<Nullable<BigInt>>, Pg>` — Serializes as a PostgreSQL array.
- `FromSql<Array<Nullable<BigInt>>, Pg>` — Deserializes from a PostgreSQL array using the internal `deserialize_array` function.

### `SignatureIndexes`

Derives: `Debug`, `AsExpression`, `FromSqlRow`

**Not a table struct.** A newtype wrapper around `[i32; NUM_WORDS]` that represents indexed words from the perceptual hash.

Diesel annotations:
- `#[diesel(sql_type = Array<Nullable<Integer>>)]` — Stored as a PostgreSQL array of nullable `INTEGER` values.

Custom trait implementations:
- `From<[i32; NUM_WORDS]>` — Allows construction from a fixed-size array.
- `ToSql<Array<Nullable<Integer>>, Pg>` — Serializes as a PostgreSQL array.
- `FromSql<Array<Nullable<Integer>>, Pg>` — Deserializes from a PostgreSQL array.

### `NewPostSignature` (Insertable)

Maps to table: `post_signature`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Foreign key to `post.id`. |
| `signature` | `CompressedSignature` | The compressed perceptual signature. |
| `words` | `SignatureIndexes` | The indexed words from the perceptual hash for similarity lookup. |

Diesel annotations:
- `#[diesel(table_name = post_signature)]`
- `#[diesel(check_for_backend(Pg))]`

### `PostSignature`

Maps to table: `post_signature`

Derives: `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Primary key. Foreign key to `post.id`. |
| `signature` | `CompressedSignature` | The compressed perceptual signature for duplicate detection. |

Diesel annotations:
- `#[diesel(table_name = post_signature)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(post_id))]`
- `#[diesel(belongs_to(Post))]`

Relationships:
- Belongs to `Post` via `post_id`.

**Note:** The `words` field from the database table is not present in this struct. It is only used in `NewPostSignature` for inserting and in the `find_similar_candidates` query via `overlaps_with`.

Custom methods:
- `find_similar_candidates(conn, words) -> QueryResult<Vec<Self>>` — Finds posts with similar perceptual signatures. Disables sequential scans temporarily to force PostgreSQL to use an index scan on the `words` column. Returns candidate posts that share at least one word index.

## Internal Helper

### `deserialize_array`

A generic function `deserialize_array<T, const N, A>(value: PgValue) -> deserialize::Result<[T; N]>` that deserializes a PostgreSQL array into a fixed-size Rust array. Used by `CompressedSignature` and `SignatureIndexes`. Validates that the array is one-dimensional, has no nulls, and has exactly `N` elements.

## Explicit Joinable Declarations

- `diesel::joinable!(post_relation -> post (parent_id))` — Allows Diesel to join `post_relation` to `post` using the `parent_id` foreign key, even though `belongs_to` uses `parent_id` for the relationship.

## Type References

- `Checksum` — A hash checksum type from `crate::content::hash`.
- `Md5Checksum` — An MD5-specific checksum type from `crate::content::hash`.
- `PostSafety`, `PostType`, `MimeType`, `PostFlags`, `Score` — Defined in `enums.rs`.
- `DateTime` — A timestamp type from `crate::time`.
- `LargeString` — A string type from `crate::string`.
- `COMPRESSED_SIGNATURE_LEN` — Compile-time constant from `crate::content::signature`.
- `NUM_WORDS` — Compile-time constant from `crate::content::signature`.
