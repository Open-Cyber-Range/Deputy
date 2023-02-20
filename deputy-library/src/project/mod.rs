pub(crate) mod enums;

use crate::project::enums::{Architecture, OperatingSystem};
use anyhow::{anyhow, Ok, Result};
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
    pub condition: Option<Condition>,
    pub event: Option<Event>,
    pub inject: Option<Inject>,
    pub picture: Option<Picture>,
    pub video: Option<Video>,
}

impl Project {
    pub fn validate_content(&mut self) -> Result<()> {
        match self.content.content_type {
            ContentType::VM => {
                if self.virtual_machine.is_none() {
                    return Err(anyhow!("Virtual machine package info not found"));
                } else if self.condition.is_some()
                    || self.feature.is_some()
                    || self.inject.is_some()
                    || self.event.is_some()
                    || self.picture.is_some()
                    || self.video.is_some()
                {
                    return Err(anyhow!(
                        "Content type (Virtual Machine) does not match package"
                    ));
                }
            }
            ContentType::Feature => {
                if self.feature.is_none() {
                    return Err(anyhow!("Feature package info not found"));
                } else if self.condition.is_some()
                    || self.virtual_machine.is_some()
                    || self.inject.is_some()
                    || self.event.is_some()
                    || self.picture.is_some()
                    || self.video.is_some()
                {
                    return Err(anyhow!("Content type (Feature) does not match package",));
                }
            }
            ContentType::Condition => {
                if self.condition.is_none() {
                    return Err(anyhow!("Condition package info not found"));
                } else if self.virtual_machine.is_some()
                    || self.feature.is_some()
                    || self.inject.is_some()
                    || self.event.is_some()
                    || self.picture.is_some()
                    || self.video.is_some()
                {
                    return Err(anyhow!("Content type (Condition) does not match package",));
                }
            }
            ContentType::Inject => {
                if self.inject.is_none() {
                    return Err(anyhow!("Inject package info not found"));
                } else if self.virtual_machine.is_some()
                    || self.feature.is_some()
                    || self.condition.is_some()
                    || self.event.is_some()
                    || self.picture.is_some()
                    || self.video.is_some()
                {
                    return Err(anyhow!("Content type (Inject) does not match package",));
                }
            }
            ContentType::Event => {
                if self.event.is_none() {
                    return Err(anyhow!("Event package info not found"));
                } else if self.virtual_machine.is_some()
                    || self.feature.is_some()
                    || self.condition.is_some()
                    || self.inject.is_some()
                    || self.picture.is_some()
                    || self.video.is_some()
                {
                    return Err(anyhow!("Content type (Event) does not match package",));
                }
            }
            ContentType::Picture => {
                if self.picture.is_none() {
                    return Err(anyhow!("Picture package info not found"));
                } else if self.virtual_machine.is_some()
                    || self.feature.is_some()
                    || self.condition.is_some()
                    || self.inject.is_some()
                    || self.event.is_some()
                    || self.video.is_some()
                {
                    return Err(anyhow!("Content type (Picture) does not match package",));
                }
            }
            ContentType::Video => {
                if self.video.is_none() {
                    return Err(anyhow!("Video package info not found"));
                }
                else if self.condition.is_some()
                    || self.feature.is_some()
                    || self.inject.is_some()
                    || self.event.is_some()
                    || self.picture.is_some()
                    || self.virtual_machine.is_some()
                {
                    return Err(anyhow!(
                        "Content type (Virtual Machine) does not match package"
                    ));
                }
            }
        }
        Ok(())
    }
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
pub enum FeatureType {
    #[serde(alias = "service", alias = "SERVICE")]
    Service,
    #[serde(alias = "configuration", alias = "CONFIGURATION")]
    Configuration,
    #[serde(alias = "artifact", alias = "ARTIFACT")]
    Artifact,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    #[serde(rename = "type", alias = "Type", alias = "TYPE")]
    pub feature_type: FeatureType,
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: Option<String>,
    #[serde(alias = "Assets", alias = "ASSETS")]
    pub assets: Vec<Vec<String>>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: String,
    #[serde(alias = "Assets", alias = "ASSETS")]
    pub assets: Vec<Vec<String>>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: String,
    #[serde(alias = "Assets", alias = "ASSETS")]
    pub assets: Vec<Vec<String>>,
    #[serde(alias = "Interval", alias = "INTERVAL")]
    pub interval: u32,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Inject {
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: String,
    #[serde(alias = "Assets", alias = "ASSETS")]
    pub assets: Vec<Vec<String>>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Picture {
    #[serde(alias = "file_path", alias = "FILE_PATH")]
    pub file_path: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    #[serde(alias = "file_path", alias = "FILE_PATH")]
    pub file_path: String,
}

pub fn create_project_from_toml_path(toml_path: &Path) -> Result<Project, anyhow::Error> {
    let mut toml_file = File::open(toml_path)?;
    let mut contents = String::new();
    toml_file.read_to_string(&mut contents)?;
    let deserialized_toml: Project = toml::from_str(&contents)?;
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
    pub readme: String,
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
            readme: deserialized_toml.package.readme,
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
    #[serde(alias = "condition", alias = "CONDITION")]
    Condition,
    #[serde(alias = "inject", alias = "INJECT")]
    Inject,
    #[serde(alias = "event", alias = "EVENT")]
    Event,
    #[serde(alias = "picture", alias = "PICTURE")]
    Picture,
    #[serde(alias = "video", alias = "VIDEO")]
    Video,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: ContentType,
}
