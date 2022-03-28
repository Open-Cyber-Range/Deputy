use serde::{Deserialize, Serialize};

mod constants;
pub mod packager;
pub mod repository;
pub mod validation;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    name: String,
    version: String,
    checksum: String,
}
