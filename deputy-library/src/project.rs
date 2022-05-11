use anyhow::{Ok, Result};
use std::{fs::File, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub package: Body,
    pub content: Content,
    #[serde(rename = "virtual-machine")]
    pub virtual_machine: Option<VirtualMachine>,
}
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct VirtualMachine {
    #[serde(default)]
    pub operating_system: String,
    #[serde(default)]
    pub architecture: String,
}
fn create_project_from_toml_path(toml_path: PathBuf) -> Result<Project, anyhow::Error> {
    let mut toml_file = File::open(&toml_path)?;
    let mut contents = String::new();
    toml_file.read_to_string(&mut contents)?;
    let deserialized_toml: Project = toml::from_str(&*contents)?;
    Ok(deserialized_toml)
}
impl VirtualMachine {
    pub fn create_from_toml(toml_path: PathBuf) -> Result<Option<VirtualMachine>> {
        let deserialized_toml = create_project_from_toml_path(toml_path)?;

        match deserialized_toml.content.content_type {
            ContentType::VM => {
                let virtual_machine = deserialized_toml.virtual_machine.unwrap_or_default();
                let result = VirtualMachine {
                    operating_system: match virtual_machine.operating_system.trim() {
                        "" => "Unknown".to_string(),
                        value => value.to_string(),
                    },
                    architecture: match virtual_machine.architecture.trim() {
                        "" => "Unknown".to_string(),
                        value => value.to_string(),
                    },
                };
                Ok(Some(result))
            } /* Unreachable pattern since VM is currently the only ContentType
              _ => Ok(None),
               */
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> VirtualMachine {
        VirtualMachine {
            operating_system: "Unknown".to_string(),
            architecture: "Unknown".to_string(),
        }
    }
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
    pub sub_type: SubType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    #[serde(alias = "vm")]
    VM,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SubType {
    #[serde(alias = "packer")]
    Packer,
}
