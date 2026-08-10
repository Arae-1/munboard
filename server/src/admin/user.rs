use crate::admin::input::{self, UserEditor};
use crate::api;
use crate::app::AppState;
use crate::auth::password;
use crate::config::RegexType;
use crate::schema::{user, user_token};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

/// This function prompts the user for input again to reset passwords for specific users.
pub fn reset_password(state: &AppState, editor: &mut UserEditor) {
    input::user_input_loop(state, editor, |state: &AppState, editor: &mut UserEditor| {
        println!(
            "Please enter the username of the user you would like to reset a password for. Enter \"done\" when finished."
        );
        let user = input::read("Username: ", editor)?;

        // Check if user exists
        let mut conn = state
            .connection_pool
            .get_blocking()
            .map_err(|err| format!("Could not establish a connection to the database for reason: {err}"))?;
        let user_id = user::table
            .select(user::id)
            .filter(user::name.eq(&user))
            .first::<i64>(&mut conn)
            .optional()
            .map_err(|err| format!("Could not determine if user exists for reason: {err}"))?
            .ok_or_else(|| "No user with this username exists".to_string())?;

        let password = input::read("New password: ", editor)?;
        api::verify_matches_regex(&state.config, &password, RegexType::Password)?;

        let salt = SaltString::generate(&mut OsRng);
        let hash = password::hash_password(&state.config, &password, &salt)
            .map_err(|err| format!("Could not hash password for reason: {err}"))?;
        diesel::update(user::table.find(user_id))
            .set((user::password_hash.eq(&hash), user::password_salt.eq(salt.as_str())))
            .execute(&mut conn)
            .map_err(|err| format!("Could not update password for reason: {err}"))?;

        // Revoke all of the user's API tokens so that a stolen token does not
        // survive a password reset.
        diesel::delete(user_token::table.filter(user_token::user_id.eq(user_id)))
            .execute(&mut conn)
            .map_err(|err| format!("Could not revoke API tokens for reason: {err}"))?;

        println!("Password reset successful.\n");
        Ok(())
    });
}
