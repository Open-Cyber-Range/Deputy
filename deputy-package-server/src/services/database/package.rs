use super::Database;
use crate::models::{NewPackage, Package};
use actix::{Handler, Message, ResponseActFuture, WrapFuture};
use actix_web::web::block;
use anyhow::{Ok, Result};
use diesel::RunQueryDsl;

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
pub struct GetPackages;

impl Handler<GetPackages> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<Package>>>;

    fn handle(&mut self, _: GetPackages, _ctx: &mut Self::Context) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let package = block(move || {
                    let packages = Package::all().load(&mut connection)?;
                    Ok(packages)
                })
                .await??;
                Ok(package)
            }
            .into_actor(self),
        )
    }
}
