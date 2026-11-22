# Sub-pools Branch Assessment

## Decision

Do not save `feature/sub-pools` as the basis for the next implementation.

The branch changes 81 files. It adds 2,512 lines. It changes the database, API, search rules, snapshots, permissions, client views, and help text in one stack of six commits. The branch has no merge from `master`. Start the next design from `master` after the cleanup below.

Keep commit `51e59824` separately. It fixes pool-post reorder detection and pool-chip color variables. It does not depend on sub-pools.

## Why a Restart Is Safer

- The branch changes the meaning of the existing `pool:` search token. It makes every existing pool search include descendant pools. This is a system-wide behavior change.
- The branch adds `pool-direct:` as a second search rule. The design does not first define the required search rules, result order, or backward-compatibility policy.
- The branch includes complex edge rewriting during pool merge in `server/src/update/pool.rs`. It has no merge-specific API test.
- The cycle check reads the full edge graph in application code. It does not protect against two concurrent edits that each pass the check and together create a cycle.
- The branch has no database migration test or rollback test.
- The API test covers one direct child and one cycle. It does not cover merge behavior, deletion behavior, a shared child, deep nesting, or concurrent edits.
- The 623-line browser test covers only the picker. It does not test the summary tree, post sidebar, search behavior, or permissions.

## Verification

- `cargo test` passed 128 tests.
- `node client/sub_pool_picker_test.cjs` passed all 10 picker scenarios.
- `cargo clippy` completed with 36 warnings and no errors.

These checks verify the current covered behavior. They do not test merge edge rewriting, concurrent edits, deep nesting, or the full client behavior.

## Database Cleanup

The cleanup applies only if a server built from `feature/sub-pools` ran against the development database. That server applies migration `033_add_pool_subpools` at startup.

1. Stop every server that uses the sub-pools branch.
2. Back up the database.
3. Check the migration record:

   ```sql
   SELECT version, run_on
   FROM __diesel_schema_migrations
   WHERE version = '033_add_pool_subpools';
   ```

4. If the record exists, run this SQL in one transaction:

   ```sql
   BEGIN;
   DROP TRIGGER IF EXISTS pool_subpool_update_trigger ON pool_subpool;
   DROP FUNCTION IF EXISTS update_pool_subpool_statistics();
   ALTER TABLE pool_statistics DROP COLUMN IF EXISTS subpool_count;
   DROP TABLE IF EXISTS pool_subpool;
   DELETE FROM __diesel_schema_migrations
   WHERE version = '033_add_pool_subpools';
   COMMIT;
   ```

5. Verify that no sub-pools objects remain:

   ```sql
   SELECT to_regclass('public.pool_subpool');
   SELECT column_name
   FROM information_schema.columns
   WHERE table_schema = 'public'
     AND table_name = 'pool_statistics'
     AND column_name = 'subpool_count';
   SELECT version
   FROM __diesel_schema_migrations
   WHERE version = '033_add_pool_subpools';
   ```

Each query in step 5 must return no object or no row.

Do not delete pool rows, pool-post rows, snapshots, or client data. The migration only adds the table, index, trigger function, trigger, and statistics column above.

## Source Cleanup

1. Preserve `51e59824` in a separate branch or cherry-pick it onto `master`.
2. Do not merge the six sub-pools commits.
3. Create the next sub-pools branch from current `master`.
4. Define the data model and search compatibility before the new migration.
5. Add migration, rollback, merge, deletion, nested-search, permission, and client tests before the UI work.

If a deployed `config.toml` has a manually added `pool_edit_sub_pool` privilege, remove that line before a server from `master` starts.
