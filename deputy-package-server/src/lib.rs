use actix::Addr;
use deputy_library::StorageFolders;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use futures::lock::Mutex;
use git2::Repository;
use std::sync::Arc;
use std::env;
use crate::services::database::Database;

pub mod configuration;
pub mod constants;
mod errors;
pub mod routes;
#[cfg(feature = "test")]
pub mod test;
pub mod utils;
pub mod schema;
pub mod services;
pub mod models;

#[derive(Clone, Debug)]
pub struct AppState {
    pub database_address: Addr<Database>,
    pub repository: Arc<Mutex<Repository>>,
    pub storage_folders: StorageFolders,
}
