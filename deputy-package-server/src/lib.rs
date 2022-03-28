use configuration::Configuration;

pub mod configuration;
pub mod routes;

pub struct AppState {
    pub configuration: Configuration,
}
