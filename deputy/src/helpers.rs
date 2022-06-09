use crate::constants::PACKAGE_TOML;
use anyhow::{anyhow, Error, Result};
use awc::error::PayloadError;
use bytes::Bytes;
use colored::Colorize;
use futures::{Stream, StreamExt};
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
) -> Result<(String, TempDir)> {
    let temporary_directory = tempfile::Builder::new()
        .prefix(package_name)
        .rand_bytes(0)
        .tempdir()?;
    let file_name = temporary_directory.path().join(package_version);

    Ok((
        file_name
            .as_path()
            .to_str()
            .ok_or_else(|| anyhow!("Failed to create temporary path"))?
            .to_string(),
        temporary_directory,
    ))
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
        let (temporary_path, temporary_directory) =
            create_temporary_package_download_path(package_name, version)?;
        insta::assert_display_snapshot!(temporary_path);
        temporary_directory.close()?;
        Ok(())
    }
}
