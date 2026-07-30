use crate::auth::Client;
use crate::model::custom_field::CustomField;
use crate::model::enums::{ResourceOperation, ResourceType};
use crate::model::snapshot::NewSnapshot;
use crate::snapshot;
use diesel::{PgConnection, QueryResult};
use serde_json::{Value, json};

pub fn creation_snapshot(conn: &mut PgConnection, client: Client, custom_field: &CustomField) -> QueryResult<()> {
    unary_snapshot(conn, client, custom_field, ResourceOperation::Created)
}

pub fn modification_snapshot(
    conn: &mut PgConnection,
    client: Client,
    old: &CustomField,
    new: &CustomField,
) -> QueryResult<()> {
    assert_eq!(old.id, new.id);

    let old_data = snapshot_data(old);
    let new_data = snapshot_data(new);
    if let Some(data) = snapshot::value_diff(old_data, new_data) {
        NewSnapshot {
            user_id: client.id,
            operation: ResourceOperation::Modified,
            resource_type: ResourceType::CustomField,
            resource_id: old.id.to_string().into(),
            data,
        }
        .insert(conn)?;
    }
    Ok(())
}

pub fn deletion_snapshot(conn: &mut PgConnection, client: Client, custom_field: &CustomField) -> QueryResult<()> {
    unary_snapshot(conn, client, custom_field, ResourceOperation::Deleted)
}

fn snapshot_data(custom_field: &CustomField) -> Value {
    json!({
        "name": custom_field.name,
        "description": custom_field.description,
        "color": custom_field.color,
    })
}

fn unary_snapshot(
    conn: &mut PgConnection,
    client: Client,
    custom_field: &CustomField,
    operation: ResourceOperation,
) -> QueryResult<()> {
    NewSnapshot {
        user_id: client.id,
        operation,
        resource_type: ResourceType::CustomField,
        resource_id: custom_field.id.to_string().into(),
        data: snapshot_data(custom_field),
    }
    .insert(conn)
}
