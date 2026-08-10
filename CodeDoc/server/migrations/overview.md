# Database Migrations

## Migration List and Schema Changes

### 000 — Diesel Initial Setup
Creates two helper functions:
- `diesel_manage_last_edit_time(_tbl)`: Creates a trigger on a table that calls `diesel_set_last_edit_time`.
- `diesel_set_last_edit_time()`: Trigger function that sets `last_edit_time` to CURRENT_TIMESTAMP when a row is modified, unless `last_edit_time` was explicitly changed.

### 001 — Create Users
**Creates tables**:
- `user`: id (serial, PK), name (CITEXT, UNIQUE), password_hash (VARCHAR 128), password_salt (VARCHAR 32), email (CITEXT, nullable), rank (SMALLINT), avatar_style (SMALLINT), creation_time, last_login_time, last_edit_time. Registers last_edit_time trigger.
- `user_token`: user_id (INTEGER, PK, FK to user ON DELETE CASCADE, ONE-TO-ONE), token (UUID, UNIQUE), note (VARCHAR 128, default ''), enabled (BOOLEAN), expiration_time (nullable), creation_time, last_edit_time, last_usage_time. Registers last_edit_time trigger.

**Enables**: `citext` extension (case-insensitive text type).

### 002 — Create Tags
**Creates tables**:
- `tag_category`: id (serial, PK), order (INTEGER), name (CITEXT, UNIQUE), color (VARCHAR 32), last_edit_time. Registers trigger. Inserts default category (id=0, order=0, name='default', color='default').
- `tag`: id (serial, PK), category_id (INTEGER, FK to tag_category ON DELETE SET DEFAULT, default 0), description (TEXT, default ''), creation_time, last_edit_time. Registers trigger.
- `tag_name`: tag_id (INTEGER, FK to tag ON DELETE CASCADE), order (INTEGER), name (CITEXT, UNIQUE). PK is (tag_id, order).
- `tag_implication`: parent_id (INTEGER, FK to tag ON DELETE CASCADE), child_id (INTEGER, FK to tag ON DELETE CASCADE). PK is (parent_id, child_id).
- `tag_suggestion`: parent_id (INTEGER), child_id (INTEGER). Same structure as tag_implication.

### 003 — Create Posts
**Creates tables**:
- `post`: id (serial, PK), user_id (INTEGER, FK to user ON DELETE SET NULL), file_size (BIGINT), width (INTEGER), height (INTEGER), safety (SMALLINT), type (SMALLINT), mime_type (SMALLINT), checksum (VARCHAR 64, UNIQUE), checksum_md5 (VARCHAR 32, nullable), flags (SMALLINT), source (VARCHAR 2048), creation_time, last_edit_time. Registers trigger.
- `post_relation`: parent_id (INTEGER, FK to post ON DELETE CASCADE), child_id (INTEGER, FK to post ON DELETE CASCADE). PK is (parent_id, child_id).
- `post_tag`: post_id (INTEGER, FK to post ON DELETE CASCADE), tag_id (INTEGER, FK to tag ON DELETE CASCADE). PK is (post_id, tag_id).
- `post_favorite`: post_id (INTEGER, FK to post ON DELETE CASCADE), user_id (INTEGER, FK to user ON DELETE CASCADE), time. PK is (post_id, user_id).
- `post_feature`: id (serial, PK), post_id (INTEGER, FK to post ON DELETE CASCADE), user_id (INTEGER, FK to user ON DELETE CASCADE), time.
- `post_note`: id (serial, PK), post_id (INTEGER, FK to post ON DELETE CASCADE), polygon (REAL[][2]), text (TEXT).
- `post_score`: post_id (INTEGER, FK to post ON DELETE CASCADE), user_id (INTEGER, FK to user ON DELETE CASCADE), score (SMALLINT), time. PK is (post_id, user_id).
- `post_signature`: post_id (INTEGER, PK, FK to post ON DELETE CASCADE), signature (BYTEA), words (INTEGER[]).

### 004 — Create Comments
**Creates tables**:
- `comment`: id (serial, PK), user_id (INTEGER, FK to user ON DELETE CASCADE), post_id (INTEGER, FK to post ON DELETE CASCADE), text (TEXT, default ''), creation_time, last_edit_time. Registers trigger.
- `comment_score`: comment_id (INTEGER, FK to comment ON DELETE CASCADE), user_id (INTEGER, FK to user ON DELETE CASCADE), score (SMALLINT), time. PK is (comment_id, user_id).

### 005 — Create Pools
**Creates tables**:
- `pool_category`: id (serial, PK), name (CITEXT, UNIQUE), color (VARCHAR 32), last_edit_time. Registers trigger. Inserts default category (id=0, name='default', color='default').
- `pool`: id (serial, PK), category_id (INTEGER, FK to pool_category ON DELETE SET DEFAULT, default 0), description (TEXT, default ''), creation_time, last_edit_time. Registers trigger.
- `pool_name`: pool_id (INTEGER, FK to pool ON DELETE CASCADE), order (INTEGER), name (CITEXT, UNIQUE). PK is (pool_id, order).
- `pool_post`: pool_id (INTEGER, FK to pool ON DELETE CASCADE), post_id (INTEGER, FK to post ON DELETE CASCADE), order (INTEGER). PK is (pool_id, post_id).

