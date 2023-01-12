use serde::{Deserialize, Serialize};

#[cfg(feature = "test")]
#[macro_use]
extern crate lazy_static;

pub mod archiver;
pub mod constants;
pub mod lockfile;
pub mod package;
pub mod project;
#[cfg(feature = "test")]
pub mod test;
pub mod validation;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageFolders {
    pub package_folder: String,
    pub toml_folder: String,
    pub readme_folder: String,
}
