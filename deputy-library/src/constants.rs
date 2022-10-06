use fancy_regex::Regex;
use lazy_static::lazy_static;
use parse_duration::parse;
use std::time::Duration;

pub const LOCKFILE: &str = "deputy.lock";
pub const LOCKFILE_TIMEOUT: &str = "5 minutes";
pub const LOCKFILE_SLEEP: &str = "250 milliseconds";

lazy_static! {
    pub static ref VALID_NAME: Regex = Regex::new(r#"^[a-zA-Z0-9_-]+$"#).unwrap();
    static ref LOCKFILE_TIMEOUT_DURATION: Duration =
        parse(LOCKFILE_TIMEOUT).expect("Error parsing lockfile timeout duration");
    pub static ref LOCKFILE_SLEEP_DURATION: Duration =
        parse(LOCKFILE_SLEEP).expect("Error parsing lockfile sleep duration");
    pub static ref LOCKFILE_TRIES: u64 =
        { LOCKFILE_TIMEOUT_DURATION.as_millis() / LOCKFILE_SLEEP_DURATION.as_millis() } as u64;
}

pub const SHA256_LENGTH: usize = 64;

pub const COMPRESSION_CHUNK_SIZE: usize = 131_072;

pub const PAYLOAD_CHUNK_SIZE: u64 = 8192;

pub const INDEX_REPOSITORY_BRANCH: &str = "master";
pub const INDEX_REPOSITORY_REMOTE: &str = "origin";
pub const CONFIGURATION_FOLDER_PATH_ENV_KEY: &str = "DEPUTY_CONFIG_FOLDER";
