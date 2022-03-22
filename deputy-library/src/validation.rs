use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{anyhow, Result};
use fancy_regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    package: Package,
    content: Content,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Package {
    name: String,
    description: String,
    version: String,
    authors: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Content {
    #[serde(rename = "type")]
    content_type: ContentType,
    sub_type: SubType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum ContentType {
    #[serde(alias = "vm")]
    VM,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum SubType {
    #[serde(alias = "packer")]
    PACKER,
}

const VALID_VM_TYPES: &'static [&'static SubType] = &[&SubType::PACKER];

fn validate_name(name: String) -> Result<()> {
    let name_re = Regex::new(r#"^[a-zA-Z0-9_-]+$"#)?;
    let name_result = name_re.is_match(&name)?;
    if name_result {
        Ok(())
    } else {
        Err(anyhow!(
            "Name {:?} must be one word of alphanumeric, `-`, or `_` characters.",
            name
        ))
    }
}

fn validate_version(version: String) -> Result<()> {
    match Version::parse(&version.as_str()) {
        Ok(_) => return Ok(()),
        Err(_) => {
            return Err(anyhow!(
                "Version {:?} must match Semantic Versioning 2.0.0 https://semver.org/",
                version
            ))
        }
    };
}

fn validate_type(content: Content) -> Result<()> {
    let is_valid = match content.content_type {
        ContentType::VM => VALID_VM_TYPES.contains(&&content.sub_type),
    };

    if !is_valid {
        return Err(anyhow!("Sub-type mismatch with the type"));
    }
    Ok(())
}

pub fn package_toml<P: AsRef<Path> + Debug>(package_path: P) -> Result<()> {
    let mut file = File::open(package_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let deserialized_toml: Config = toml::from_str(&*contents)?;

    validate_name(deserialized_toml.package.name)?;
    validate_version(deserialized_toml.package.version)?;
    validate_type(deserialized_toml.content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn positive_result_all_fields_correct() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut file_path = temp_dir.path().to_path_buf();
        file_path.push("package.toml".to_string());

        let mut file = File::create(&file_path)?;
        file.write_all(
            br#"

[package]
name = "test_package_1-0-4"
description = "This package does nothing at all, and we spent 300 manhours on it..."
version = "1.0.4"
authors = ["Robert robert@exmaple.com", "Bobert the III bobert@exmaple.com", "Miranda Rustacean miranda@rustacean.rust" ]

[content]
type = "vm"
sub_type = "packer"
"#,
        )?;

        let mut file = File::open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let deserialized_toml: Config = toml::from_str(&*contents)?;

        assert!(validate_name(deserialized_toml.package.name).is_ok());
        assert!(validate_version(deserialized_toml.package.version).is_ok());
        assert!(validate_type(deserialized_toml.content).is_ok());

        assert!(package_toml(&file_path).is_ok());
        temp_dir.close()?;
        Ok(())
    }
}
