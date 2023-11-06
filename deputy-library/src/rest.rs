use anyhow::Result;
use chrono::NaiveDateTime;
use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageWithVersionsRest {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub versions: Vec<VersionRest>,
}

impl PackageWithVersionsRest {
    pub fn remove_yanked_versions(packages: &mut Vec<Self>) {
        for package in packages.iter_mut() {
            package.versions.retain(|version| !version.is_yanked);
        }

        packages.retain(|package| !package.versions.is_empty());
    }
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackagesWithVersionsAndPagesRest {
    pub packages: Vec<PackageWithVersionsRest>,
    pub total_pages: i64,
}
