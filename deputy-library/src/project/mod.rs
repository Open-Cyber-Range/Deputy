pub(crate) mod enums;

use crate::project::enums::{Architecture, OperatingSystem};
use anyhow::{anyhow, Ok, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::{fs::File, io::Read, path::Path};

use self::enums::VirtualMachineType;

pub fn create_project_from_toml_path(toml_path: &Path) -> Result<Project, anyhow::Error> {
    let mut toml_file = File::open(toml_path)?;
    let mut contents = String::new();
    toml_file.read_to_string(&mut contents)?;
    let deserialized_toml: Project = toml::from_str(&contents)?;
    Ok(deserialized_toml)
}

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
    pub malware: Option<Malware>,
    pub exercise: Option<Exercise>,
    pub other: Option<Other>,
}

impl Project {
    pub fn validate_assets(&self) -> Result<()> {
        let package_type: String = self.content.content_type.clone().try_into()?;
        if let Some(assets) = &self.package.assets {
            if assets.is_empty() {
                return Err(anyhow!(
                    "Assets are required for '{package_type}' package type"
                ));
            }
            for (index, asset) in assets.iter().enumerate() {
                if asset.len() < 2 {
                    return Err(anyhow!(
                        "Package.assets[{index}] is invalid.
                        Expected format: [\"relative source path\", \"absolute destination path\", optional file permissions]
                        E.g. [\"files/file.sh\", \"/usr/local/bin/renamed_file.sh\", 755] or [\"files/file.sh\", \"/usr/local/bin/\"]"
                    ));
                }
            }
        } else {
            return Err(anyhow!(
                "Assets are required for '{package_type}' package type"
            ));
        }
        Ok(())
    }

    pub fn validate_content(&mut self) -> Result<()> {
        let mut content_types = vec![
            self.virtual_machine.as_ref().map(|_| ContentType::VM),
            self.feature.as_ref().map(|_| ContentType::Feature),
            self.condition.as_ref().map(|_| ContentType::Condition),
            self.inject.as_ref().map(|_| ContentType::Inject),
            self.event.as_ref().map(|_| ContentType::Event),
        ];
        content_types.retain(|potential_content_types| potential_content_types.is_some());
        if content_types.len() > 1 {
            return Err(anyhow!(
                "Multiple content types per package are not supported"
            ));
        }

        match self.content.content_type {
            ContentType::VM => {
                if self.virtual_machine.is_none() {
                    return Err(anyhow!("Virtual machine package info not found"));
                }
            }
            ContentType::Feature => {
                if self.feature.is_none() {
                    return Err(anyhow!("Feature package info not found"));
                }
                self.validate_assets()?;
            }
            ContentType::Condition => {
                if self.condition.is_none() {
                    return Err(anyhow!("Condition package info not found"));
                }
                self.validate_assets()?;
            }
            ContentType::Inject => {
                if self.inject.is_none() {
                    return Err(anyhow!("Inject package info not found"));
                }
                self.validate_assets()?;
            }
            ContentType::Event => {
                if self.event.is_none() {
                    return Err(anyhow!("Event package info not found"));
                }
                self.validate_assets()?;
            }
            ContentType::Malware => {
                if self.malware.is_none() {
                    return Err(anyhow!("Malware package info not found"));
                }
                self.validate_assets()?;
            }
            ContentType::Exercise => {
                if self.exercise.is_none() {
                    return Err(anyhow!("Exercise package info not found"));
                }
                self.validate_assets()?;
            }
            ContentType::Other => {
                if self.other.is_none() {
                    return Err(anyhow!("Other package info not found"));
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
    pub file_path: String,
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
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: String,
    #[serde(alias = "Interval", alias = "INTERVAL")]
    pub interval: u32,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Inject {
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Malware {
    #[serde(alias = "Action", alias = "ACTION")]
    pub action: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Exercise {
    #[serde(alias = "File_path", alias = "FILE_PATH")]
    pub file_path: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Other {}

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
    pub assets: Option<Vec<Vec<String>>>,
}

impl Body {
    pub fn create_from_toml(toml_path: &Path) -> Result<Body> {
        let deserialized_toml = create_project_from_toml_path(toml_path)?;
        Ok(Body {
            name: deserialized_toml.package.name,
            description: deserialized_toml.package.description,
            version: deserialized_toml.package.version,
            authors: deserialized_toml.package.authors,
            license: deserialized_toml.package.license,
            readme: deserialized_toml.package.readme,
            assets: deserialized_toml.package.assets,
        })
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
    #[serde(alias = "malware", alias = "MALWARE")]
    Malware,
    #[serde(alias = "exercise", alias = "EXERCISE")]
    Exercise,
    #[serde(alias = "other", alias = "OTHER")]
    Other,
}

impl TryFrom<ContentType> for String {
    type Error = anyhow::Error;

    fn try_from(content_type: ContentType) -> Result<String, Self::Error> {
        match content_type {
            ContentType::VM => Ok("VM".to_string()),
            ContentType::Feature => Ok("Feature".to_string()),
            ContentType::Condition => Ok("Condition".to_string()),
            ContentType::Inject => Ok("Inject".to_string()),
            ContentType::Event => Ok("Event".to_string()),
            ContentType::Malware => Ok("Malware".to_string()),
            ContentType::Exercise => Ok("Exercise".to_string()),
            ContentType::Other => Ok("Other".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
#[serde(tag = "type", content = "value")]
pub enum Preview {
    #[serde(alias = "picture", alias = "PICTURE")]
    Picture(Vec<String>),
    #[serde(alias = "video", alias = "VIDEO")]
    Video(Vec<String>),
    #[serde(alias = "code", alias = "CODE")]
    Code(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: ContentType,
    #[serde(alias = "preview", alias = "PREVIEW")]
    pub preview: Option<Vec<Preview>>,
}
