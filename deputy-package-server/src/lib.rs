use actix::Addr;
use deputy_library::StorageFolders;
use futures::lock::Mutex;
use git2::Repository;
use std::sync::Arc;
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

#[derive(Clone, Debug)]
pub struct AppState {
    pub database_address: Addr<Database>,
    pub repository: Arc<Mutex<Repository>>,
    pub storage_folders: StorageFolders,
}
