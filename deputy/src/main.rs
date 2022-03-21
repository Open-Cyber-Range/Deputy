use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use fancy_regex::Regex;
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
    sub_type: String,
}

#[derive(Debug, Serialize, PartialEq)]
enum ContentType {
    VM,
}

const VALID_VM_TYPES: &'static [&'static str] = &["PACKER"];

fn main() {}

impl<'de> serde::Deserialize<'de> for ContentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VMVisitor;

        impl<'de> serde::de::Visitor<'de> for VMVisitor {
            type Value = ContentType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string representing 'VM'")
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<ContentType, E> {
                Ok(match s.to_uppercase().as_str() {
                    "VM" => ContentType::VM,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
                })
            }
        }
        deserializer.deserialize_any(VMVisitor)
    }
}

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

fn validate_description(description: String) -> Result<()> {
    let description_re = Regex::new(r#"^[a-zA-Z0-9\s,.\-!?]+$"#)?;
    let description_result = description_re.is_match(&description)?;
    if description_result {
        Ok(())
    } else {
        Err(anyhow!(
            "Description {:?} must plain text, allowed characters alphanumeric and whitespaces.",
            description
        ))
    }
}

fn validate_version(version: String) -> Result<()> {
    let version_re = Regex::new(
        r#"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?"#,
    )?;
    let version_result = version_re.is_match(&version)?;
    if version_result {
        Ok(())
    } else {
        Err(anyhow!(
            "Version {:?} must match Semantic Versioning 2.0.0 https://semver.org/",
            version
        ))
    }
}

fn validate_authors(authors: Option<Vec<String>>) -> Result<()> {
    if authors.is_none() {
        Ok(())
    } else {
        let author_re = Regex::new(r#".+[\w\.-]+@[\w\.-]+\.\w{2,4}"#)?;
        let authors = authors.unwrap();
        for author in authors {
            let author_result = author_re.is_match(&author)?;
            if !author_result {
                return Err(anyhow!(
                    "Author {:?} must match \"<name> <example@email.com>\" pattern.",
                    author
                ));
            };
        }
        Ok(())
    }
}

fn validate_content(content_type: Content) -> Result<()> {
    return match VALID_VM_TYPES.contains(&content_type.sub_type.to_uppercase().as_str()) {
        true => Ok(()),
        false => {
            return Err(anyhow!(
                "Given sub-type {:?} is not supported. Supported types are: {:?}",
                content_type.sub_type,
                VALID_VM_TYPES
            ))
        }
    };
}

fn validate_package_toml(package_path: &Path) -> Result<()> {
    let mut file = File::open(package_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let deserialized_toml: Config = toml::from_str(&*contents)?;

    validate_name(deserialized_toml.package.name)?;
    validate_description(deserialized_toml.package.description)?;
    validate_version(deserialized_toml.package.version)?;
    validate_authors(deserialized_toml.package.authors)?;
    validate_content(deserialized_toml.content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use serial_test::serial;
    use tempfile::TempDir;

    use super::*;

    #[test]
    #[serial]
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
type = "vM"
sub_type = "packER"
"#,
        )?;

        let mut file = File::open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let deserialized_toml: Config = toml::from_str(&*contents)?;

        assert!(validate_name(deserialized_toml.package.name).is_ok());
        assert!(validate_description(deserialized_toml.package.description).is_ok());
        assert!(validate_version(deserialized_toml.package.version).is_ok());
        assert!(validate_authors(deserialized_toml.package.authors).is_ok());
        assert!(validate_content(deserialized_toml.content).is_ok());

        assert!(validate_package_toml(&file_path).is_ok());
        temp_dir.close()?;
        Ok(())
    }
}
