use futures::lock::Mutex;
use git2::Repository;
use std::sync::Arc;

pub mod configuration;
mod errors;
pub mod routes;
#[cfg(feature = "test")]
pub mod test;

#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<Mutex<Repository>>,
    pub package_folder: String,
}
