use super::Database;
use crate::models::helpers::pagination::*;
use crate::models::{Category, NewPackageVersion, Package, PackageCategory, PackageVersion, Version};
use actix::{Handler, Message, ResponseActFuture, WrapFuture};
use actix_web::web::block;
use anyhow::{Ok, Result};
use diesel::{OptionalExtension, RunQueryDsl};
use crate::models::helpers::uuid::Uuid;

#[derive(Message)]
#[rtype(result = "Result<PackageVersion>")]
pub struct CreatePackage(pub NewPackageVersion);

impl Handler<CreatePackage> for Database {
    type Result = ResponseActFuture<Self, Result<PackageVersion>>;

    fn handle(&mut self, msg: CreatePackage, _ctx: &mut Self::Context) -> Self::Result {
        let NewPackageVersion(new_package, mut new_version) = msg.0;
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let optional_package = Package::by_name(new_package.name.clone())
                        .first(&mut connection)
                        .optional()?;
                    let existing_package = match optional_package {
                        Some(package) => package,
                        None => {
                            new_package.create_insert().execute(&mut connection)?;
                            Package::by_id(new_package.id).first(&mut connection)?
                        }
                    };
                    new_version.package_id = existing_package.id;
                    new_version.create_insert().execute(&mut connection)?;
                    let version = Version::by_id(new_version.id).first(&mut connection)?;
                    Ok(PackageVersion(existing_package, version))
                })
                .await??;
                Ok(package)
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Package>>")]
pub struct GetPackages {
    pub page: i64,
    pub per_page: i64,
}

impl Handler<GetPackages> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<Package>>>;

    fn handle(&mut self, get_packages: GetPackages, _ctx: &mut Self::Context) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let packages = Package::all()
                        .paginate(get_packages.page)
                        .per_page(get_packages.per_page)
                        .load_and_count_pages(&mut connection)?;
                    Ok(packages.0)
                })
                .await??;
                Ok(package)
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Package>>")]
pub struct SearchPackages {
    pub search_term: String,
    pub page: i64,
    pub per_page: i64,
}

impl Handler<SearchPackages> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<Package>>>;

    fn handle(
        &mut self,
        search_packages: SearchPackages,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let packages = Package::search_name(search_packages.search_term)
                        .paginate(search_packages.page)
                        .per_page(search_packages.per_page)
                        .load_and_count_pages(&mut connection)?;
                    Ok(packages.0)
                })
                .await??;
                Ok(package)
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<Version>")]
pub struct GetPackageByNameAndVersion {
    pub name: String,
    pub version: String,
}

impl Handler<GetPackageByNameAndVersion> for Database {
    type Result = ResponseActFuture<Self, Result<Version>>;

    fn handle(
        &mut self,
        query_params: GetPackageByNameAndVersion,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let package: Package =
                        Package::by_name(query_params.name).first(&mut connection)?;
                    let package_version = package
                        .exact_version(query_params.version)
                        .first(&mut connection)?;
                    Ok(package_version)
                })
                .await??;
                Ok(package)
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Version>>")]
pub struct GetVersionsByPackageName(pub String);

impl Handler<GetVersionsByPackageName> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<Version>>>;

    fn handle(
        &mut self,
        query_params: GetVersionsByPackageName,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let package = Package::by_name(query_params.0)
                        .first(&mut connection)
                        .optional()?;
                    if let Some(package) = package {
                        let package_versions = package.versions().load(&mut connection)?;
                        return Ok(package_versions);
                    }
                    Ok(Vec::new())
                })
                .await??;
                Ok(package)
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Category>>")]
pub struct GetCategoriesForPackage(pub Uuid);

impl Handler<GetCategoriesForPackage> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<Category>>>;

    fn handle(
        &mut self,
        query_params: GetCategoriesForPackage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let categories = block(move || {
                    let package_categories = PackageCategory::by_package_id(query_params.0)
                        .load(&mut connection)?;

                    let category_ids: Vec<Uuid> = package_categories
                        .iter()
                        .map(|pc| pc.category_id)
                        .collect();

                    let categories = Category::belonging_to_query(&category_ids)
                        .load(&mut connection)?;

                    Ok(categories)
                })
                    .await??;
                Ok(categories)
            }
            .into_actor(self),
        )
    }
}