### 006 — Create Indices
**Enables**: `intarray` extension for GIN index on integer arrays.

**Creates indices**:
- GIN on `post_signature.words` with `gin__int_ops` for fast array overlap queries.
- BTREE indices on:
  - `comment.post_id`, `comment.user_id`
  - `comment_score.user_id`
  - `post.user_id`
  - `post_favorite.user_id`
  - `post_feature.post_id`, `post_feature.user_id`
  - `post_note.post_id`
  - `post_relation.child_id`
  - `post_score.user_id`
  - `post_tag.tag_id`
  - `tag.category_id`
  - `tag_implication.child_id`
  - `tag_name.order`
  - `tag_suggestion.child_id`

### 007 — Make Emails Unique
Adds UNIQUE constraint on `user.email`.

### 008 — Compress Post Signatures
Drops the old `signature` column (BYTEA), adds new `signature` column (BIGINT[]), initializes to empty array, sets NOT NULL.

### 009 — Nullable Comment user_id
Makes `comment.user_id` nullable. Drops old FK constraint. Adds new FK constraint with ON DELETE SET NULL (instead of CASCADE). This allows anonymous comments to persist if a user is deleted.

### 010 — Drop Last Edit Time Trigger
Drops the `diesel_set_last_edit_time` and `diesel_manage_last_edit_time` functions (and all triggers that depend on them) with CASCADE. From this point, `last_edit_time` must be managed in application code.

### 011 — Allow Multiple User Tokens
Drops the old `user_token` table (which had a one-to-one FK to user). Recreates it with:
- `id` (UUID) as PRIMARY KEY (instead of `user_id`).
- `user_id` (INTEGER, FK to user ON DELETE CASCADE) as a regular foreign key (allowing multiple tokens per user).
- Same columns: note, enabled, expiration_time, creation_time, last_edit_time, last_usage_time.

### 012 — Use BIGINT Primary Keys
Converts ALL integer primary keys and foreign keys to BIGINT across all tables: user, user_token, tag_category, tag, tag_name, tag_implication, tag_suggestion, post, post_relation, post_tag, post_favorite, post_feature, post_note, post_score, post_signature, comment, comment_score, pool_category, pool, pool_name, pool_post (including pool_post.order).

### 013 — Database Statistics
**Creates tables**:
- `database_statistics`: id (BOOLEAN, PK, constrained to true — ensures single row), disk_usage, comment_count, pool_count, post_count, tag_count, user_count. Populated from existing data.
- `comment_statistics`: comment_id (BIGINT, PK, FK to comment), score. Populated from comment_score sums.
- `pool_category_statistics`: category_id (BIGINT, PK, FK to pool_category), usage_count. Populated from pool counts.
- `pool_statistics`: pool_id (BIGINT, PK, FK to pool), post_count. Populated from pool_post counts.
- `post_statistics`: post_id (BIGINT, PK, FK to post), tag_count, pool_count, note_count, comment_count, relation_count, score, favorite_count, feature_count, last_comment_time, last_favorite_time, last_feature_time. Populated from comprehensive joins.
- `tag_category_statistics`: category_id (BIGINT, PK, FK to tag_category), usage_count.
- `tag_statistics`: tag_id (BIGINT, PK, FK to tag), usage_count, implication_count, suggestion_count.
- `user_statistics`: user_id (BIGINT, PK, FK to user), comment_count, favorite_count, upload_count.

**Adds columns**: `post.generated_thumbnail_size` (BIGINT), `post.custom_thumbnail_size` (BIGINT), `user.custom_avatar_size` (BIGINT).

