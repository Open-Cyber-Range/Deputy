use crate::models::helpers::uuid::Uuid;
use crate::{
    schema::{packages, versions},
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
            package_type: String::from(package_metadata.package_type),
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
