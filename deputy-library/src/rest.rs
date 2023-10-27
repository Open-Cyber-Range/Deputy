use anyhow::Result;
use chrono::NaiveDateTime;
use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionRest {
    pub id: Uuid,
    pub version: String,
    pub description: String,
    pub license: String,
    pub is_yanked: bool,
    pub readme_html: String,
    pub package_size: u64,
    pub checksum: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl VersionRest {
    pub fn get_latest_package(packages: Vec<Self>) -> Option<Self> {
        packages
            .into_iter()
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    pub fn is_latest_version(
        uploadable_version: &str,
        packages: Vec<Self>,
    ) -> Result<Option<String>> {
        let latest_package = Self::get_latest_package(packages);
        match latest_package {
            Some(package) => {
                if uploadable_version.parse::<Version>()? > package.version.parse::<Version>()? {
                    Ok(None)
                } else {
                    Ok(Some(package.version))
                }
            }
            None => Ok(None),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerRest {
    pub email: String,
}
