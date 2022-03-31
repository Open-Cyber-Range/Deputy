#[cfg(feature = "test")]
#[macro_use]
extern crate lazy_static;

pub mod archiver;
mod constants;
pub mod package;
pub mod repository;
#[cfg(feature = "test")]
pub mod test;
mod toml_structure;
pub mod validation;
