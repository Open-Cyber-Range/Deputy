use actix::Addr;
use deputy_library::StorageFolders;
use crate::services::database::Database;

pub mod configuration;
pub mod constants;
mod errors;
pub mod routes;
#[cfg(feature = "test")]
pub mod test;
pub mod schema;
pub mod services;
pub mod models;
pub mod utilities;
pub mod utils;

#[derive(Clone, Debug)]
pub struct AppState {
    pub database_address: Addr<Database>,
    pub storage_folders: StorageFolders,
}
