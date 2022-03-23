use serde::{Deserialize, Serialize};

pub mod constants;
pub mod repository;
pub mod validation;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    name: String,
    version: String,
    checksum: String,
}
