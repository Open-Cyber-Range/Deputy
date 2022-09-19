use deputy_library::StorageFolders;
use futures::lock::Mutex;
use git2::Repository;
use std::sync::Arc;

pub mod configuration;
pub mod constants;
mod errors;
pub mod routes;
#[cfg(feature = "test")]
pub mod test;
pub mod utils;

#[derive(Clone, Debug)]
pub struct AppState {
    pub repository: Arc<Mutex<Repository>>,
    pub storage_folders: StorageFolders,
}
