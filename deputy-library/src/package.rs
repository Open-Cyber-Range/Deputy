use crate::{
    archiver,
    client::{find_toml, upload_package},
    configuration::Configuration,
    constants::{PACKAGE_TOML, PACKAGE_UPLOAD_PATH},
    project::Body,
    repository::{find_metadata_by_package_name, update_index_repository},
};
use anyhow::{anyhow, Ok, Result};
use git2::Repository;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{copy, BufReader, Read, Seek as _, SeekFrom, Write},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub checksum: String,
}

impl PackageMetadata {
    fn get_latest_metadata(name: &str, repository: &Repository) -> Result<Option<PackageMetadata>> {
        let metadata_list = find_metadata_by_package_name(repository, name)?;

        let latest_metadata = metadata_list
            .iter()
            .max_by(|a, b| a.version.cmp(&b.version))
            .cloned();

        Ok(latest_metadata)
    }

    pub fn is_latest_version(&self, repository: &Repository) -> Result<bool> {
        if let Some(current_latest_version) =
            PackageMetadata::get_latest_metadata(&self.name, repository)?
        {
            return Ok(self.version.parse::<Version>()?
                > current_latest_version.version.parse::<Version>()?);
        }
        Ok(true)
    }

    pub fn gather_metadata(toml_path: PathBuf, archive_path: &Path) -> Result<PackageMetadata> {
        let package_body = Body::create_from_toml(toml_path)?;
        let archive_file = File::open(&archive_path)?;
        let metadata = PackageMetadata {
            name: package_body.name,
            version: package_body.version,
            checksum: PackageFile(archive_file).calculate_checksum()?,
        };
        Ok(metadata)
    }
}

#[derive(Debug)]
pub struct PackageFile(pub File);

impl PackageFile {
    fn save(&mut self, package_folder: String, name: String, version: String) -> Result<()> {
        let mut content_buffer: Vec<u8> = Vec::new();
        self.seek(SeekFrom::Start(0))?;
        self.read_to_end(&mut content_buffer)?;
        let package_folder_path: PathBuf = [package_folder, name].iter().collect();
        fs::create_dir_all(package_folder_path.clone())?;
        let final_file_path: PathBuf = package_folder_path.join(version);
        let mut file = File::create(final_file_path)?;
        file.write_all(&content_buffer)?;
        Ok(())
    }

    fn calculate_checksum(&mut self) -> Result<String> {
        let mut hasher = Sha256::new();
        copy(&mut self.0, &mut hasher)?;
        let hash_bytes = hasher.finalize();
        Ok(format!("{:x}", hash_bytes))
    }
}

#[derive(Debug)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub file: PackageFile,
}

impl Package {
    pub fn save(&mut self, package_folder: String, repository: &Repository) -> Result<()> {
        update_index_repository(repository, &self.metadata)?;
        self.file.save(
            package_folder,
            self.metadata.name.clone(),
            self.metadata.version.clone(),
        )?;
        Ok(())
    }

    pub fn validate_checksum(&mut self) -> Result<()> {
        let calculated = self.file.calculate_checksum()?;
        if calculated != self.metadata.checksum {
            return Err(anyhow!(
                "Checksum mismatch. Calculated: {:?}, Expected: {:?}",
                calculated,
                self.metadata.checksum
            ));
        }
        Ok(())
    }
}

impl Deref for PackageFile {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PackageFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<&PackageMetadata> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(package_metadata: &PackageMetadata) -> Result<Self> {
        let mut formatted_bytes = Vec::new();
        let string = serde_json::to_string(&package_metadata)?;
        let main_bytes = string.as_bytes();
        let length: u32 = main_bytes.len().try_into()?;

        formatted_bytes.extend_from_slice(&length.to_le_bytes());
        formatted_bytes.extend_from_slice(main_bytes);

        Ok(formatted_bytes)
    }
}

impl TryFrom<&[u8]> for PackageMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata_bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(metadata_bytes)?)
    }
}

impl TryFrom<PackageFile> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(package_file: PackageFile) -> Result<Self> {
        let mut formatted_bytes = Vec::new();
        let file = package_file.0;
        let mut reader = BufReader::new(file);
        let mut file_buffer = Vec::new();
        reader.read_to_end(&mut file_buffer)?;

        let length: u32 = file_buffer.len().try_into()?;
        formatted_bytes.extend_from_slice(&length.to_le_bytes());
        formatted_bytes.extend(file_buffer);

