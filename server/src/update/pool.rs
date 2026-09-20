use crate::api::error::{ApiError, ApiResult};
use crate::api::{self, error};
use crate::app::Context;
use crate::config::{Config, RegexType};
use crate::model::enums::{ResourceProperty, ResourceType};
use crate::model::pool::{NewPoolName, PoolPost, PoolSubpool};
use crate::schema::{pool, pool_name, pool_post, pool_subpool};
use crate::search::preferences;
use crate::string::SmallString;
use crate::time::DateTime;
use diesel::dsl::{exists, max};
use diesel::{BoolExpressionMethods, ExpressionMethods, Insertable, PgConnection, QueryDsl, QueryResult, RunQueryDsl};

/// Updates `last_edit_time` of pool associated with `pool_id`.
pub fn last_edit_time(conn: &mut PgConnection, pool_id: i64) -> QueryResult<()> {
    diesel::update(pool::table.find(pool_id))
        .set(pool::last_edit_time.eq(DateTime::now()))
        .execute(conn)?;
    Ok(())
}

/// Locks hierarchy edits for the current transaction.
pub fn lock_tree(conn: &mut PgConnection) -> QueryResult<()> {
    diesel::sql_query("SELECT 1 FROM pool_tree_lock WHERE id FOR UPDATE").execute(conn)?;
    Ok(())
}

/// Replaces the current ordered list of names with `names` for pool associated with `pool_id`.
pub fn set_names(conn: &mut PgConnection, config: &Config, pool_id: i64, names: &[SmallString]) -> ApiResult<()> {
    names
        .iter()
        .try_for_each(|name| api::verify_matches_regex(config, name, RegexType::Pool))?;

    diesel::delete(pool_name::table)
        .filter(pool_name::pool_id.eq(pool_id))
        .execute(conn)?;
    add_names(conn, pool_id, 0, names)
}

/// Replaces the current ordered list of posts with `posts` for pool associated with `pool_id`.
pub fn set_posts(conn: &mut PgConnection, ctx: &Context, pool_id: i64, posts: &mut Vec<i64>) -> ApiResult<()> {
    // Add posts client doesn't know about
    if let Some(hidden_posts) = preferences::hidden_posts(ctx, pool_post::post_id) {
        let hidden_posts: Vec<i64> = pool_post::table
            .select(pool_post::post_id)
            .filter(pool_post::pool_id.eq(pool_id))
            .filter(exists(hidden_posts))
            .load(conn)?;
        posts.extend(hidden_posts);
    }

    diesel::delete(pool_post::table)
        .filter(pool_post::pool_id.eq(pool_id))
        .execute(conn)?;
    add_posts(conn, pool_id, 0, posts)
}

/// Appends `posts` onto the current list of posts in the pool associated with `pool_id`.
pub fn add_posts(conn: &mut PgConnection, pool_id: i64, current_post_count: i64, posts: &[i64]) -> ApiResult<()> {
    let total_post_count = i64::try_from(posts.len())
        .unwrap_or(i64::MAX)
        .saturating_add(current_post_count);
    let new_pool_posts: Vec<_> = posts
        .iter()
        .zip(current_post_count..total_post_count)
        .map(|(&post_id, order)| PoolPost {
            pool_id,
            post_id,
            order,
        })
        .collect();
    let insert_result = new_pool_posts.insert_into(pool_post::table).execute(conn);
    error::map_unique_or_foreign_key_violation(insert_result, ResourceProperty::PoolPost, ResourceType::Post)?;
    Ok(())
}

/// Sets the one direct parent of a pool. `None` makes the pool a root pool.
pub fn set_parent(conn: &mut PgConnection, child_id: i64, parent_id: Option<i64>) -> ApiResult<()> {
    lock_tree(conn)?;
    if parent_id == Some(child_id) {
        return Err(ApiError::CyclicDependency(ResourceType::Pool));
    }
    if let Some(parent_id) = parent_id {
        let parent_exists: bool = diesel::select(exists(pool::table.find(parent_id))).first(conn)?;
        if !parent_exists {
            return Err(ApiError::NotFound(ResourceType::Pool));
        }
    }
    diesel::delete(pool_subpool::table.filter(pool_subpool::child_id.eq(child_id))).execute(conn)?;
    if let Some(parent_id) = parent_id {
        let order = pool_subpool::table
            .select(max(pool_subpool::order) + 1)
            .filter(pool_subpool::parent_id.eq(parent_id))
            .first::<Option<i64>>(conn)?
            .unwrap_or(0);
        error::map_pool_tree_violation(
            PoolSubpool { parent_id, child_id, order }
                .insert_into(pool_subpool::table)
                .execute(conn),
        )?;
    }
    Ok(())
}

