use git2::Repository;

pub mod configuration;
mod errors;
pub mod routes;
#[cfg(feature = "test")]
pub mod test;

pub struct AppState {
    pub repository: Repository,
    pub package_folder: String,
}
