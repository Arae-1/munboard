use crate::app::Context;
use crate::content::hash::PostHash;
use crate::model::pool::{Pool, PoolName, PoolPost};
use crate::resource;
use crate::resource::field::{Batcher, Mask};
use crate::resource::post::MicroPost;
use crate::schema::{pool, pool_category, pool_name, pool_post, pool_subpool};
use crate::search::preferences;
use crate::string::{LargeString, SmallString};
use crate::time::DateTime;
use diesel::dsl::{exists, not};
use serde_with::skip_serializing_none;
use diesel::{
    BelongingToDsl, ExpressionMethods, GroupedBy, Identifiable, PgConnection, QueryDsl, QueryResult,
    JoinOnDsl, QueryableByName, RunQueryDsl,
};
use serde::Serialize;
use server_macros::non_nullable_options;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use strum::EnumString;
use utoipa::ToSchema;

/// A pool resource stripped down to `id`, `names`, `category`, `description` and `postCount` fields.
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MicroPool {
    /// Resource version. See [versioning](#Versioning).
    pub id: i64,
    /// List of pool names (aliases).
    #[schema(value_type = Vec<SmallString>)]
    pub names: Arc<[SmallString]>,
    /// The name of the category the given pool belongs to.
    pub category: SmallString,
    /// The pool description (instructions how to use, history etc.). The client should render it as Markdown.
    pub description: LargeString,
    /// The number of posts the pool has.
    pub post_count: i64,
    /// Launch date for the pool.
    pub launch_date: Option<DateTime>,
}

/// A direct parent of a pool.
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParentPool {
    pub id: i64,
    pub name: SmallString,
}

/// A direct child of a pool.
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChildPool {
    pub id: i64,
    pub name: SmallString,
    pub category: SmallString,
    pub post_count: i64,
}

/// A post shown on a pool page. Descendant posts contain their source pool.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolPostInfo {
    pub id: i64,
    pub thumbnail_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_pool: Option<ParentPool>,
}

#[derive(Clone, Copy, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum Field {
    Version,
    Id,
    Description,
    CreationTime,
    LastEditTime,
    Category,
    Names,
    Posts,
    PostCount,
    DirectPosts,
    Parent,
    Ancestors,
    Children,
    LaunchDate,
}

impl From<Field> for u64 {
    fn from(value: Field) -> Self {
        value as u64
    }
}

/// An ordered list of posts, with a description and category.
#[non_nullable_options]
#[skip_serializing_none]
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolInfo {
    /// Resource version. See [versioning](#Versioning).
    version: Option<DateTime>,
    /// The pool identifier.
    id: Option<i64>,
    /// The pool description (instructions how to use, history etc.). The client should render it as Markdown.
    description: Option<LargeString>,
    /// Time the pool was created.
    creation_time: Option<DateTime>,
    /// Time the pool was last edited.
    last_edit_time: Option<DateTime>,
    /// The name of the category the given pool belongs to.
    category: Option<SmallString>,
    /// A list of pool names (aliases).
    names: Option<Vec<SmallString>>,
    /// An ordered list of posts. Posts are ordered by insertion by default.
    posts: Option<Vec<PoolPostInfo>>,
    /// Direct posts only. Use this field for an edit view or a direct-post view.
    direct_posts: Option<Vec<MicroPost>>,
    /// The number of posts the pool has.
    post_count: Option<i64>,
    /// The direct parent pool, if this pool has one.
    parent: Option<Option<ParentPool>>,
    /// Parent pools from the root to the direct parent.
    ancestors: Option<Vec<ParentPool>>,
    /// Direct child pools in their configured order.
    children: Option<Vec<ChildPool>>,
    /// Launch date for the pool.
    launch_date: Option<Option<DateTime>>,
}

impl PoolInfo {
    pub fn new(conn: &mut PgConnection, ctx: &Context, pool: Pool, fields: Mask<Field>) -> QueryResult<Self> {
        Self::new_batch(conn, ctx, vec![pool], fields).map(resource::single)
    }

    pub fn new_from_id(conn: &mut PgConnection, ctx: &Context, pool_id: i64, fields: Mask<Field>) -> QueryResult<Self> {
        Self::new_batch_from_ids(conn, ctx, &[pool_id], fields).map(resource::single)
    }