**Creates database triggers** for automatic statistics maintenance:
- **comment_update_trigger**: On INSERT/DELETE on comment: updates database_statistics.comment_count, post_statistics.comment_count + last_comment_time, user_statistics.comment_count.
- **comment_score_update_trigger**: On INSERT/UPDATE/DELETE on comment_score: updates comment_statistics.score by delta.
- **pool_category_update_trigger**: On INSERT on pool_category: inserts default row into statistics. On DELETE: migrates usage count to default category (id=0).
- **pool_update_trigger**: On INSERT/DELETE on pool: updates database_statistics.pool_count and pool_category_statistics.usage_count. On UPDATE of category_id: migrates usage count between categories.
- **pool_post_update_trigger**: On INSERT/DELETE on pool_post: updates pool_statistics.post_count and post_statistics.pool_count.
- **post_update_trigger**: On INSERT/UPDATE/DELETE on post: updates database_statistics.post_count and disk_usage, user_statistics.upload_count.
- **post_relation_update_trigger**: On INSERT/DELETE on post_relation: updates post_statistics.relation_count.
- **post_tag_update_trigger**: On INSERT/DELETE on post_tag: updates post_statistics.tag_count and tag_statistics.usage_count.
- **post_favorite_update_trigger**: On INSERT/DELETE on post_favorite: updates post_statistics.favorite_count + last_favorite_time, user_statistics.favorite_count.
- **post_feature_update_trigger**: On INSERT/DELETE on post_feature: updates post_statistics.feature_count + last_feature_time.
- **post_note_update_trigger**: On INSERT/DELETE on post_note: updates post_statistics.note_count.
- **post_score_update_trigger**: On INSERT/UPDATE/DELETE on post_score: updates post_statistics.score by delta.
- **tag_category_update_trigger**: Similar to pool_category triggers.
- **tag_update_trigger**: Similar to pool triggers.
- **tag_implication_update_trigger**: On INSERT/DELETE on tag_implication: updates tag_statistics.implication_count.
- **tag_suggestion_update_trigger**: On INSERT/DELETE on tag_suggestion: updates tag_statistics.suggestion_count.
- **user_update_trigger**: On INSERT/UPDATE/DELETE on user: updates database_statistics.user_count and disk_usage.

### 014 — Add Signature Version
Adds `database_statistics.signature_version` (INTEGER, default 0). Allows tracking when post signatures need reindexing.

### 015 — Non-nullable Source
Temporarily disables post triggers. Updates all NULL source values to empty string. Enables triggers. Alters `post.source` to TEXT and NOT NULL.

### 016 — Compress Checksums
Temporarily disables post triggers. Updates NULL checksum_md5 to empty string. Enables triggers. Creates helper function `try_decode_base64` that decodes base64 or falls back to UTF-8. Converts `post.checksum` from VARCHAR to BYTEA using base64 decode. Converts `post.checksum_md5` from VARCHAR to BYTEA and sets NOT NULL. Drops helper function.

### 017 — Name Search Indexes
Creates pattern-matching BTREE indices using `lower()` with `text_pattern_ops` on:
- `user.name` — for case-insensitive user search.
- `tag_name.name` — for case-insensitive tag name search.
- `pool_name.name` — for case-insensitive pool name search.

Creates composite index on `post_tag(tag_id, post_id)` to speed up tag-filtered post searches (avoids sequential scan).

### 018 — Post Descriptions
Adds `post.description` (TEXT, default '').

### 019 — Create Snapshots
**Creates table**:
- `snapshot`: id (BIGINT, serial, PK), user_id (BIGINT, FK to user ON DELETE SET NULL), operation (SMALLINT), resource_type (SMALLINT), resource_id (TEXT), data (JSONB), creation_time.

### 020 — Add User Search Seed
Adds `user.search_seed` (REAL, default 0.0). Used for randomized user ordering in search results.

### 021 — Remove Redundant Index
Drops `idx_post_tag_tag_id` since a covering index on `(tag_id, post_id)` already exists.

### 022 — Add Launch Date and Document Type
Adds `post.launch_date` (TIMESTAMPTZ, nullable).

### 023 — Add Media Created Date
Adds `post.media_created_date` (TIMESTAMPTZ, nullable).

### 024 — Add Pool Launch Date
Adds `pool.launch_date` (TIMESTAMP WITH TIME ZONE, nullable).

### 025 — Create Custom Field
**Creates table**:
- `custom_field`: id (BIGSERIAL, PK), name (CITEXT, UNIQUE), creation_time (TIMESTAMPTZ, default now()).

### 026 — Create Post Custom Field
**Creates table**:
- `post_custom_field`: post_id (BIGINT, FK to post ON DELETE CASCADE), custom_field_id (BIGINT, FK to custom_field ON DELETE CASCADE), value (BIGINT). PK is (post_id, custom_field_id).

### 027 — Add Custom Field Description and Color
Adds `custom_field.description` (TEXT, default '') and `custom_field.color` (TEXT, default '').

### 028 — Add Custom Field Last Edit Time
Adds `custom_field.last_edit_time` (TIMESTAMPTZ, default NOW()).

### 029 — Add Password Reset Token
Adds `user.password_reset_token` (TEXT, nullable) and `user.password_reset_expires` (TIMESTAMPTZ, nullable).

### 030 — Hash User Tokens
Stores only a BLAKE3 hash (hex) and an AES-256-GCM ciphertext of each user token; the plaintext token value is never persisted.

**Deletes** all existing rows from `user_token`: legacy rows stored the raw token in the `id` column, so their hashes cannot be recomputed and the tokens are revoked.

**Adds columns**:
- `token_hash` (TEXT, NOT NULL, default `''`).
- `token_secret` (BYTEA, NOT NULL, default `'\x'`).

**Creates index**: unique partial index `user_token_token_hash` on `token_hash` WHERE `token_hash` <> `''`. Real tokens get a unique 64-character hex hash; empty-string hashes (rows without a token value, for example legacy inserts) may repeat.

**Down migration**: drops the index, then drops the `token_secret` and `token_hash` columns.