/// Replaces the ordered children of a pool.
pub fn set_children(conn: &mut PgConnection, parent_id: i64, child_ids: &[i64]) -> ApiResult<()> {
    lock_tree(conn)?;
    if child_ids.iter().any(|&child_id| child_id == parent_id) || child_ids.len() != child_ids.iter().collect::<std::collections::HashSet<_>>().len() {
        return Err(ApiError::CyclicDependency(ResourceType::Pool));
    }
    let existing: i64 = pool::table.filter(pool::id.eq_any(child_ids)).count().first(conn)?;
    if existing != i64::try_from(child_ids.len()).unwrap_or(i64::MAX) {
        return Err(ApiError::NotFound(ResourceType::Pool));
    }
    diesel::delete(pool_subpool::table.filter(pool_subpool::parent_id.eq(parent_id))).execute(conn)?;
    for (order, &child_id) in child_ids.iter().enumerate() {
        diesel::delete(pool_subpool::table.filter(pool_subpool::child_id.eq(child_id))).execute(conn)?;
        error::map_pool_tree_violation(PoolSubpool {
            parent_id,
            child_id,
            order: i64::try_from(order).unwrap_or(i64::MAX),
        }.insert_into(pool_subpool::table).execute(conn))?;
    }
    Ok(())
}

/// Returns true when a pool has a parent or a child edge.
pub fn has_relationships(conn: &mut PgConnection, pool_id: i64) -> QueryResult<bool> {
    diesel::select(exists(
        pool_subpool::table
            .filter(pool_subpool::parent_id.eq(pool_id).or(pool_subpool::child_id.eq(pool_id))),
    ))
    .first(conn)
}

/// Merges pool associated with `abosorbed_id` to one with associated with `merged_to_id`.
pub fn merge(conn: &mut PgConnection, absorbed_id: i64, merge_to_id: i64) -> ApiResult<()> {
    lock_tree(conn)?;
    if has_relationships(conn, absorbed_id)? || has_relationships(conn, merge_to_id)? {
        return Err(ApiError::PoolHasRelationships);
    }
    // Merge posts
    let merge_to_pool_posts = pool_post::table
        .select(pool_post::post_id)
        .filter(pool_post::pool_id.eq(merge_to_id))
        .into_boxed();
    let new_pool_posts: Vec<_> = pool_post::table
        .select(pool_post::post_id)
        .filter(pool_post::pool_id.eq(absorbed_id))
        .filter(pool_post::post_id.ne_all(merge_to_pool_posts))
        .order(pool_post::order)
        .load(conn)?;
    let post_count: i64 = pool_post::table
        .filter(pool_post::pool_id.eq(merge_to_id))
        .count()
        .first(conn)?;
    add_posts(conn, merge_to_id, post_count, &new_pool_posts)?;

    // Merge names
    let current_name_count = pool_name::table
        .select(max(pool_name::order) + 1)
        .filter(pool_name::pool_id.eq(merge_to_id))
        .first::<Option<_>>(conn)?
        .unwrap_or(0);
    let removed_names = diesel::delete(pool_name::table.filter(pool_name::pool_id.eq(absorbed_id)))
        .returning(pool_name::name)
        .get_results(conn)?;
    add_names(conn, merge_to_id, current_name_count, &removed_names)?;

    diesel::delete(pool::table.find(absorbed_id)).execute(conn)?;
    last_edit_time(conn, merge_to_id).map_err(ApiError::from)
}

/// Appends `names` onto the current list of names for the pool associated with `pool_id`.
fn add_names(conn: &mut PgConnection, pool_id: i64, current_name_count: i32, names: &[SmallString]) -> ApiResult<()> {
    let total_name_count = i32::try_from(names.len())
        .unwrap_or(i32::MAX)
        .saturating_add(current_name_count);
    let updated_names: Vec<_> = names
        .iter()
        .zip(current_name_count..total_name_count)
        .map(|(name, order)| NewPoolName { pool_id, order, name })
        .collect();
    let insert_result = updated_names.insert_into(pool_name::table).execute(conn);
    error::map_unique_violation(insert_result, ResourceProperty::PoolName)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{get_connection, reset_database};
    use crate::schema::post;
    use diesel::OptionalExtension;
    use serial_test::serial;

    #[test]
    #[serial]
    fn pool_tree_rejects_cycle_and_multiple_parents() {
        let mut conn = get_connection().unwrap();
        set_children(&mut conn, 4, &[5]).unwrap();

        let parent = pool_subpool::table
            .select(pool_subpool::parent_id)
            .filter(pool_subpool::child_id.eq(5))
            .first(&mut conn)
            .optional()
            .unwrap();
        assert_eq!(parent, Some(4));

        // The database trigger sees 4 -> 5 and rejects 5 -> 4.
        assert!(matches!(
            set_parent(&mut conn, 4, Some(5)),
            Err(ApiError::CyclicDependency(ResourceType::Pool))
        ));

        // The unique child key rejects a second direct parent even outside the API.
        assert!(PoolSubpool {
            parent_id: 2,
            child_id: 5,
            order: 0,
        }
        .insert_into(pool_subpool::table)
        .execute(&mut conn)
        .is_err());
        reset_database();
    }

    #[test]
    #[serial]
    fn pool_posts_persist_for_root_and_child_pools() {
        let mut conn = get_connection().unwrap();
        let post_id = post::table.select(post::id).first(&mut conn).unwrap();

        set_parent(&mut conn, 5, Some(4)).unwrap();
        add_posts(&mut conn, 4, 0, &[post_id]).unwrap();
        add_posts(&mut conn, 5, 0, &[post_id]).unwrap();

        for pool_id in [4, 5] {
            let saved_post_ids: Vec<i64> = pool_post::table
                .select(pool_post::post_id)
                .filter(pool_post::pool_id.eq(pool_id))
                .load(&mut conn)
                .unwrap();
            assert!(saved_post_ids.contains(&post_id));
        }
        reset_database();
    }
}
