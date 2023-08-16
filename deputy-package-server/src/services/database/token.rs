use super::Database;
use crate::models::token::{ApiToken, NewApiToken};
use actix::{Handler, Message, ResponseActFuture, WrapFuture};
use actix_web::web::block;
use anyhow::{Ok, Result};
use diesel::RunQueryDsl;

#[derive(Message)]
#[rtype(result = "Result<ApiToken>")]
pub struct CreateApiToken(pub NewApiToken);

impl Handler<CreateApiToken> for Database {
    type Result = ResponseActFuture<Self, Result<ApiToken>>;

    fn handle(&mut self, msg: CreateApiToken, _ctx: &mut Self::Context) -> Self::Result {
        let new_api_token = msg.0;
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let api_token = block(move || {
                    new_api_token.create_insert().execute(&mut connection)?;
                    let api_token = ApiToken::by_id(new_api_token.id).first(&mut connection)?;
                    Ok(api_token)
                })
                .await??;
                Ok(api_token)
            }
            .into_actor(self),
        )
    }
}
