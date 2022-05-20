use crate::constants::{Architecture, OperatingSystem};
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub package: Body,
    pub content: Content,
    #[serde(rename = "virtual-machine")]
    pub virtual_machine: Option<VirtualMachine>,
}
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct VirtualMachine {
    #[serde(default = "default_os")]
    pub operating_system: OperatingSystem,
    #[serde(default = "default_architecture")]
    pub architecture: Architecture,
}
pub fn create_project_from_toml_path(toml_path: PathBuf) -> Result<Project, anyhow::Error> {
    let mut toml_file = File::open(&toml_path)?;
    let mut contents = String::new();
    toml_file.read_to_string(&mut contents)?;
    let deserialized_toml: Project = toml::from_str(&*contents)?;
    Ok(deserialized_toml)
}

pub fn default_os() -> OperatingSystem {
    OperatingSystem::Unknown
}

pub fn default_architecture() -> Architecture {
    Architecture::Unknown
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Body {
    pub name: String,
    pub description: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
}

impl Body {
    pub fn create_from_toml(toml_path: PathBuf) -> Result<Body> {
        let deserialized_toml = create_project_from_toml_path(toml_path)?;
        let result = Body {
            name: deserialized_toml.package.name,
            description: deserialized_toml.package.description,
            version: deserialized_toml.package.version,
            authors: deserialized_toml.package.authors,
        };
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: ContentType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    #[serde(alias = "vm")]
    VM,
}
