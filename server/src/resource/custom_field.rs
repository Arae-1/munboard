use crate::app::Context;
use crate::model::custom_field::CustomField;
use crate::resource;
use crate::resource::field::{Batcher, Mask};
use crate::schema::custom_field;
use crate::string::SmallString;
use crate::time::DateTime;
use diesel::{Identifiable, PgConnection, QueryDsl, QueryResult, RunQueryDsl};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use server_macros::non_nullable_options;
use std::collections::HashMap;
use strum::EnumString;
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldValue {
    pub name: SmallString,
    pub value: i64,
}

#[derive(Clone, Copy, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum Field {
    Version,
    Id,
    Name,
    CreationTime,
}

impl From<Field> for u64 {
    fn from(value: Field) -> Self {
        value as u64
    }
}

#[non_nullable_options]
#[skip_serializing_none]
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldInfo {
    version: Option<DateTime>,
    id: Option<i64>,
    name: Option<SmallString>,
    creation_time: Option<DateTime>,
}

impl CustomFieldInfo {
    pub fn new(
        conn: &mut PgConnection,
        _ctx: &Context,
        custom_field: CustomField,
        fields: Mask<Field>,
    ) -> QueryResult<Self> {
        Self::new_batch(conn, _ctx, vec![custom_field], fields).map(resource::single)
    }

    pub fn new_from_id(
        conn: &mut PgConnection,
        ctx: &Context,
        custom_field_id: i64,
        fields: Mask<Field>,
    ) -> QueryResult<Self> {
        Self::new_batch_from_ids(conn, ctx, &[custom_field_id], fields).map(resource::single)
    }

    pub fn new_batch(
        _conn: &mut PgConnection,
        _ctx: &Context,
        custom_fields: Vec<CustomField>,
        fields: Mask<Field>,
    ) -> QueryResult<Vec<Self>> {
        let mut results = custom_fields
            .into_iter()
            .rev()
            .map(|cf| Self {
                version: fields[Field::Version].then_some(cf.creation_time),
                id: fields[Field::Id].then_some(cf.id),
                name: fields[Field::Name].then_some(cf.name),
                creation_time: fields[Field::CreationTime].then_some(cf.creation_time),
            })
            .collect::<Vec<_>>();
        results.reverse();
        Ok(results)
    }

    pub fn new_batch_from_ids(
        conn: &mut PgConnection,
        ctx: &Context,
        custom_field_ids: &[i64],
        fields: Mask<Field>,
    ) -> QueryResult<Vec<Self>> {
        let unordered = custom_field::table
            .filter(custom_field::id.eq_any(custom_field_ids))
            .load(conn)?;
        let cf_fields = resource::order_as(unordered, custom_field_ids);
        Self::new_batch(conn, ctx, cf_fields, fields)
    }
}

pub fn get_custom_fields_for_posts(
    conn: &mut PgConnection,
    post_ids: &[i64],
) -> QueryResult<HashMap<i64, Vec<CustomFieldValue>>> {
    use crate::schema::{custom_field, post_custom_field};

    let values: Vec<(i64, SmallString, i64)> = post_custom_field::table
        .inner_join(custom_field::table)
        .select((post_custom_field::post_id, custom_field::name, post_custom_field::value))
        .filter(post_custom_field::post_id.eq_any(post_ids))
        .load(conn)?;

    let mut map: HashMap<i64, Vec<CustomFieldValue>> = HashMap::new();
    for (post_id, name, value) in values {
        map.entry(post_id).or_default().push(CustomFieldValue { name, value });
    }
    Ok(map)
}
