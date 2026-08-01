# db.rs — Database Connection Pool and Migrations

## Purpose

This file defines the asynchronous database connection pool, embedded migrations, connection customization, and migration logic.

## Type Aliases

| Alias | Definition |
|-------|------------|
| `Connection` | `PooledConnection<ConnectionManager<PgConnection>>` |
| `AsyncConnectionResult<'a>` | `Result<AsyncConnection<'a>, PoolError>` |

## Struct: `AsyncConnection<'a>`

```rust
pub struct AsyncConnection<'a> {
    conn: Connection,
    permit: SemaphorePermit<'a>,
}
```

A database connection paired with a semaphore permit. The permit ensures the number of concurrent connections does not exceed the pool size.

### Trait: `AsMut<PgConnection>`

Implemented to allow `&mut self` to be used as `&mut PgConnection`. Delegates to `self.conn`.

## Struct: `AsyncConnectionPool`

```rust
pub struct AsyncConnectionPool {
    pool: Pool<ConnectionManager<PgConnection>>,
    semaphore: Semaphore,
}
```

An async-safe connection pool. Combines an r2d2 blocking connection pool with a tokio semaphore to limit concurrent access.

| Field | Type | Purpose |
|-------|------|---------|
| `pool` | `Pool<ConnectionManager<PgConnection>>` | Blocking r2d2 connection pool |
| `semaphore` | `Semaphore` | Async semaphore limiting concurrent connections |

### Method: `AsyncConnectionPool::get(&self) -> AsyncConnectionResult<'_>`

Acquires a semaphore permit, then gets a connection from the pool. Returns an `AsyncConnection` wrapping both. The semaphore panics if closed (should never happen).

### Method: `AsyncConnectionPool::get_blocking(&self) -> Result<Connection, PoolError>`

Gets a connection directly from the pool without the semaphore. Used for startup/initialization before the async runtime is fully active.

### Method: `AsyncConnectionPool::transaction<T, E, F>(&self, f: F) -> ApiResult<T>`

Runs a database transaction within a `spawn_blocking` task.

**Type parameters:**
- `T: Send + 'static` — The return type of the closure
- `E: From<DieselError> + Send + 'static` — The error type from the closure
- `F: FnOnce(&mut Connection) -> Result<T, E> + Send + 'static` — The closure to run in a transaction

**Process:**
1. Acquires a semaphore permit.
2. Gets a connection from the pool.
3. Spawns a blocking task that runs `conn.transaction(f)`.
4. Awaits the join handle, then maps the error.

## Function: `create_connection_pool() -> AsyncConnectionPool`

Creates the production connection pool. Panics if called in a test build.

**Pool sizing:** `max(num_tokio_threads, num_rayon_threads) + 1`

**Pool settings:**
- `max_lifetime`: `None` (no limit)
- `idle_timeout`: `None` (no limit)
- `test_on_check_out`: `true` (validates connections on checkout)
- `connection_customizer`: `ConnectionInitializer` (enables `auto_explain` if configured)

## Function: `create_test_connection_pool(test_url: String) -> AsyncConnectionPool`

Available only in test builds. Creates a pool with:
- `max_lifetime`: `None`
- `idle_timeout`: `None`
- `test_on_check_out`: `true`
- Semaphore sized to match `pool.max_size()`

## Function: `run_database_migrations(connection_pool: &AsyncConnectionPool) -> Result<RangeInclusive<i32>, Box<dyn Error + Send + Sync>>`

Runs embedded diesel migrations.

1. Gets a blocking connection and checks for pending migrations.
2. If none are pending, returns an empty range `1..=0`.
3. Otherwise, computes the range of migration numbers from the first and last pending migration.
4. Logs "Running pending migrations..." and calls `conn.run_pending_migrations(MIGRATIONS)`.
5. Returns the migration range.

## Function: `run_server_migrations(state: &AppState, migration_range: RangeInclusive<i32>) -> Result<(), Box<dyn Error + Send + Sync>>`

Runs post-migration server-side logic based on which migrations were applied:

| Condition | Action |
|-----------|--------|
| Migration 1 in range | Sets the initial `signature_version` in `database_statistics` |
| Migration 12 in range | Calls `admin::database::reset_filenames_impl` (BIGINT primary key migration) |
| Migration 13 in range | Calls `admin::database::reset_thumbnail_sizes_impl` (statistics system migration) |
| Migration 21 in range | Calls `admin::database::reset_filenames_impl` and `admin::post::recompute_checksums` (new storage structure + checksum fix) |

## Function: `check_signature_version(conn: &mut PgConnection) -> QueryResult<()>`

Checks whether the database signature version matches the compiled-in `SIGNATURE_VERSION`.

1. Reads `signature_version` from `database_statistics`.
2. If it matches, returns immediately.
3. If it does not match, logs an error message instructing the admin to run the `RecomputeSignatures` task via the admin CLI.
4. Blocks in a loop, sleeping 1 second at a time, until the version matches.

The purpose is to require the admin to recompute post signatures before the server can operate.

## Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `MIGRATIONS` | `embed_migrations!()` | Embedded diesel migrations |
| `SEMAPHORE_PANIC_MESSAGE` | `"Semaphore should never close"` | Message for semaphore panic |

## Struct: `ConnectionInitializer`

```rust
struct ConnectionInitializer {}
```

Implements `CustomizeConnection<PgConnection, diesel::r2d2::Error>`.

### Method: `ConnectionInitializer::on_acquire(&self, conn: &mut PgConnection) -> Result<(), diesel::r2d2::Error>`

Called when a connection is acquired from the pool. If `config.auto_explain` is `true`, executes these SQL statements on the session:
- `LOAD 'auto_explain';`
- `SET SESSION auto_explain.log_min_duration = 500;`
- `SET SESSION auto_explain.log_parameter_max_length = 0;`
- `SET SESSION auto_explain.log_analyze = TRUE;`

### Method: `ConnectionInitializer::on_release(&self, _conn: PgConnection)`

No-op on connection release.
