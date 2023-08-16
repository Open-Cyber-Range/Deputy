use crate::{
    errors::{PackageServerError, ServerResponseError},
    middleware::authentication::jwt::UserInfo,
    models::apitoken::{ApiTokenRest, FullApiTokenRest, NewApiTokenRest},
    services::database::apitoken::{CreateApiToken, GetApiTokens},
    AppState,
};
use actix::{Actor, Handler};
use actix_web::{
    web::{Data, Json},
    Error,
};
use anyhow::Result;
use log::error;

pub async fn get_all_api_tokens<T>(
    app_state: Data<AppState<T>>,
    user_info: UserInfo,
) -> Result<Json<Vec<ApiTokenRest>>, Error>
where
    T: Actor + Handler<GetApiTokens>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, GetApiTokens>,
{
    let user_id = user_info.id.clone();
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

pub async fn create_api_token<T>(
    app_state: Data<AppState<T>>,
    new_api_token: Json<NewApiTokenRest>,
    user_info: UserInfo,
) -> Result<Json<FullApiTokenRest>, Error>
where
    T: Actor + Handler<CreateApiToken>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, CreateApiToken>,
{
    let new_api_token = new_api_token
        .into_inner()
        .create_new_token(user_info.id.clone());

    let full_api_token = app_state
        .database_address
        .send(CreateApiToken(new_api_token))
        .await
        .map_err(|error| {
            error!("Failed to get all tokens: {error}");
            ServerResponseError(PackageServerError::DatabaseRecordNotFound.into())
        })?
        .map_err(|error| {
            error!("Failed to get all tokens: {error}");
            ServerResponseError(PackageServerError::DatabaseRecordNotFound.into())
        })?;
    Ok(Json(full_api_token.into()))
}
