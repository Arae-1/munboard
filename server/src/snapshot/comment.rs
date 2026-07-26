use crate::auth::Client;
use crate::model::comment::Comment;
use crate::model::enums::{ResourceOperation, ResourceType};
use crate::model::snapshot::NewSnapshot;
use crate::snapshot;
use diesel::{PgConnection, QueryResult};
use serde_json::{Value, json};

pub fn creation_snapshot(conn: &mut PgConnection, client: Client, comment: &Comment) -> QueryResult<()> {
    unary_snapshot(conn, client, comment, ResourceOperation::Created)
}

pub fn modification_snapshot(
    conn: &mut PgConnection,
    client: Client,
    old: &Comment,
    new: &Comment,
) -> QueryResult<()> {
    assert_eq!(old.id, new.id);

    let old_data = snapshot_data(old);
    let new_data = snapshot_data(new);
    if let Some(data) = snapshot::value_diff(old_data, new_data) {
        NewSnapshot {
            user_id: client.id,
            operation: ResourceOperation::Modified,
            resource_type: ResourceType::Comment,
            resource_id: old.id.into(),
            data,
        }
        .insert(conn)?;
    }
    Ok(())
}

pub fn deletion_snapshot(conn: &mut PgConnection, client: Client, comment: &Comment) -> QueryResult<()> {
    unary_snapshot(conn, client, comment, ResourceOperation::Deleted)
}

fn snapshot_data(comment: &Comment) -> Value {
    json!({
        "text": comment.text,
        "post_id": comment.post_id,
    })
}

fn unary_snapshot(
    conn: &mut PgConnection,
    client: Client,
    comment: &Comment,
    operation: ResourceOperation,
) -> QueryResult<()> {
    NewSnapshot {
        user_id: client.id,
        operation,
        resource_type: ResourceType::Comment,
        resource_id: comment.id.into(),
        data: snapshot_data(comment),
    }
    .insert(conn)
}
