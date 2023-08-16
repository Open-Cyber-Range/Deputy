use crate::{
    errors::{PackageServerError, ServerResponseError},
    services::database::apitoken::GetApiTokens,
    AppState,
};
use actix::{Actor, Handler};
use actix_web::{
    web::{Data, Json},
    Error,
};
use anyhow::Result;
use deputy_library::rest::ApiTokenRest;
use log::error;

pub async fn get_all_api_tokens<T>(
    app_state: Data<AppState<T>>,
) -> Result<Json<Vec<ApiTokenRest>>, Error>
where
    T: Actor + Handler<GetApiTokens>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, GetApiTokens>,
{
    let user_id = "123".to_string();
    let apitokens = app_state
        .database_address
        .send(GetApiTokens {
            user_id: user_id.clone(),
        })
        .await
        .map_err(|error| {
            error!("Failed to get all tokens: {error}");
            ServerResponseError(PackageServerError::DatabaseRecordNotFound.into())
        })?
        .map_err(|error| {
            error!("Failed to get all tokens: {error}");
            ServerResponseError(PackageServerError::DatabaseRecordNotFound.into())
        })?;
    Ok(Json(apitokens))
}
