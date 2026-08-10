# User Admin Operations (`admin/user.rs`)

## Summary

This module provides the password reset functionality for the admin CLI. It prompts for a username and new password, validates the user exists, enforces password rules, hashes the password with Argon2id, updates the database, and revokes all of the user's API tokens.

## Imports

- `admin::input::{self, UserEditor}`: User input editor for the admin CLI.
- `api`: API module for validation utilities.
- `app::AppState`: Application state (config, connection pool).
- `auth::password`: Password hashing and verification.
- `config::RegexType`: Regex type for password validation rules.
- `schema::{user, user_token}`: User and user token table schemas.
- `argon2::password_hash::SaltString`: Argon2 salt generation.
- `argon2::password_hash::rand_core::OsRng`: Cryptographically secure RNG.
- `diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl}`: Diesel ORM methods.

## Function: `reset_password(state, editor)`

```rust
pub fn reset_password(state: &AppState, editor: &mut UserEditor)
```

Public entry point. Resets passwords for one or more users.

**Algorithm:**
1. Enter `input::user_input_loop` with the `UserEditor`.
2. The loop closure contains the full password reset flow. The user can type `done` to exit the loop after resetting passwords for multiple users.

### Per-iteration password reset flow:

**Step 1: Prompt for username.**
- Prints: `"Please enter the username of the user you would like to reset a password for. Enter \"done\" when finished."`
- Calls `input::read("Username: ", editor)`. Returns the username. The `done`/`exit` handling is done by `input::read`.

**Step 2: Verify user exists and get the user id.**
- Gets a blocking database connection from `state.connection_pool.get_blocking()`.
- Maps the pool error to a string-based error.
- Runs a query that selects `user::id` from the `user` table where `user::name` equals the username, using `.first::<i64>(&mut conn).optional()`.
  - `Ok(Some(user_id))`: User exists; keep the user id for the later steps.
  - `Ok(None)`: Return `Err("No user with this username exists".into())`.
  - `Err(err)`: Return error with the database error message.

**Step 3: Prompt for new password.**
- Calls `input::read("New password: ", editor)`.

**Step 4: Validate password against rules.**
- Calls `api::verify_matches_regex(&state.config, &password, RegexType::Password)`.
- This checks that the password meets the configured regex requirements (minimum length, character classes, etc.).

**Step 5: Generate salt and hash password.**
- Generates a random salt with `SaltString::generate(&mut OsRng)`.
- Hashes the password: `password::hash_password(&state.config, &password, &salt)`.
  - Maps error to a string: `"Could not hash password for reason: {err}"`.

**Step 6: Update database.**
- Updates the `user` table by primary key: `user::table.find(user_id)`.
- Sets two fields:
  - `user::password_hash`: The Argon2id hash string.
  - `user::password_salt`: The salt string (as `&str`).
- Executes the update.
  - Maps error: `"Could not update password for reason: {err}"`.

**Step 7: Revoke all API tokens.**
- Deletes every row in the `user_token` table where `user_id` equals the user id.
- This revokes all of the user's API tokens so that a stolen token does not survive a password reset.
- Maps error: `"Could not revoke API tokens for reason: {err}"`.

**Step 8: Confirm.**
- Prints `"Password reset successful.\n"`.
- Returns `Ok(())`.

The loop returns to step 1. The user can enter `done` to stop or a new username to continue.
