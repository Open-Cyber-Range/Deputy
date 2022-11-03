use crate::{
    schema::packages,
};
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

impl Package {
    fn all_with_deleted() -> All<packages::table, Self> {
        packages::table.select(Self::as_select())
    }

    pub fn all() -> FilterExisting<All<packages::table, Self>, packages::deleted_at> {
        Self::all_with_deleted().filter(packages::deleted_at.is_null())
    }
}