    pub fn new_batch(
        conn: &mut PgConnection,
        ctx: &Context,
        pools: Vec<Pool>,
        fields: Mask<Field>,
    ) -> QueryResult<Vec<Self>> {
        let f = Batcher::new(fields, pools.len());
        let mut categories = f.exec(Field::Category, || get_categories(conn, &pools))?;
        let mut names = f.exec(Field::Names, || get_names(conn, &pools))?;
        let mut posts = f.exec(Field::Posts, || get_posts(conn, ctx, &pools))?;
        let mut direct_posts = f.exec(Field::DirectPosts, || get_direct_posts(conn, ctx, &pools))?;
        let mut parents = f.exec(Field::Parent, || get_parents(conn, &pools))?;
        let mut ancestors = f.exec(Field::Ancestors, || get_ancestors(conn, &pools))?;
        let mut children = f.exec(Field::Children, || get_children(conn, &pools))?;
        let mut post_counts = f.exec(Field::PostCount, || get_post_counts(conn, &pools))?;

        let mut results = pools
            .into_iter()
            .rev()
            .map(|pool| Self {
                version: fields[Field::Version].then_some(pool.last_edit_time),
                id: fields[Field::Id].then_some(pool.id),
                description: fields[Field::Description].then_some(pool.description),
                creation_time: fields[Field::CreationTime].then_some(pool.creation_time),
                last_edit_time: fields[Field::LastEditTime].then_some(pool.last_edit_time),
                category: categories.pop(),
                names: names.pop(),
                posts: posts.pop(),
                direct_posts: direct_posts.pop(),
                post_count: post_counts.pop(),
                parent: parents.pop(),
                ancestors: ancestors.pop(),
                children: children.pop(),
                launch_date: fields[Field::LaunchDate].then_some(pool.launch_date),
            })
            .collect::<Vec<_>>();
        results.reverse();
        Ok(results)
    }

    pub fn new_batch_from_ids(
        conn: &mut PgConnection,
        ctx: &Context,
        pool_ids: &[i64],
        fields: Mask<Field>,
    ) -> QueryResult<Vec<Self>> {
        let unordered_pools = pool::table.filter(pool::id.eq_any(pool_ids)).load(conn)?;
        let pools = resource::order_as(unordered_pools, pool_ids);
        Self::new_batch(conn, ctx, pools, fields)
    }
}

fn get_categories(conn: &mut PgConnection, pools: &[Pool]) -> QueryResult<Vec<SmallString>> {
    let pool_ids: Vec<_> = pools.iter().map(Identifiable::id).copied().collect();
    pool::table
        .inner_join(pool_category::table)
        .select((pool::id, pool_category::name))
        .filter(pool::id.eq_any(&pool_ids))
        .load(conn)
        .map(|category_names| {
            resource::order_transformed_as(category_names, &pool_ids, |&(pool_id, _)| pool_id)
                .into_iter()
                .map(|(_, category_name)| category_name)
                .collect()
        })
}

fn get_names(conn: &mut PgConnection, pools: &[Pool]) -> QueryResult<Vec<Vec<SmallString>>> {
    Ok(PoolName::belonging_to(pools)
        .order(pool_name::order)
        .load::<PoolName>(conn)?
        .grouped_by(pools)
        .into_iter()
        .map(|pool_names| pool_names.into_iter().map(|pool_name| pool_name.name).collect())
        .collect())
}

fn get_direct_posts(conn: &mut PgConnection, ctx: &Context, pools: &[Pool]) -> QueryResult<Vec<Vec<MicroPost>>> {
    let mut pool_posts = PoolPost::belonging_to(pools).order(pool_post::order).into_boxed();

    // Apply preference filters to pool posts
    if let Some(hidden_posts) = preferences::hidden_posts(ctx, pool_post::post_id) {
        pool_posts = pool_posts.filter(not(exists(hidden_posts)));
    }

    Ok(pool_posts
        .load::<PoolPost>(conn)?
        .grouped_by(pools)
        .into_iter()
        .map(|posts_in_pool| {
            posts_in_pool
                .into_iter()
                .map(|pool_post| MicroPost {
                    id: pool_post.post_id,
                    thumbnail_url: PostHash::new(&ctx.config, pool_post.post_id).thumbnail_url(),
                })
                .collect()
        })
        .collect())
}

