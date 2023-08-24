pub const CONFIGURATION_FILE_RELATIVE_PATH: &str = "configuration.toml";
pub const TOKEN_FILE_RELATIVE_PATH: &str = "token";
pub const PACKAGE_TOML: &str = "package.toml";
pub const DEFAULT_REGISTRY_NAME: &str = "main-registry";

pub mod fetching {
    pub const DEFAULT_SAVE_PATH: &str = ".";
    pub const DEFAULT_PACKAGE_VERSION_REQUIREMENT: &str = "*";
}
pub mod inspecting {
    pub const DEFAULT_PACKAGE_PATH: &str = ".";
}
pub mod endpoints {
    pub const PACKAGE_UPLOAD_PATH: &str = "api/v1/package";
}
