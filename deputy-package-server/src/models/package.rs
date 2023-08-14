use crate::models::helpers::uuid::Uuid;
use crate::services::database::CategoryFilter;
use crate::{
    schema::{categories, package_categories, packages, versions},
    services::database::{All, Create, FilterExisting},
};
use chrono::NaiveDateTime;
use deputy_library::package::PackageMetadata;
use deputy_library::rest::VersionRest;
use diesel::helper_types::{Filter, FindBy, Like};
use diesel::insert_into;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable,
    QueryableByName,
    Identifiable,
    Selectable,
    Insertable,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Category {
    fn all_with_deleted() -> All<categories::table, Self> {
        categories::table.select(Self::as_select())
    }

    pub fn all() -> FilterExisting<All<categories::table, Self>, categories::deleted_at> {
        Self::all_with_deleted().filter(categories::deleted_at.is_null())
    }

    pub fn by_id(
        id: Uuid,
    ) -> FindBy<
        FilterExisting<All<categories::table, Self>, categories::deleted_at>,
        categories::id,
        Uuid,
    > {
        Self::all().filter(categories::id.eq(id))
    }

    pub fn by_ids(
        category_ids: Vec<Uuid>,
    ) -> CategoryFilter<categories::table, categories::id, categories::deleted_at, Self> {
        Self::all().filter(categories::id.eq_any(category_ids))
    }
}

#[derive(
    Queryable, Selectable, Insertable, Clone, Debug, Eq, PartialEq, Deserialize, Serialize,
)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub id: Uuid,
    pub name: String,
}

impl NewCategory {
    pub fn create_insert(&self) -> Create<&Self, categories::table> {
        insert_into(categories::table).values(self)
    }
}

#[derive(
    Queryable, Selectable, Insertable, Clone, Debug, Eq, PartialEq, Deserialize, Serialize,
)]
#[diesel(belongs_to(Package, foreign_key = package_id))]
#[diesel(belongs_to(Category, foreign_key = category_id))]
#[diesel(table_name = package_categories)]
pub struct NewPackageCategory {
    pub package_id: Uuid,
    pub category_id: Uuid,
}

impl NewPackageCategory {
    pub fn create_insert(&self) -> Create<&Self, package_categories::table> {
        insert_into(package_categories::table).values(self)
    }
}

#[derive(
    Associations, Clone, Queryable, QueryableByName, Selectable, Debug, Deserialize, Serialize,
)]
#[diesel(belongs_to(Package, foreign_key = package_id))]
#[diesel(belongs_to(Category, foreign_key = category_id))]
#[diesel(table_name = package_categories)]
pub struct PackageCategory {
    pub package_id: Uuid,
    pub category_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl PackageCategory {
    fn all_with_deleted() -> All<package_categories::table, Self> {
        package_categories::table.select(Self::as_select())
    }

    pub fn all(
    ) -> FilterExisting<All<package_categories::table, Self>, package_categories::deleted_at> {
        Self::all_with_deleted().filter(package_categories::deleted_at.is_null())
    }

    pub fn by_package_id(
        id: Uuid,
    ) -> FindBy<
        FilterExisting<All<package_categories::table, Self>, package_categories::deleted_at>,
        package_categories::package_id,
        Uuid,
    > {
        Self::all().filter(package_categories::package_id.eq(id))
    }
}

#[derive(
    Associations,
    Clone,
    Queryable,
    QueryableByName,
    Selectable,
    Identifiable,
    Debug,
    Deserialize,
    Serialize,
)]
#[diesel(belongs_to(Package, foreign_key = package_id))]
#[diesel(table_name = versions)]
pub struct Version {
    pub id: Uuid,
    pub package_id: Uuid,
    pub version: String,
    pub description: String,
    pub license: String,
    pub readme_html: String,
    pub checksum: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Version {
    pub fn by_id(
        id: Uuid,
    ) -> FindBy<FilterExisting<All<versions::table, Self>, versions::deleted_at>, versions::id, Uuid>
    {
        Self::all().filter(versions::id.eq(id))
    }

    fn all_with_deleted() -> All<versions::table, Self> {
        versions::table.select(Self::as_select())
    }

    pub fn all() -> FilterExisting<All<versions::table, Self>, versions::deleted_at> {
        Self::all_with_deleted().filter(versions::deleted_at.is_null())
    }
}

#[derive(
    Queryable,
    QueryableByName,
    Identifiable,
    Selectable,
    Insertable,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Deserialize,
    Serialize,
)]
#[diesel(table_name = packages)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
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

    pub fn search_name(
        search_term: String,
    ) -> Filter<
        FilterExisting<All<packages::table, Self>, packages::deleted_at>,
        Like<packages::name, String>,
    > {
        Self::all_with_deleted()
            .filter(packages::deleted_at.is_null())
            .filter(packages::name.like(format!("%{}%", search_term)))
    }

    pub fn by_id(
        id: Uuid,
    ) -> FindBy<FilterExisting<All<packages::table, Self>, packages::deleted_at>, packages::id, Uuid>
    {
        Self::all().filter(packages::id.eq(id))
    }

    pub fn by_name(
        name: String,
    ) -> FindBy<
        FilterExisting<All<packages::table, Self>, packages::deleted_at>,
        packages::name,
        String,
    > {
        Self::all().filter(packages::name.eq(name))
    }

    pub fn versions(&self) -> versions::BoxedQuery<'_, Mysql> {
        Version::belonging_to(self).into_boxed()
    }

    pub fn exact_version(
        &self,
        version: String,
    ) -> FindBy<versions::BoxedQuery<'_, Mysql>, versions::version, String> {
        self.versions().filter(versions::version.eq(version))
    }
}

pub struct PackageVersion(pub Package, pub Version);

#[derive(
    Queryable, Selectable, Insertable, Clone, Debug, Eq, PartialEq, Deserialize, Serialize,
)]
#[diesel(table_name = packages)]
pub struct NewPackage {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
}

impl NewPackage {
    pub fn create_insert(&self) -> Create<&Self, packages::table> {
        insert_into(packages::table).values(self)
    }
}

#[derive(
    Queryable, Selectable, Insertable, Clone, Debug, Eq, PartialEq, Deserialize, Serialize,
)]
#[diesel(table_name = versions)]
pub struct NewVersion {
    pub id: Uuid,
    pub version: String,
    pub description: String,
    pub license: String,
    pub readme_html: String,
    pub checksum: String,
    pub package_id: Uuid,
}

impl NewVersion {
    pub fn create_insert(&self) -> Create<&Self, versions::table> {
        insert_into(versions::table).values(self)
    }
}

pub struct NewPackageVersion(pub NewPackage, pub NewVersion);

impl From<(PackageMetadata, String)> for NewPackageVersion {
    fn from((package_metadata, readme_html): (PackageMetadata, String)) -> Self {
        let package = NewPackage {
            id: Uuid::random().to_owned(),
            name: package_metadata.name,
            package_type: package_metadata.package_type.to_string(),
        };
        let version = NewVersion {
            id: Uuid::random().to_owned(),
            version: package_metadata.version,
            description: package_metadata.description,
            license: package_metadata.license,
            readme_html,
            checksum: package_metadata.checksum,
            package_id: package.id,
        };

        NewPackageVersion(package, version)
    }
}

impl From<Version> for VersionRest {
    fn from(version: Version) -> Self {
        Self {
            id: version.id.into(),
            version: version.version,
            description: version.description,
            license: version.license,
            readme_html: version.readme_html,
            checksum: version.checksum,
            created_at: version.created_at,
            updated_at: version.updated_at,
        }
    }
}
