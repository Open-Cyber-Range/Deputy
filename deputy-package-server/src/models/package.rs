use crate::models::helpers::uuid::Uuid;
use crate::{
    schema::packages,
    services::database::{All, Create, FilterExisting, SelectById},
};
use chrono::NaiveDateTime;
use diesel::helper_types::FindBy;
use diesel::insert_into;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Insertable, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[diesel(table_name = packages)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub license: String,
    pub readme: String,
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

    pub fn by_id(
        id: Uuid,
    ) -> SelectById<packages::table, packages::id, packages::deleted_at, Self> {
        Self::all().filter(packages::id.eq(id))
    }

    pub fn by_name_and_version(
        name: String,
        version: String,
    ) -> FindBy<
        FindBy<
            FilterExisting<All<packages::table, Self>, packages::deleted_at>,
            packages::name,
            String,
        >,
        packages::version,
        String,
    > {
        Self::all()
            .filter(packages::name.eq(name))
            .filter(packages::version.eq(version))
    }

    pub fn create_insert(&self) -> Create<&Self, packages::table> {
        insert_into(packages::table).values(self)
    }
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[diesel(table_name = packages)]
pub struct NewPackage {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub license: String,
    pub readme: String,
}

impl NewPackage {
    pub fn create_insert(&self) -> Create<&Self, packages::table> {
        insert_into(packages::table).values(self)
    }
}
