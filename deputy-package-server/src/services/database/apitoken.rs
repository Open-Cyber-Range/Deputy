use super::Database;
use crate::models::apitoken::{ApiToken, ApiTokenRest, NewApiToken};
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

#[derive(Message)]
#[rtype(result = "Result<Vec<ApiTokenRest>>")]
pub struct GetApiTokens {
    pub user_id: String,
}

impl Handler<GetApiTokens> for Database {
    type Result = ResponseActFuture<Self, Result<Vec<ApiTokenRest>>>;

    fn handle(&mut self, get_api_tokens: GetApiTokens, _ctx: &mut Self::Context) -> Self::Result {
        let connection_result = self.get_connection();

        Box::pin(
            async move {
                let mut connection = connection_result?;
                let api_tokens = block(move || {
                    let api_tokens =
                        ApiToken::by_user_id(get_api_tokens.user_id).load(&mut connection)?;
                    let api_tokens = api_tokens
                        .into_iter()
                        .map(|api_token| api_token.into())
                        .collect();
                    Ok(api_tokens)
                })
                .await??;
                Ok(api_tokens)
            }
            .into_actor(self),
        )
    }
}
