# Post Custom Field Model

## Database Tables

- `post_custom_field` — Junction table that assigns a custom field with a value to a post.

## Structs

### `PostCustomField`

Maps to table: `post_custom_field`

Derives: `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `post_id` | `i64` | Part of composite primary key. Foreign key to `post.id`. |
| `custom_field_id` | `i64` | Part of composite primary key. Foreign key to `custom_field.id`. |
| `value` | `i64` | The value assigned to the custom field for this post. Must be a non-negative integer (0 is allowed; negative values are rejected by the API and by a DB CHECK constraint). |

Diesel annotations:
- `#[diesel(table_name = post_custom_field)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(post_id, custom_field_id))]`
- `#[diesel(belongs_to(CustomField), belongs_to(Post))]`

Relationships:
- Belongs to `CustomField` via `custom_field_id`.
- Belongs to `Post` via `post_id`.

Composite primary key: `(post_id, custom_field_id)`.

This struct implements `Insertable` directly (no separate `New*` struct) because all its fields are required on insert.

## Type References

No custom enum or special types are used. All fields are `i64`.
