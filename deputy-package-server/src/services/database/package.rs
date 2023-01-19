use super::Database;
use crate::models::{NewPackage, Package};
use actix::{Handler, Message, ResponseActFuture, WrapFuture};
use actix_web::web::block;
use anyhow::{Ok, Result};
use diesel::RunQueryDsl;
use crate::models::helpers::pagination::*;

#[derive(Message)]
#[rtype(result = "Result<Package>")]
pub struct CreatePackage(pub NewPackage);

impl Handler<CreatePackage> for Database {
    type Result = ResponseActFuture<Self, Result<Package>>;

    fn handle(&mut self, msg: CreatePackage, _ctx: &mut Self::Context) -> Self::Result {
        let new_package = msg.0;
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    new_package.create_insert().execute(&mut connection)?;
                    let package = Package::by_id(new_package.id).first(&mut connection)?;
                    Ok(package)
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
pub struct GetLatestPackages {
    pub page: i64,
    pub per_page: i64,
}

impl Handler<GetLatestPackages> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<Package>>>;

    fn handle(&mut self, get_latest_packages: GetLatestPackages, _ctx: &mut Self::Context) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let packages = Package::all()
                        .paginate(get_latest_packages.page)
                        .per_page(get_latest_packages.per_page)
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
#[rtype(result = "Result<Package>")]
pub struct GetPackageByNameAndVersion {
    pub name: String,
    pub version: String,
}

impl Handler<GetPackageByNameAndVersion> for Database {
    type Result = ResponseActFuture<Self, Result<Package>>;

    fn handle(&mut self, query_params: GetPackageByNameAndVersion, _ctx: &mut Self::Context) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let package = Package::by_name_and_version(query_params.name, query_params.version).first(&mut connection)?;
                    Ok(package)
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
pub struct GetPackagesByName {
    pub name: String,
}

impl Handler<GetPackagesByName> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<Package>>>;

    fn handle(&mut self, query_params: GetPackagesByName, _ctx: &mut Self::Context) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let packages = Package::by_name(query_params.name).load(&mut connection)?;
                    Ok(packages)
                })
                    .await??;
                Ok(package)
            }
            .into_actor(self),
        )
    }
}
