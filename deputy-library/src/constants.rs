use fancy_regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VALID_NAME: Regex = Regex::new(r#"^[a-zA-Z0-9_-]+$"#).unwrap();
}

pub const SHA256_LENGTH: usize = 64;
pub const COMPERSSION_CHUNK_SIZE: usize = 131_072;

pub const INDEX_REPOSITORY_BRANCH: &str = "master";
pub const INDEX_REPOSITORY_REMOTE: &str = "origin";

pub const PAYLOAD_CHUNK_SIZE: u64 = 8192;
