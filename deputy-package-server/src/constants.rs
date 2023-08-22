use chrono::NaiveDateTime;
use lazy_static::lazy_static;

pub const fn default_page() -> u32 {
    1
}

pub const fn default_limit() -> u32 {
    20
}

pub const PACKAGE_TOML: &str = "package.toml";

pub const NAIVEDATETIME_DEFAULT_STRING: &str = "1970-01-01 00:00:01";
pub const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

lazy_static! {
    pub static ref NAIVEDATETIME_DEFAULT_VALUE: NaiveDateTime =
        NaiveDateTime::parse_from_str(NAIVEDATETIME_DEFAULT_STRING, DATETIME_FORMAT).unwrap();
}
