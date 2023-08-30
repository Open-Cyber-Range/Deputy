use crate::middleware::authentication::local_token::UserTokenInfo;
use crate::models::Owner;
use crate::models::{OwnerQuery, Owners};
use crate::services::database::owner::{AddOwner, DeleteOwner, GetOwners};
use crate::{
    errors::{PackageServerError, ServerResponseError},
    AppState,
};
use actix::{Actor, Handler};
use actix_web::{
    web::{Data, Json, Path, Query},
    Error,
};
use anyhow::Result;
use log::{debug, error};

pub async fn add_owner<T>(
    path_variables: Path<String>,
    app_state: Data<AppState<T>>,
    user_info: UserTokenInfo,
    query: Query<OwnerQuery>,
) -> Result<Json<Owner>, Error>
where
    T: Actor + Handler<AddOwner>,
    T: Actor + Handler<GetOwners>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, AddOwner>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, GetOwners>,
{
    let package_name = path_variables.into_inner();

    let owners = app_state
        .database_address
        .send(GetOwners(package_name.clone()))
        .await
        .map_err(|error| {
            error!("Failed to get owners: {error}");
            ServerResponseError(PackageServerError::OwnersList.into())
        })?
        .map_err(|error| {
            error!("Failed to get owners: {error}");
            ServerResponseError(PackageServerError::OwnersList.into())
        })?;

    if owners.contains_email(&user_info.email) {
        let owner = app_state
            .database_address
            .send(AddOwner {
                package_name: package_name.clone(),
                email: query.email.clone(),
            })
            .await
            .map_err(|error| {
                error!("Failed to add owner: {error}");
                ServerResponseError(PackageServerError::OwnerAdd.into())
            })?
            .map_err(|error| {
                error!("Failed to add owner: {error}");
                ServerResponseError(PackageServerError::OwnerAdd.into())
            })?;
        debug!(
            "Added owner: {owner_email} to package: {package_name}",
            owner_email = query.email
        );
        Ok(Json(owner))
    } else {
        error!("Requester not authorized to add owner to package");
        Err(ServerResponseError(PackageServerError::NotAuthorized.into()).into())
    }
}

pub async fn get_all_owners<T>(
    path_variables: Path<String>,
    app_state: Data<AppState<T>>,
) -> Result<Json<Owners>, Error>
where
    T: Actor + Handler<GetOwners>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, GetOwners>,
{
    let package_name = path_variables.into_inner();
    let owners = app_state
        .database_address
        .send(GetOwners(package_name))
        .await
        .map_err(|error| {
            error!("Failed to get owners: {error}");
            ServerResponseError(PackageServerError::OwnersList.into())
        })?
        .map_err(|error| {
            error!("Failed to get owners: {error}");
            ServerResponseError(PackageServerError::OwnersList.into())
        })?;

    Ok(Json(owners))
}

pub async fn delete_owner<T>(
    path_variables: Path<(String, String)>,
    app_state: Data<AppState<T>>,
    user_info: UserTokenInfo,
) -> Result<Json<String>, Error>
where
    T: Actor + Handler<DeleteOwner>,
    T: Actor + Handler<GetOwners>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, DeleteOwner>,
    <T as Actor>::Context: actix::dev::ToEnvelope<T, GetOwners>,
{
    let (package_name, owner_email) = path_variables.into_inner();

    let owners = app_state
        .database_address
        .send(GetOwners(package_name.clone()))
        .await
        .map_err(|error| {
            error!("Failed to get owners: {error}");
            ServerResponseError(PackageServerError::OwnersList.into())
        })?
        .map_err(|error| {
            error!("Failed to get owners: {error}");
            ServerResponseError(PackageServerError::OwnersList.into())
        })?;

    if owners.contains_email(&user_info.email) {
        app_state
            .database_address
            .send(DeleteOwner(package_name.clone(), owner_email.clone()))
            .await
            .map_err(|error| {
                error!("Failed to delete owner: {error}");
                ServerResponseError(PackageServerError::OwnerRemove.into())
            })?
            .map_err(|error| {
                error!("Failed to delete owner: {error}");
                ServerResponseError(PackageServerError::OwnerRemove.into())
            })?;

        debug!("Deleted owner: {owner_email} from package: {package_name}");
        Ok(Json(owner_email))
    } else {
        Err(ServerResponseError(PackageServerError::NotAuthorized.into()).into())
    }
}
