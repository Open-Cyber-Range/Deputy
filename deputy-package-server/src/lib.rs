use deputy_library::StorageFolders;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use futures::lock::Mutex;
use git2::Repository;
use std::sync::Arc;
use std::env;

pub mod configuration;
pub mod constants;
mod errors;
pub mod models;
pub mod routes;
#[cfg(feature = "test")]
pub mod test;
pub mod utils;
pub mod schema;

pub fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub repository: Arc<Mutex<Repository>>,
    pub storage_folders: StorageFolders,
}
