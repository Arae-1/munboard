use crate::schema::custom_field;
use crate::string::SmallString;
use crate::time::DateTime;
use diesel::pg::Pg;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};

#[derive(Insertable)]
#[diesel(table_name = custom_field)]
#[diesel(check_for_backend(Pg))]
pub struct NewCustomField<'a> {
    pub name: &'a str,
}

#[derive(Clone, Debug, AsChangeset, Identifiable, Queryable, Selectable)]
#[diesel(table_name = custom_field)]
#[diesel(check_for_backend(Pg))]
pub struct CustomField {
    pub id: i64,
    pub name: SmallString,
    pub creation_time: DateTime,
}
