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