fn get_posts(conn: &mut PgConnection, ctx: &Context, pools: &[Pool]) -> QueryResult<Vec<Vec<PoolPostInfo>>> {
    let descendants = get_descendants(conn, pools)?;
    let source_pool_ids: Vec<i64> = descendants.iter().map(|row| row.id).collect();
    let names: HashMap<i64, SmallString> = pool_name::table
        .select((pool_name::pool_id, pool_name::name))
        .filter(pool_name::pool_id.eq_any(&source_pool_ids))
        .filter(PoolName::primary())
        .load(conn)?
        .into_iter()
        .collect();
    let mut query = pool_post::table
        .filter(pool_post::pool_id.eq_any(&source_pool_ids))
        .order((pool_post::pool_id, pool_post::order))
        .into_boxed();
    if let Some(hidden_posts) = preferences::hidden_posts(ctx, pool_post::post_id) {
        query = query.filter(not(exists(hidden_posts)));
    }
    let mut posts_by_pool: HashMap<i64, Vec<PoolPost>> = HashMap::new();
    for pool_post in query.load::<PoolPost>(conn)? {
        posts_by_pool.entry(pool_post.pool_id).or_default().push(pool_post);
    }

    let mut descendants_by_root: HashMap<i64, Vec<i64>> = HashMap::new();
    for descendant in descendants {
        descendants_by_root.entry(descendant.root_id).or_default().push(descendant.id);
    }
    Ok(pools
        .iter()
        .map(|root| {
            let mut seen = HashSet::new();
            descendants_by_root
                .get(&root.id)
                .into_iter()
                .flatten()
                .flat_map(|source_pool_id| posts_by_pool.get(source_pool_id).into_iter().flatten())
                .filter(|pool_post| seen.insert(pool_post.post_id))
                .map(|pool_post| PoolPostInfo {
                    id: pool_post.post_id,
                    thumbnail_url: PostHash::new(&ctx.config, pool_post.post_id).thumbnail_url(),
                    source_pool: (pool_post.pool_id != root.id).then(|| ParentPool {
                        id: pool_post.pool_id,
                        name: names[&pool_post.pool_id].clone(),
                    }),
                })
                .collect()
        })
        .collect())
}

fn get_parents(conn: &mut PgConnection, pools: &[Pool]) -> QueryResult<Vec<Option<ParentPool>>> {
    let pool_ids: Vec<_> = pools.iter().map(Identifiable::id).copied().collect();
    let parents: HashMap<i64, ParentPool> = pool_subpool::table
        .inner_join(pool_name::table.on(pool_name::pool_id.eq(pool_subpool::parent_id)))
        .select((pool_subpool::child_id, pool_subpool::parent_id, pool_name::name))
        .filter(pool_subpool::child_id.eq_any(&pool_ids))
        .filter(PoolName::primary())
        .load::<(i64, i64, SmallString)>(conn)?
        .into_iter()
        .map(|(child_id, id, name)| (child_id, ParentPool { id, name }))
        .collect();
    Ok(pools.iter().map(|pool| parents.get(&pool.id).cloned()).collect())
}

fn get_ancestors(conn: &mut PgConnection, pools: &[Pool]) -> QueryResult<Vec<Vec<ParentPool>>> {
    #[derive(QueryableByName)]
    struct Ancestor {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        root_id: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        id: i64,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: SmallString,
    }
    let pool_ids: Vec<_> = pools.iter().map(Identifiable::id).copied().collect();
    let rows: Vec<Ancestor> = diesel::sql_query(
        "WITH RECURSIVE ancestors(root_id, id, depth) AS (\
            SELECT roots.root_id, edge.parent_id, 1::INTEGER FROM UNNEST($1::BIGINT[]) AS roots(root_id) JOIN pool_subpool edge ON edge.child_id = roots.root_id \
            UNION ALL \
            SELECT ancestors.root_id, edge.parent_id, ancestors.depth + 1 FROM pool_subpool edge JOIN ancestors ON edge.child_id = ancestors.id\
         ) SELECT ancestors.root_id, ancestors.id, pool_name.name FROM ancestors JOIN pool_name ON pool_name.pool_id = ancestors.id AND pool_name.order = 0 ORDER BY ancestors.root_id, ancestors.depth DESC",
    )
    .bind::<diesel::sql_types::Array<diesel::sql_types::BigInt>, _>(pool_ids)
    .load(conn)?;
    let mut result: HashMap<i64, Vec<ParentPool>> = HashMap::new();
    for row in rows {
        result.entry(row.root_id).or_default().push(ParentPool { id: row.id, name: row.name });
    }
    Ok(pools.iter().map(|pool| result.remove(&pool.id).unwrap_or_default()).collect())
}

