use serde::{Deserialize, Serialize};

pub mod repository;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    name: String,
    version: String,
    checksum: String,
}