        Ok(formatted_bytes)
    }
}

impl TryFrom<&[u8]> for PackageFile {
    type Error = anyhow::Error;

    fn try_from(metadata_bytes: &[u8]) -> Result<Self> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(metadata_bytes)?;
        file.flush()?;

        let new_handler = file.reopen()?;
        Ok(PackageFile(new_handler))
    }
}

impl<'a> TryFrom<Package> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(package: Package) -> Result<Self> {
        let mut payload: Vec<u8> = Vec::new();
        let package_file = package.file;

        let metadata_bytes = Vec::try_from(&package.metadata)?;
        payload.extend(metadata_bytes);
        let file_bytes = Vec::try_from(package_file)?;
        payload.extend(file_bytes);

        Ok(payload)
    }
}

impl TryFrom<&[u8]> for Package {
    type Error = anyhow::Error;

    fn try_from(package_bytes: &[u8]) -> Result<Self> {
        let mut metadata_length_bytes: [u8; 4] = Default::default();
        metadata_length_bytes.copy_from_slice(
            package_bytes
                .get(0..4)
                .ok_or_else(|| anyhow::anyhow!("Could not find metadata length"))?,
        );
        let metadata_length = u32::from_le_bytes(metadata_length_bytes);
        let metadata_end = (4 + metadata_length) as usize;
        let metadata_bytes = package_bytes
            .get(4..metadata_end)
            .ok_or_else(|| anyhow::anyhow!("Could not din metadata"))?;
        let metadata = PackageMetadata::try_from(metadata_bytes)?;

        let mut file_length_bytes: [u8; 4] = Default::default();
        file_length_bytes.copy_from_slice(
            package_bytes
                .get(metadata_end..metadata_end + 4)
                .ok_or_else(|| anyhow::anyhow!("Could not find file length"))?,
        );
        let file_length = u32::from_le_bytes(file_length_bytes);
        let file_start = metadata_end + 4;
        let file_end = file_start + (file_length) as usize;
        let file_bytes = package_bytes
            .get(file_start..file_end)
            .ok_or_else(|| anyhow::anyhow!("Could not find file"))?;
        let file = PackageFile::try_from(file_bytes)?;

        Ok(Package { metadata, file })
    }
}

pub fn create_package_from_toml(toml_path: PathBuf) -> Result<Package> {
    let package_root = toml_path
        .parent()
        .ok_or_else(|| anyhow!("Directory error"))?;
    let archive_path = archiver::create_package(package_root.to_path_buf())?;
    let metadata = PackageMetadata::gather_metadata(toml_path, &archive_path)?;
    let file = File::open(&archive_path)?;
    let package = Package {
        metadata,
        file: PackageFile(file),
    };
    Ok(package)
}

