use crate::auth::Client;
use crate::model::enums::{ResourceOperation, ResourceType};
use crate::model::snapshot::NewSnapshot;
use crate::model::user::User;
use crate::snapshot;
use diesel::{PgConnection, QueryResult};
use serde_json::{Value, json};

pub fn creation_snapshot(conn: &mut PgConnection, client: Client, user: &User) -> QueryResult<()> {
    unary_snapshot(conn, client, user, ResourceOperation::Created)
}

pub fn modification_snapshot(
    conn: &mut PgConnection,
    client: Client,
    old: &User,
    new: &User,
) -> QueryResult<()> {
    assert_eq!(old.id, new.id);

    let old_data = snapshot_data(old);
    let new_data = snapshot_data(new);
    if let Some(data) = snapshot::value_diff(old_data, new_data) {
        NewSnapshot {
            user_id: client.id,
            operation: ResourceOperation::Modified,
            resource_type: ResourceType::User,
            resource_id: old.name.clone(),
            data,
        }
        .insert(conn)?;
    }
    Ok(())
}

pub fn deletion_snapshot(conn: &mut PgConnection, client: Client, user: &User) -> QueryResult<()> {
    unary_snapshot(conn, client, user, ResourceOperation::Deleted)
}

fn snapshot_data(user: &User) -> Value {
    json!({
        "name": user.name,
        "rank": user.rank,
        "avatar_style": user.avatar_style,
    })
}

fn unary_snapshot(
    conn: &mut PgConnection,
    client: Client,
    user: &User,
    operation: ResourceOperation,
) -> QueryResult<()> {
    NewSnapshot {
        user_id: client.id,
        operation,
        resource_type: ResourceType::User,
        resource_id: user.name.clone(),
        data: snapshot_data(user),
    }
    .insert(conn)
}
