use super::Database;
use crate::models::helpers::pagination::*;
use crate::models::helpers::uuid::Uuid;
use crate::models::{
    Category, NewCategory, NewOwner, NewPackageCategory, NewPackageVersion, Owner, Owners, Package,
    PackageCategory, PackageVersion, Version,
};
use actix::{Handler, Message, ResponseActFuture, WrapFuture};
use actix_web::web::block;
use anyhow::{anyhow, Ok, Result};
use diesel::{OptionalExtension, RunQueryDsl};

#[derive(Message)]
#[rtype(result = "Result<PackageVersion>")]
pub struct CreatePackage(pub NewPackageVersion, pub String);

impl Handler<CreatePackage> for Database {
    type Result = ResponseActFuture<Self, Result<PackageVersion>>;

    fn handle(&mut self, msg: CreatePackage, _ctx: &mut Self::Context) -> Self::Result {
        let NewPackageVersion(new_package, mut new_version) = msg.0;
        let requester_email = msg.1;
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let optional_package = Package::by_name(new_package.name.clone())
                        .first(&mut connection)
                        .optional()?;
                    let existing_package = match optional_package {
                        Some(package) => {
                            let owners =
                                Owners(Owner::by_package_id(package.id).load(&mut connection)?);
                            if !owners.contains_email(&requester_email) {
                                return Err(anyhow!("Requester is not an owner of this package"));
                            }
                            package
                        }
                        None => {
                            new_package.create_insert().execute(&mut connection)?;
                            let package = Package::by_id(new_package.id).first(&mut connection)?;
                            let package_owner = NewOwner::new(requester_email, package.id);
                            package_owner.create_insert().execute(&mut connection)?;
                            package
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
#[rtype(result = "Result<Category>")]
pub struct CreateCategory(pub NewCategory, pub Uuid);

impl Handler<CreateCategory> for Database {
    type Result = ResponseActFuture<Self, Result<Category>>;

    fn handle(&mut self, msg: CreateCategory, _ctx: &mut Self::Context) -> Self::Result {
        let new_category: NewCategory = msg.0;
        let package_id: Uuid = msg.1;
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let category: Category = block(move || {
                    let optional_category = Category::by_name(new_category.name.clone())
                        .first(&mut connection)
                        .optional()?;
                    let existing_category = match optional_category {
                        Some(cat) => cat,
                        None => {
                            new_category.create_insert().execute(&mut connection)?;
                            Category::by_id(new_category.id).first(&mut connection)?
                        }
                    };
                    let optional_package_category = PackageCategory::by_package_and_category_id(
                        package_id,
                        existing_category.id,
                    )
                    .first(&mut connection)
                    .optional()?;
                    if optional_package_category.is_none() {
                        let new_package_category = NewPackageCategory {
                            package_id,
                            category_id: existing_category.id,
                        };
                        new_package_category
                            .create_insert()
                            .execute(&mut connection)?;
                    }
                    Ok(existing_category)
                })
                .await??;
                Ok(category)
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Category>>")]
pub struct GetCategoriesForPackage {
    pub id: Uuid,
}

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
                    let package_categories =
                        PackageCategory::by_package_id(query_params.id).load(&mut connection)?;
                    let category_ids: Vec<Uuid> =
                        package_categories.iter().map(|pc| pc.category_id).collect();
                    let categories = Category::by_ids(category_ids).load(&mut connection)?;
                    Ok(categories)
                })
                .await??;
                Ok(categories)
            }
            .into_actor(self),
        )
    }
}

#[derive(Message)]
#[rtype(result = "Result<Version>")]
pub struct UpdateVersionMsg {
    pub id: Uuid,
    pub version: Version,
}

impl Handler<UpdateVersionMsg> for Database {
    type Result = ResponseActFuture<Self, Result<Version>>;

    fn handle(&mut self, msg: UpdateVersionMsg, _ctx: &mut Self::Context) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let version = block(move || {
                    msg.version.create_update(msg.id).execute(&mut connection)?;
                    let version = Version::by_id(msg.id).first(&mut connection)?;
                    Ok(version)
                })
                .await??;
                Ok(version)
            }
            .into_actor(self),
        )
    }
}
