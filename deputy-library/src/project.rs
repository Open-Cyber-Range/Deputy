use anyhow::{Ok, Result};
use std::path::PathBuf;
use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub package: Body,
    pub content: Content,
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
        let mut toml_file = File::open(&toml_path)?;
        let mut contents = String::new();
        toml_file.read_to_string(&mut contents)?;

        let deserialized_toml: Project = toml::from_str(&*contents)?;
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
