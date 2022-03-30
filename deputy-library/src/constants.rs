use crate::validation::SubType;
use fancy_regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VALID_NAME: Regex = Regex::new(r#"^[a-zA-Z0-9_-]+$"#).unwrap();
    pub static ref VALID_VM_TYPES: &'static [&'static SubType] = &[&SubType::Packer];
}

pub const SHA256_LENGTH: usize = 64;
