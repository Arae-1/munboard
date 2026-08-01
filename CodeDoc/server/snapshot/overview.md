# Snapshot System

## Individual Files

- [mod.md](mod.md)
- [comment.md](comment.md)
- [custom_field.md](custom_field.md)
- [pool.md](pool.md)
- [pool_category.md](pool_category.md)
- [post.md](post.md)
- [tag.md](tag.md)
- [tag_category.md](tag_category.md)
- [user.md](user.md)
- [user_token.md](user_token.md)

## Purpose

The snapshot system records every change to database resources. Each snapshot stores the user who made the change, the operation (created, modified, deleted, merged), the resource type and ID, and the relevant data. This creates a full audit trail.

## How Changes Are Tracked

The module provides one snapshot function per resource type. Each resource type has these operations:

- **Creation** — A full copy of the resource data is stored as the snapshot.
- **Modification** — A diff between old and new data is computed and stored as the snapshot.
- **Deletion** — A full copy of the resource data is stored as the snapshot.
- **Merge** — The target resource ID and type are stored.

For modification snapshots, the old and new data are compared. A snapshot is only inserted if there is an actual difference.

## Diff System

The file `mod.rs` contains three diff functions:

### `value_diff(old, new)`
Computes a recursive diff between two JSON values. Handles these cases:
- **Primitive change**: An old and new primitive value differ.
- **Property changed type**: A value changes from one JSON type to another.
- **List change**: Items were added or removed from an array.
- **Object change**: Properties were added, deleted, or modified within an object.

### `array_diff(old, new)`
Computes a set-based diff between two arrays. It finds removed items (present in old but not new) and added items (present in new but not old). Duplicates within the arrays are ignored.

### `object_diff(old, new)`
Computes a property-level diff between two JSON objects. For each property:
- If the property exists only in old, records as "deleted property".
- If the property exists only in new, records as "added property".
- If the property exists in both, recursively diffs the values.

### `set_default_snapshot`
A special function for tag and pool category "default" status changes. It creates two modification snapshots:
1. One for the old default resource showing `{"default": false}`.
2. One for the new default resource showing `{"default": true}`.

## Per-Resource Snapshot Data

### Comment
Stored fields: `text`, `post_id`. The resource ID is the comment's numeric ID.

### Custom Field
Stored fields: `name`, `description`, `color`. The resource ID is the custom field's numeric ID.

### Pool
Stored fields: `description`, `launch_date`, `category`, `names` (sorted), `posts` (sorted). The resource ID is the pool's numeric ID.

For pool creation and modification, `SnapshotData::retrieve` performs extra database queries to collect:
- The pool category name by joining `pool_category`.
- All names from `pool_name`.
- All post IDs from `pool_post`.

Before comparison or storage, names and posts are sorted to ensure consistent ordering.

Merge snapshots store a JSON array of `[ResourceType::Pool, merge_to_pool_id]`.

### Pool Category
Stored fields: `name`, `color`, `default` (derived from `id == 0`). The resource ID is the category name.

When the default pool category changes, `set_default_snapshot` from `mod.rs` is called.

### Post
Stored fields: `safety`, `checksum` (hex-encoded), `flags`, `source`, `description`, `launch_date`, `media_created_date`, `tags` (sorted), `relations` (sorted), `notes` (sorted by note ID), `featured`, `custom_fields` (sorted by name).

`SnapshotData::retrieve` collects:
- Primary tag names via `post_tag` joined with `tag_name`.
- Relations via `post_relation`.
- Notes via `post_note`.
- Featured status by checking `post_feature` for the latest feature.
- Custom field values via `post_custom_field` joined with `custom_field`.

Feature snapshots are special: when the featured post changes, the old featured post gets a snapshot with `{"featured": false}` and the new one gets `{"featured": true}`.

Merge snapshots store `[ResourceType::Post, merge_to_post_id]`.

### Tag
Stored fields: `description`, `category` (category name), `names` (sorted), `implications` (sorted), `suggestions` (sorted).

`SnapshotData::retrieve` collects:
- The category name from `tag_category`.
- All names from `tag_name`.
- All implications from `tag_implication` joined with `tag_name` (primary names only).
- All suggestions from `tag_suggestion` joined with `tag_name` (primary names only).

For `new_name_snapshots`, a batch of new tags can have creation snapshots inserted in bulk. Each new tag gets a snapshot with the default category name and an empty description.

The resource ID for tag snapshots is the first (primary) tag name.

Merge snapshots store `[ResourceType::Tag, merge_to_tag_name]`.

### Tag Category
Stored fields: `name`, `color`, `default` (derived from `id == 0`). The resource ID is the category name.

Same default-change behavior as pool category using `set_default_snapshot`.

### User
Stored fields: `name`, `rank`, `avatar_style`. The resource ID is the user name.

### User Token
Stored fields: `note`, `enabled`, `expiration_time`. The resource ID is the token's UUID string.
