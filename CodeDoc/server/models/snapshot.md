# Snapshot Model

## Database Tables

- `snapshot` — Stores an audit trail of changes made to any resource in the system.

## Structs

### `NewSnapshot` (Insertable)

Maps to table: `snapshot`

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Option<i64>` | Foreign key to `user.id`. Null if the action was performed by the system. |
| `operation` | `ResourceOperation` | The type of change (Created, Modified, Merged, Deleted). |
| `resource_type` | `ResourceType` | The type of resource that was changed. |
| `resource_id` | `SmallString` | The identifier of the changed resource, stored as a string. |
| `data` | `Value` | A JSON value containing the full serialized state of the resource at the time of the change. Uses `serde_json::Value`. |

Diesel annotations:
- `#[diesel(table_name = snapshot)]`
- `#[diesel(check_for_backend(Pg))]`

Custom methods:
- `insert(self, conn) -> QueryResult<()>` — Inserts the snapshot into the `snapshot` table and discards the number of rows result.

### `Snapshot`

Maps to table: `snapshot`

Derives: `Serialize`, `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `user_id` | `Option<i64>` | Foreign key to `user.id`. Null for system actions. |
| `operation` | `ResourceOperation` | The type of change. |
| `resource_type` | `ResourceType` | The type of resource changed. |
| `resource_id` | `SmallString` | The identifier of the changed resource. |
| `data` | `Value` | JSON data containing the resource state. |
| `creation_time` | `DateTime` | Timestamp when the snapshot was recorded. |

Diesel annotations:
- `#[diesel(table_name = snapshot)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(belongs_to(User))]`

Serde annotations:
- `#[derive(Serialize)]` — The struct can be serialized to JSON for API responses. There is no `Deserialize` derive.

Relationships:
- Belongs to `User` via `user_id`.

## Type References

- `ResourceOperation` — Defined in `enums.rs`. Stored as `SmallInt`: Created (0), Modified (1), Merged (2), Deleted (3).
- `ResourceType` — Defined in `enums.rs`. Stored as `SmallInt`: identifies the type of resource.
- `Value` — `serde_json::Value`. Stored as a PostgreSQL JSONB or JSON column.
- `SmallString` — A bounded string type from `crate::string`.
- `DateTime` — A timestamp type from `crate::time`.
