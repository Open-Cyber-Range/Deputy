use actix::{Handler, Message, ResponseActFuture, WrapFuture};
use actix_web::web::block;
use anyhow::{Ok, Result};
use crate::models::Package;
use super::Database;

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