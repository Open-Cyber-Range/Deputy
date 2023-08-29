use crate::models::helpers::uuid::Uuid;
use crate::models::{Category, NewCategory, NewPackageVersion, Package, PackageVersion, Version};
use crate::services::database::package::{
    CreateCategory, CreatePackage, GetPackageByNameAndVersion, GetPackages,
    GetVersionsByPackageName, UpdateVersionMsg,
};
use actix::Actor;
use actix::ActorFutureExt;
use actix::{Handler, ResponseActFuture, WrapFuture};
use anyhow::{Ok, Result};
use chrono::NaiveDateTime;
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct MockDatabase {
    packages: HashMap<Uuid, Package>,
    package_versions: HashMap<Uuid, Vec<Version>>,
    categories: HashMap<Uuid, Category>,
}

impl Actor for MockDatabase {
    type Context = actix::Context<Self>;
}

impl From<NewPackageVersion> for PackageVersion {
    fn from(NewPackageVersion(new_package, new_version): NewPackageVersion) -> Self {
        let package = Package {
            id: new_package.id,
            package_type: new_package.package_type,
            created_at: NaiveDateTime::MAX,
            updated_at: NaiveDateTime::MAX,
            deleted_at: None,
            name: new_package.name,
        };
        let version = Version {
            id: Uuid::random().to_owned(),
            created_at: NaiveDateTime::MAX,
            updated_at: NaiveDateTime::MAX,
            deleted_at: None,
            package_id: new_package.id,
            version: new_version.version,
            description: new_version.description,
            license: new_version.license,
            is_yanked: new_version.is_yanked,
            readme_html: new_version.readme_html,
            package_size: new_version.package_size,
            checksum: new_version.checksum,
        };
        Self(package, version)
    }
}

impl Handler<CreatePackage> for MockDatabase {
    type Result = ResponseActFuture<Self, Result<PackageVersion>>;

    fn handle(&mut self, msg: CreatePackage, _ctx: &mut Self::Context) -> Self::Result {
        let new_package_version = msg.0;
        let PackageVersion(package, version) = new_package_version.into();

        Box::pin(
            async move { package }
                .into_actor(self)
                .map(move |package, mock_database, _| {
                    mock_database.packages.insert(package.id, package.clone());
                    mock_database
                        .package_versions
                        .entry(package.id)
                        .or_default()
                        .push(version.clone());
                    Ok(PackageVersion(package, version))
                }),
        )
    }
}

impl Handler<GetPackages> for MockDatabase {
    type Result = ResponseActFuture<Self, Result<Vec<Package>>>;

    fn handle(&mut self, _msg: GetPackages, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(
            async move {}
                .into_actor(self)
                .map(move |_, mock_database, _| {
                    let packages = mock_database.packages.values().cloned().collect();
                    Ok(packages)
                }),
        )
    }
}

impl Handler<GetPackageByNameAndVersion> for MockDatabase {
    type Result = ResponseActFuture<Self, Result<Version>>;

    fn handle(
        &mut self,
        msg: GetPackageByNameAndVersion,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let name = msg.name;
        let version_value = msg.version;
        Box::pin(async move { (name, version_value) }.into_actor(self).map(
            move |(name, version_value), mock_database, _| {
                let packages: Vec<Package> = mock_database.packages.values().cloned().collect();

                let package = packages
                    .into_iter()
                    .find(|package| package.name == name)
                    .ok_or(anyhow::anyhow!("Package not found"))?;

                let version = mock_database
                    .package_versions
                    .get(&package.id)
                    .ok_or(anyhow::anyhow!("Package not found"))?
                    .iter()
                    .find(|version| version.version == version_value)
                    .ok_or(anyhow::anyhow!("Package not found"))?
                    .clone();
                Ok(version)
            },
        ))
    }
}

impl Handler<GetVersionsByPackageName> for MockDatabase {
    type Result = ResponseActFuture<Self, Result<Vec<Version>>>;

    fn handle(&mut self, msg: GetVersionsByPackageName, _ctx: &mut Self::Context) -> Self::Result {
        let name = msg.0;
        Box::pin(
            async move { name }
                .into_actor(self)
                .map(move |name, mock_database, _| {
                    let packages: Vec<Package> = mock_database.packages.values().cloned().collect();

                    let package = packages.into_iter().find(|package| package.name == name);

                    if let Some(package) = package {
                        let versions = mock_database
                            .package_versions
                            .get(&package.id)
                            .ok_or(anyhow::anyhow!("Package not found"))?
                            .to_owned();

                        return Ok(versions);
                    }

                    Ok(vec![])
                }),
        )
    }
}

impl Handler<CreateCategory> for MockDatabase {
    type Result = ResponseActFuture<Self, Result<Category>>;

    fn handle(&mut self, msg: CreateCategory, _ctx: &mut Self::Context) -> Self::Result {
        let new_category: NewCategory = msg.0;

        Box::pin(async move { new_category }.into_actor(self).map(
            move |new_category, mock_database, _| {
                let category = Category {
                    id: new_category.id,
                    name: new_category.name,
                    created_at: Default::default(),
                    updated_at: Default::default(),
                    deleted_at: None,
                };
                mock_database
                    .categories
                    .insert(category.id, category.clone());
                Ok(category)
            },
        ))
    }
}

impl Handler<UpdateVersionMsg> for MockDatabase {
    type Result = ResponseActFuture<Self, Result<Version>>;

    fn handle(&mut self, msg: UpdateVersionMsg, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(
            async move { msg }
                .into_actor(self)
                .map(move |msg, mock_database, _| {
                    let version = msg.version;
                    let versions = mock_database
                        .package_versions
                        .get_mut(&version.package_id)
                        .ok_or(anyhow::anyhow!("Package not found"))?;
                    let index = versions
                        .iter()
                        .position(|v| v.id == version.id)
                        .ok_or(anyhow::anyhow!("Version not found"))?;
                    versions[index] = version.clone();
                    Ok(version)
                }),
        )
    }
}
