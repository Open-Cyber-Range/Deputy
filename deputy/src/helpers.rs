use crate::{commands::UnpackLevel, constants::PACKAGE_TOML};
use anyhow::{anyhow, Error, Ok, Result};
use awc::error::PayloadError;
use bytes::Bytes;
use colored::Colorize;
use deputy_library::archiver::{decompress_archive, unpack_archive};
use deputy_library::project::FeatureType;
use dialoguer::{Input, Select};
use futures::{Stream, StreamExt};
use std::fs;
use std::path::Path;
use std::{io::Write, path::PathBuf};
use tempfile::TempDir;

pub fn find_toml(current_path: PathBuf) -> Result<PathBuf> {
    let mut toml_path = current_path.join(PACKAGE_TOML);
    if toml_path.is_file() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        Ok(find_toml(toml_path)?)
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
    let default_account: String = Input::new()
        .with_prompt("Default Account Name")
        .interact_text()
        .unwrap();

    format!(
        r#"
[virtual-machine]
accounts = [{{ name = "{default_account}", password = "" }}]
default_account = "{default_account}"
operating_system = ""
architecture = ""
type = "OVA"
file_path = ""
readme_path = "README.md"
"#,
    )
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

pub fn set_feature_type(feature_type: &str) -> String {
    let action = if feature_type == "service" {
        let action_input: String = Input::new()
            .with_prompt("action")
            .default("".to_string())
            .interact_text()
            .unwrap();
        format!(
            r#"action = "{}"
"#,
            action_input
        )
    } else {
        "".to_string()
    };

    format!(
        r#"
[feature]
type = "{}"
{}"#,
        feature_type, action
    )
}

pub fn exercise_fields() -> String {
    let file_path: String = Input::new()
        .with_prompt("Exercise File Path")
        .interact_text()
        .unwrap();

    format!(
        r#"
[exercise]
file_path = "{path}"
"#,
        path = file_path
    )
}

pub fn condition_fields() -> String {
    let action_path: &str = "path to executable";

    format!(
        r#"[condition]
action = "{action_path}"
interval = "30"

"#,
        action_path = action_path,
    )
}

pub fn inject_fields() -> String {
    r#"[inject]
action = "path to executable"

"#
    .to_string()
}

pub fn event_fields() -> String {
    r#"[event]
action = "path to executable"

"#
    .to_string()
}

pub fn malware_fields() -> String {
    r#"[malware]
action = "path to executable"

"#
    .to_string()
}

pub fn create_default_readme(package_dir: &str) -> Result<()> {
    let readme_content = r#"# My Package

This is the README file for my package.

## Installation

Provide installation instructions here.

## Usage

Provide usage instructions here.
"#;

    fs::write(format!("{}/README.md", package_dir), readme_content)?;

    Ok(())
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

        assert!(find_toml(temp_dir.path().to_path_buf())?.is_file());
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