fn get_children(conn: &mut PgConnection, pools: &[Pool]) -> QueryResult<Vec<Vec<ChildPool>>> {
    #[derive(QueryableByName)]
    struct ChildRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        parent_id: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        id: i64,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: SmallString,
        #[diesel(sql_type = diesel::sql_types::Text)]
        category: SmallString,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        post_count: i64,
    }
    let pool_ids: Vec<_> = pools.iter().map(Identifiable::id).copied().collect();
    let rows: Vec<ChildRow> = diesel::sql_query(
        "SELECT edge.parent_id, child.id, names.name, categories.name AS category, COUNT(posts.post_id) AS post_count \
         FROM pool_subpool edge \
         JOIN pool child ON child.id = edge.child_id \
         JOIN pool_category categories ON categories.id = child.category_id \
         JOIN pool_name names ON names.pool_id = child.id AND names.order = 0 \
         LEFT JOIN pool_post posts ON posts.pool_id = child.id \
         WHERE edge.parent_id = ANY($1) \
         GROUP BY edge.parent_id, edge.order, child.id, names.name, categories.name \
         ORDER BY edge.parent_id, edge.order",
    )
    .bind::<diesel::sql_types::Array<diesel::sql_types::BigInt>, _>(pool_ids)
    .load(conn)?;
    let mut result: HashMap<i64, Vec<ChildPool>> = HashMap::new();
    for row in rows {
        result.entry(row.parent_id).or_default().push(ChildPool {
            id: row.id,
            name: row.name,
            category: row.category,
            post_count: row.post_count,
        });
    }
    Ok(pools.iter().map(|pool| result.remove(&pool.id).unwrap_or_default()).collect())
}

#[derive(QueryableByName)]
struct PoolDescendant {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    root_id: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    id: i64,
}

fn get_descendants(conn: &mut PgConnection, pools: &[Pool]) -> QueryResult<Vec<PoolDescendant>> {
    let pool_ids: Vec<_> = pools.iter().map(Identifiable::id).copied().collect();
    diesel::sql_query(
        "WITH RECURSIVE descendants(root_id, id, depth) AS (\
            SELECT roots.root_id, roots.root_id, 0::INTEGER FROM UNNEST($1::BIGINT[]) AS roots(root_id) \
            UNION ALL \
            SELECT descendants.root_id, edge.child_id, descendants.depth + 1 \
            FROM pool_subpool edge JOIN descendants ON edge.parent_id = descendants.id\
         ) SELECT root_id, id FROM descendants ORDER BY root_id, depth, id",
    )
    .bind::<diesel::sql_types::Array<diesel::sql_types::BigInt>, _>(pool_ids)
    .load(conn)
}

fn get_post_counts(conn: &mut PgConnection, pools: &[Pool]) -> QueryResult<Vec<i64>> {
    let pool_ids: Vec<_> = pools.iter().map(Identifiable::id).copied().collect();
    let counts: HashMap<i64, i64> = diesel::sql_query(
        "WITH RECURSIVE descendants(root_id, id) AS (\
            SELECT roots.root_id, roots.root_id FROM UNNEST($1::BIGINT[]) AS roots(root_id) \
            UNION ALL \
            SELECT descendants.root_id, edge.child_id \
            FROM pool_subpool edge JOIN descendants ON edge.parent_id = descendants.id\
         ) \
         SELECT descendants.root_id, COUNT(DISTINCT pool_post.post_id) AS count \
         FROM descendants LEFT JOIN pool_post ON pool_post.pool_id = descendants.id \
         GROUP BY descendants.root_id",
    )
    .bind::<diesel::sql_types::Array<diesel::sql_types::BigInt>, _>(pool_ids)
    .load::<PoolPostCount>(conn)?
    .into_iter()
    .map(|count| (count.root_id, count.count))
    .collect();
    Ok(pools.iter().map(|pool| counts[&pool.id]).collect())
}

#[derive(QueryableByName)]
struct PoolPostCount {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    root_id: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{get_connection, reset_database};
    use crate::update::pool::set_parent;
    use serial_test::serial;

    #[test]
    #[serial]
    fn pool_tree_resources_batch_multiple_roots() {
        let mut conn = get_connection().unwrap();
        set_parent(&mut conn, 5, Some(4)).unwrap();
        let pools: Vec<Pool> = pool::table
            .filter(pool::id.eq_any([4, 5]))
            .order(pool::id)
            .load(&mut conn)
            .unwrap();

        let descendants = get_descendants(&mut conn, &pools).unwrap();
        let ids: Vec<(i64, i64)> = descendants
            .into_iter()
            .map(|descendant| (descendant.root_id, descendant.id))
            .collect();
        assert_eq!(ids, vec![(4, 4), (4, 5), (5, 5)]);

        let parents = get_parents(&mut conn, &pools).unwrap();
        assert!(parents[0].is_none());
        assert_eq!(parents[1].as_ref().map(|parent| parent.id), Some(4));

        let ancestors = get_ancestors(&mut conn, &pools).unwrap();
        assert!(ancestors[0].is_empty());
        assert_eq!(ancestors[1].last().map(|parent| parent.id), Some(4));

        let children = get_children(&mut conn, &pools).unwrap();
        assert_eq!(children[0].as_slice().first().map(|child| child.id), Some(5));
        assert!(children[1].is_empty());
        reset_database();
    }
}
