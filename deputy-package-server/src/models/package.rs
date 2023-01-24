use crate::models::helpers::uuid::Uuid;
use crate::{
    schema::packages,
    services::database::{All, Create, FilterExisting},
};
use chrono::NaiveDateTime;
use diesel::helper_types::{Desc, FindBy, Order};
use diesel::{insert_into};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, QueryableByName, Selectable, Insertable, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[diesel(table_name = packages)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub license: String,
    pub readme: String,
    pub readme_html: String,
    pub checksum: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Package {
    fn all_with_deleted() -> All<packages::table, Self> {
        packages::table.select(Self::as_select())
    }

    pub fn all() -> Order<FilterExisting<All<packages::table, Self>, packages::deleted_at>, packages::version> {
        Self::all_with_deleted().filter(packages::deleted_at.is_null()).order_by(packages::version)
    }

    #[allow(clippy::type_complexity)]
    pub fn by_id(
        id: Uuid,
    ) -> FindBy<
            Order<FilterExisting<All<packages::table, Self>, packages::deleted_at>, packages::version>,
            packages::id,
            Uuid,
    > {
        Self::all().filter(packages::id.eq(id))
    }

    #[allow(clippy::type_complexity)]
    pub fn by_name_and_version(
        name: String,
        version: String,
    ) -> FindBy<
        FindBy<
            Order<FilterExisting<All<packages::table, Self>, packages::deleted_at>, packages::version>,
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

    #[allow(clippy::type_complexity)]
    pub fn by_name(
        name: String,
    ) -> Order<FindBy<Order<FilterExisting<All<packages::table, Self>, packages::deleted_at>, packages::version>, packages::name, String>, Desc<packages::version>> {
        Self::all().filter(packages::name.eq(name)).order(packages::version.desc())
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
    pub readme_html: String,
    pub checksum: String,
}

impl NewPackage {
    pub fn create_insert(&self) -> Create<&Self, packages::table> {
        insert_into(packages::table).values(self)
    }
}
