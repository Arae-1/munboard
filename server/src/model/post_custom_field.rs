use crate::model::custom_field::CustomField;
use crate::model::post::Post;
use crate::schema::post_custom_field;
use diesel::pg::Pg;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};

#[derive(Associations, Identifiable, Insertable, Queryable, Selectable)]
#[diesel(belongs_to(CustomField), belongs_to(Post))]
#[diesel(table_name = post_custom_field)]
#[diesel(primary_key(post_id, custom_field_id))]
#[diesel(check_for_backend(Pg))]
pub struct PostCustomField {
    pub post_id: i64,
    pub custom_field_id: i64,
    pub value: i64,
}
