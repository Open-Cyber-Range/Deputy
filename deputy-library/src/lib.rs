#[cfg(feature = "test")]
#[macro_use]
extern crate lazy_static;

pub mod package;
pub mod repository;
#[cfg(feature = "test")]
pub mod test;
