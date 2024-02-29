use crate::{commands::UnpackLevel, constants::PACKAGE_TOML};
use anyhow::{anyhow, Error, Ok, Result};
use awc::error::PayloadError;
use bytes::Bytes;
use colored::Colorize;
use deputy_library::archiver::{decompress_archive, unpack_archive};
use deputy_library::project::FeatureType;
use deputy_library::rest::{PackageWithVersionsRest, VersionRest};
use dialoguer::Select;
use futures::{Stream, StreamExt};
use human_bytes::human_bytes;
use std::fs;
use std::path::Path;
use std::{io::Write, path::PathBuf};
use tempfile::TempDir;

pub fn find_toml(current_path: &Path) -> Result<PathBuf> {
    let mut toml_path = current_path.join(PACKAGE_TOML);
    if toml_path.is_file() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        Ok(find_toml(&toml_path)?)
    } else {
        Err(anyhow!("Could not find package.toml"))
    }
}

pub fn print_success_message(message: &str) {
    println!("{} {}", "Success:".green(), message);
}

pub fn print_error_message(error: Error) {
    eprintln!("{} {}", "Error:".red(), error);
}

pub fn create_temporary_package_download_path(
    package_name: &str,
    package_version: &str,
) -> Result<(String, TempDir, TempDir)> {
    let temporary_directory_root = tempfile::Builder::new()
        .prefix("deputy-package-")
        .rand_bytes(12)
        .tempdir()?;
    let temporary_package_directory = tempfile::Builder::new()
        .prefix(package_name)
        .rand_bytes(0)
        .tempdir_in(&temporary_directory_root)?;
    let file_name = temporary_package_directory.path().join(package_version);

    Ok((
        file_name
            .as_path()
            .to_str()
            .ok_or_else(|| anyhow!("Failed to create temporary path"))?
            .to_string(),
        temporary_directory_root,
        temporary_package_directory,
    ))
}

pub fn get_download_target_name(
    unpack_level: &UnpackLevel,
    save_path: &str,
    name: &str,
    version: &str,
) -> PathBuf {
    match unpack_level {
        UnpackLevel::Raw => Path::new(save_path).join(format!("{}-{}.tar.gz", name, version)),
        UnpackLevel::Uncompressed => Path::new(save_path).join(format!("{}-{}.tar", name, version)),
        UnpackLevel::Regular => Path::new(save_path).join(format!("{}-{}", name, version)),
    }
}

pub fn unpack_package_file(
    temporary_file_path: &str,
    unpack_level: &UnpackLevel,
) -> Result<String> {
    match unpack_level {
        UnpackLevel::Raw => Ok(temporary_file_path.to_string()),
        UnpackLevel::Uncompressed => {
            let decompresesed_path = decompress_archive(&PathBuf::from(temporary_file_path))?;
            Ok(decompresesed_path
                .to_str()
                .ok_or_else(|| anyhow!("Failed to get decompressed path"))?
                .to_string())
        }
        UnpackLevel::Regular => {
            let decompresesed_path = decompress_archive(&PathBuf::from(temporary_file_path))?;
            let destination_path = PathBuf::from(format!("{}-dir", temporary_file_path));
            unpack_archive(&decompresesed_path, &destination_path)?;
            Ok(destination_path
                .to_str()
                .ok_or_else(|| anyhow!("Failed to get destination path"))?
                .to_string())
        }
    }
}

pub async fn create_file_from_stream(
    stream: &mut (impl Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static),
    file_path: &str,
) -> Result<()> {
    let mut file = std::fs::File::create(file_path)?;
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?)?;
        file.flush()?;
    }

    Ok(())
}

pub fn virtual_machine_fields() -> String {
    r#"
[virtual-machine]
accounts = [{ name = "", password = "" }]
operating_system = ""
architecture = ""
type = "OVA"
file_path = ""
"#
    .to_string()
}

pub fn feature_fields() -> String {
    let feature_types = FeatureType::all_variants();
    let feature_type_selection = Select::new()
        .with_prompt("Select feature type")
        .default(0)
        .items(&feature_types)
        .interact()
        .unwrap();

    let chosen_feature_type = feature_types[feature_type_selection];
    set_feature_type(chosen_feature_type)
}

pub fn set_assets_field() -> String {
    r#"assets = []
"#
    .to_string()
}

pub fn set_feature_type(feature_type: &str) -> String {
    let action_field: &str = if feature_type != "artifact" {
        r#"action = ""
"#
    } else {
        r#""#
    };

    format!(
        r#"
[feature]
type = "{}"
{action_field}restarts = false
"#,
        feature_type
    )
}

pub fn exercise_fields() -> String {
    r#"
[exercise]
file_path = ""
"#
    .to_string()
}

pub fn condition_fields() -> String {
    r#"
[condition]
action = ""
interval = 30
"#
    .to_string()
}

pub fn inject_fields() -> String {
    r#"
[inject]
action = ""
restarts = false
"#
    .to_string()
}

pub fn event_fields() -> String {
    r#"
[event]
file_path = ""
"#
    .to_string()
}

pub fn malware_fields() -> String {
    r#"
[malware]
action = ""
"#
    .to_string()
}

pub fn banner_fields() -> String {
    r#"
[banner]
file_path = ""
"#
    .to_string()
}

pub fn other_fields() -> String {
    r#"
[other]
"#
    .to_string()
}

pub fn create_default_readme(package_dir: &str) -> Result<()> {
    let readme_content = r#"This is a readme file"#;

    fs::write(format!("{}/README.md", package_dir), readme_content)?;

    Ok(())
}

pub fn print_latest_version_package_list_entry(package: &PackageWithVersionsRest) -> Result<()> {
    let latest_version = VersionRest::get_latest_package(package.versions.clone())?
        .map(|version_rest| version_rest.version.to_owned())
        .ok_or_else(|| anyhow!("Package missing version"))?;

    println!(
        "{name}/{type}, {latest_version}",
        name = package.name.green(),
        type = package.package_type
    );

    Ok(())
}

pub fn print_package_list_entry(package: &PackageWithVersionsRest) -> Result<()> {
    let mut versions = package.versions.clone();
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    for version in versions {
        println!(
            "{name}/{type}, {version}",
            name = package.name.green(),
            type = package.package_type,
            version = version.version,
        );
    }
    Ok(())
}

pub fn print_package_info(package: &PackageWithVersionsRest, package_version: &VersionRest) {
    println!("Name: {}", package.name);
    println!("Version: {}", package_version.version);
    println!("Type: {}", package.package_type);
    println!("License: {}", package_version.license);
    println!("Description: {}", package_version.description);
    println!(
        "Package Size: {}",
        human_bytes(package_version.package_size as f64)
    );
    println!("Created at: {}", package_version.created_at);
    println!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tempfile::{Builder, TempDir};

    #[test]
    fn successfully_found_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;

        assert!(find_toml(temp_dir.path())?.is_file());
        package_toml.close()?;
        temp_dir.close()?;
        Ok(())
    }

    #[test]
    fn creates_temporary_file_path() -> Result<()> {
        let package_name = "Shakespeare";
        let version = "0.5.0";
        let (temporary_path, _temporary_parent_directory, temporary_directory) =
            create_temporary_package_download_path(package_name, version)?;

        let mut package_path = temporary_path.split('/').rev();
        assert_eq!(package_path.next(), Some(version));
        assert_eq!(package_path.next(), Some(package_name));

        temporary_directory.close()?;
        Ok(())
    }
}
