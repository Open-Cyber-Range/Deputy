use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone, Debug, Deserialize, Serialize)]
#[diesel(table_name = packages)]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub readme: String,
    pub license: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}