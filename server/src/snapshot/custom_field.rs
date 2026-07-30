use crate::api::error::{ApiError, ApiResult};
use crate::auth::Client;
use crate::model::custom_field::CustomField;
use crate::model::enums::{ResourceOperation, ResourceType};
use crate::model::snapshot::NewSnapshot;
use crate::snapshot;
use crate::string::SmallString;
use diesel::PgConnection;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct SnapshotData {
    pub name: SmallString,
    pub description: SmallString,
    pub color: SmallString,
}

impl SnapshotData {
    pub fn retrieve(custom_field: CustomField) -> Self {
        Self {
            name: custom_field.name,
            description: custom_field.description,
            color: custom_field.color,
        }
    }

    fn sort_fields(&mut self) {}
}

pub fn creation_snapshot(
    conn: &mut PgConnection,
    client: Client,
    custom_field_id: i64,
    data: SnapshotData,
) -> ApiResult<()> {
    unary_snapshot(conn, client, custom_field_id, data, ResourceOperation::Created)
}

pub fn modification_snapshot(
    conn: &mut PgConnection,
    client: Client,
    custom_field_id: i64,
    old: SnapshotData,
    new: SnapshotData,
) -> ApiResult<()> {
    let old_data = serde_json::to_value(old)?;
    let new_data = serde_json::to_value(new)?;
    if let Some(data) = snapshot::value_diff(old_data, new_data) {
        NewSnapshot {
            user_id: client.id,
            operation: ResourceOperation::Modified,
            resource_type: ResourceType::CustomField,
            resource_id: custom_field_id.into(),
            data,
        }
        .insert(conn)?;
    }
    Ok(())
}

pub fn deletion_snapshot(
    conn: &mut PgConnection,
    client: Client,
    custom_field_id: i64,
    data: SnapshotData,
) -> ApiResult<()> {
    unary_snapshot(conn, client, custom_field_id, data, ResourceOperation::Deleted)
}

fn unary_snapshot(
    conn: &mut PgConnection,
    client: Client,
    custom_field_id: i64,
    mut data: SnapshotData,
    operation: ResourceOperation,
) -> ApiResult<()> {
    data.sort_fields();
    serde_json::to_value(data)
        .map_err(ApiError::from)
        .and_then(|data| {
            NewSnapshot {
                user_id: client.id,
                operation,
                resource_type: ResourceType::CustomField,
                resource_id: custom_field_id.into(),
                data,
            }
            .insert(conn)
            .map_err(ApiError::from)
        })
}
