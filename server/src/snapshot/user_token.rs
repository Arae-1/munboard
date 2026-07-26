use crate::auth::Client;
use crate::model::enums::{ResourceOperation, ResourceType};
use crate::model::snapshot::NewSnapshot;
use crate::model::user::UserToken;
use crate::snapshot;
use diesel::{PgConnection, QueryResult};
use serde_json::{Value, json};

pub fn creation_snapshot(conn: &mut PgConnection, client: Client, user_token: &UserToken) -> QueryResult<()> {
    unary_snapshot(conn, client, user_token, ResourceOperation::Created)
}

pub fn modification_snapshot(
    conn: &mut PgConnection,
    client: Client,
    old: &UserToken,
    new: &UserToken,
) -> QueryResult<()> {
    assert_eq!(old.id, new.id);

    let old_data = snapshot_data(old);
    let new_data = snapshot_data(new);
    if let Some(data) = snapshot::value_diff(old_data, new_data) {
        NewSnapshot {
            user_id: client.id,
            operation: ResourceOperation::Modified,
            resource_type: ResourceType::UserToken,
            resource_id: old.id.to_string().into(),
            data,
        }
        .insert(conn)?;
    }
    Ok(())
}

pub fn deletion_snapshot(conn: &mut PgConnection, client: Client, user_token: &UserToken) -> QueryResult<()> {
    unary_snapshot(conn, client, user_token, ResourceOperation::Deleted)
}

fn snapshot_data(user_token: &UserToken) -> Value {
    json!({
        "note": user_token.note,
        "enabled": user_token.enabled,
        "expiration_time": user_token.expiration_time,
    })
}

fn unary_snapshot(
    conn: &mut PgConnection,
    client: Client,
    user_token: &UserToken,
    operation: ResourceOperation,
) -> QueryResult<()> {
    NewSnapshot {
        user_id: client.id,
        operation,
        resource_type: ResourceType::UserToken,
        resource_id: user_token.id.to_string().into(),
        data: snapshot_data(user_token),
    }
    .insert(conn)
}