pub async fn create_and_send_package_file(
    execution_directory: PathBuf,
    client: reqwest::Client,
) -> Result<()> {
    let package_toml = PathBuf::from(PACKAGE_TOML);
    let toml_path = [&execution_directory, &package_toml].iter().collect();
    let toml_path = find_toml(toml_path)?;
    let package = create_package_from_toml(toml_path)?;
    let package_bytes = Vec::try_from(package)?;
    let api = Configuration::get_configuration()?.repository.repositories[0]
        .api
        .to_owned();
    let put_uri = api + PACKAGE_UPLOAD_PATH;
    upload_package(put_uri.as_str(), package_bytes, client).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::PathBuf};

    use super::{Package, PackageFile, PackageMetadata};
    use crate::test::{
        create_readable_temporary_file, create_test_package, get_last_commit_message,
        initialize_test_repository, TEST_FILE_BYTES, TEST_METADATA_BYTES, TEST_PACKAGE_BYTES,
        TEST_PACKAGE_METADATA,
    };
    use anyhow::{Ok, Result};
    use tempfile::tempdir;

    #[test]
    fn package_file_can_be_saved() -> Result<()> {
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let bytes = TEST_FILE_BYTES.clone();

        let mut package_file = PackageFile::try_from(&bytes as &[u8])?;

        let test_name = String::from("test-name");
        let test_version = String::from("1.0.0");
        let expected_file_path: PathBuf = [
            package_folder.clone(),
            test_name.clone(),
            test_version.clone(),
        ]
        .iter()
        .collect();
        package_file.save(package_folder, test_name, test_version)?;

        let mut created_file = File::open(expected_file_path)?;
        assert!(created_file.metadata().unwrap().is_file());

        let mut file_content = Vec::new();
        created_file.read_to_end(&mut file_content)?;
        insta::assert_debug_snapshot!(file_content);

        target_directory.close()?;
        Ok(())
    }

    #[test]
    fn package_can_be_saved() -> Result<()> {
        let mut test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let (repository_directory, repository) = initialize_test_repository();
        test_package.save(package_folder, &repository)?;

        assert!(target_directory.path().join("some-package-name").exists());
        assert_eq!(
            get_last_commit_message(&repository),
            "Adding package: some-package-name, version: 0.1.0"
        );
        target_directory.close()?;
        repository_directory.close()?;
        Ok(())
    }

    #[test]
    fn latest_package_metadata_is_found() -> Result<()> {
        let mut test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let (repository_directory, repository) = initialize_test_repository();
        test_package.save(package_folder.clone(), &repository)?;

        let mut new_test_package = create_test_package()?;
        new_test_package.metadata.version = String::from("4.0.0");
        new_test_package.save(package_folder, &repository)?;

        insta::assert_debug_snapshot!(PackageMetadata::get_latest_metadata(
            &test_package.metadata.name,
            &repository
        )?);
        target_directory.close()?;
        repository_directory.close()?;
        Ok(())
    }

    #[test]
    fn is_the_latest_package_version_if_is_the_first() -> Result<()> {
        let test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let (repository_directory, repository) = initialize_test_repository();

        assert!(test_package.metadata.is_latest_version(&repository)?);
        target_directory.close()?;
        repository_directory.close()?;
        Ok(())
    }

    #[test]
    fn is_the_latest_package_version() -> Result<()> {
        let mut test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let (repository_directory, repository) = initialize_test_repository();
        test_package.save(package_folder, &repository)?;

        let mut new_test_package = create_test_package()?;
        new_test_package.metadata.version = String::from("4.0.0");

        assert!(new_test_package.metadata.is_latest_version(&repository)?);
        target_directory.close()?;
        repository_directory.close()?;
        Ok(())
    }

    #[test]
    fn is_not_the_latest_package_version() -> Result<()> {
        let mut test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let (repository_directory, repository) = initialize_test_repository();
        test_package.save(package_folder, &repository)?;

        let mut new_test_package = create_test_package()?;
        new_test_package.metadata.version = String::from("0.0.1");

        assert!(!new_test_package.metadata.is_latest_version(&repository)?);
        target_directory.close()?;
        repository_directory.close()?;
        Ok(())
    }

    #[test]
    fn metadata_is_converted_to_bytes() -> Result<()> {
        let package_metadata: &PackageMetadata = &TEST_PACKAGE_METADATA;
        let metadata_bytes = Vec::try_from(package_metadata)?;

        insta::assert_debug_snapshot!(metadata_bytes);
        Ok(())
    }

    #[test]
    fn metadata_is_parsed_from_bytes() -> Result<()> {
        let bytes = TEST_METADATA_BYTES.clone();

        let metadata = PackageMetadata::try_from(&bytes as &[u8])?;
        insta::assert_debug_snapshot!(metadata);
        Ok(())
    }

    #[test]
    fn file_is_converted_to_bytes() -> Result<()> {
        let temporary_file = create_readable_temporary_file("Some content\n")?;
        let metadata_bytes = Vec::try_from(PackageFile(temporary_file))?;

        insta::assert_debug_snapshot!(metadata_bytes);
        Ok(())
    }

    #[test]
    fn file_is_parsed_from_byte() -> Result<()> {
        let bytes = TEST_FILE_BYTES.clone();

        let mut package_file = PackageFile::try_from(&bytes as &[u8])?.0;
        let mut file_content = Vec::new();
        package_file.read_to_end(&mut file_content)?;
        assert_eq!(file_content, bytes);
        assert_eq!(package_file.metadata()?.len(), 17);
        Ok(())
    }

    #[test]
    fn package_is_converted_to_bytes() -> Result<()> {
        let package = create_test_package()?;
        let package_bytes = Vec::try_from(package)?;
        insta::assert_debug_snapshot!(package_bytes);

        Ok(())
    }

    #[test]
    fn package_is_parsed_from_bytes() -> Result<()> {
        let bytes = TEST_PACKAGE_BYTES.clone();
        let package = Package::try_from(&bytes as &[u8])?;

        assert_eq!(package.file.metadata()?.len(), 961);
        insta::assert_debug_snapshot!(package.metadata);
        Ok(())
    }
}
