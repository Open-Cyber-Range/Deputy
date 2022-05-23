use byte_unit::Byte;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SMALL_PACKAGE_LIMIT: u64 =
        u64::try_from(Byte::from_str("10MB").unwrap().get_bytes()).unwrap();
}

pub const CONFIG_FILE_PATH_ENV_KEY: &str = "DEPUTY_CONFIG";
pub const PACKAGE_TOML: &str = "package.toml";

pub mod endpoints {
    pub const SMALL_PACKAGE_UPLOAD_PATH: &str = "/api/v1/package";
    pub const LARGE_PACKAGE_UPLOAD_PATH: &str = "/api/v1/package/stream";
}
