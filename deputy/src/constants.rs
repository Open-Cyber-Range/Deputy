use byte_unit::Byte;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SMALL_PACKAGE_LIMIT: u64 =
        u64::try_from(Byte::from_str("256kB").unwrap().get_bytes()).unwrap();
}

pub const CONFIGURATION_FILE_RELATIVE_PATH: &str = "configuration.toml";
pub const PACKAGE_TOML: &str = "package.toml";
pub const DEFAULT_REGISTRY_NAME: &str = "main-registry";

pub mod fetching {
    pub const DEFAULT_SAVE_PATH: &str = ".";
    pub const DEFAULT_PACKAGE_VERSION_REQUIREMENT: &str = "*";
}

pub mod endpoints {
    pub const SMALL_PACKAGE_UPLOAD_PATH: &str = "api/v1/package";
    pub const LARGE_PACKAGE_UPLOAD_PATH: &str = "api/v1/package/stream";
}
