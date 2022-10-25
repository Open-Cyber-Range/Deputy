pub(crate) mod enums;

use crate::project::enums::{Architecture, OperatingSystem};
use anyhow::{Ok, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::{fs::File, io::Read, path::Path};

use self::enums::VirtualMachineType;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Project {
    pub package: Body,
    pub content: Content,
    #[serde(rename = "virtual-machine")]
    pub virtual_machine: Option<VirtualMachine>,
    pub feature: Option<Feature>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Account {
    pub name: String,
    pub password: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct VirtualMachine {
    pub accounts: Option<Vec<Account>>,
    pub default_account: Option<String>,
    #[serde(default)]
    pub operating_system: Option<OperatingSystem>,
    #[serde(default)]
    pub architecture: Option<Architecture>,
    #[serde(rename = "type")]
    pub virtual_machine_type: VirtualMachineType,
    file_path: String,
    pub readme_path: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    pub assets: Vec<Vec<String>>,
}

pub fn create_project_from_toml_path(toml_path: &Path) -> Result<Project, anyhow::Error> {
    let mut toml_file = File::open(toml_path)?;
    let mut contents = String::new();
    toml_file.read_to_string(&mut contents)?;
    let deserialized_toml: Project = toml::from_str(&*contents)?;
    Ok(deserialized_toml)
}

#[derive(Debug)]
enum Values<T> {
    Null,
    Value(T),
}

impl<T> From<Option<T>> for Values<T> {
    fn from(opt: Option<T>) -> Values<T> {
        match opt {
            Some(v) => Values::Value(v),
            None => Values::Null,
        }
    }
}

impl<'de, T> Deserialize<'de> for Values<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Into::into)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Body {
    pub name: String,
    pub description: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub license: String,
}

impl Body {
    pub fn create_from_toml(toml_path: &Path) -> Result<Body> {
        let deserialized_toml = create_project_from_toml_path(toml_path)?;
        let result = Body {
            name: deserialized_toml.package.name,
            description: deserialized_toml.package.description,
            version: deserialized_toml.package.version,
            authors: deserialized_toml.package.authors,
            license: deserialized_toml.package.license,
        };
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ContentType {
    #[serde(alias = "vm")]
    VM,
    #[serde(alias = "feature", alias = "FEATURE")]
    Feature,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: ContentType,
}
