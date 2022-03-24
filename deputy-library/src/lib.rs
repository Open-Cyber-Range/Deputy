use serde::{Deserialize, Serialize};
#[cfg(test)]
#[macro_use]
extern crate lazy_static;

pub mod package;
pub mod repository;
#[cfg(test)]
mod test;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    name: String,
    version: String,
    checksum: String,
}
