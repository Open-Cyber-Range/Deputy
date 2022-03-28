use crate::repository::update_index_repository;
use anyhow::{Ok, Result};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub checksum: String,
}

#[derive(Debug)]
pub struct PackageFile(pub File);

impl PackageFile {
    fn save(&mut self, package_folder: String, name: String) -> Result<()> {
        let mut content_buffer: Vec<u8> = Vec::new();
        self.read_to_end(&mut content_buffer)?;
        let final_file_path: PathBuf = [package_folder, name].iter().collect();
        let mut file = File::create(final_file_path)?;
        file.write_all(&content_buffer)?;
        Ok(())
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
        self.file.save(package_folder, self.metadata.name.clone())?;
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
        metadata_length_bytes.copy_from_slice(&package_bytes[0..4]);
        let metadata_length = u32::from_le_bytes(metadata_length_bytes);
        let metadata_end = (4 + metadata_length) as usize;
        let metadata_bytes = &package_bytes[4..metadata_end];
        let metadata = PackageMetadata::try_from(metadata_bytes)?;

        let mut file_length_bytes: [u8; 4] = Default::default();
        file_length_bytes.copy_from_slice(&package_bytes[metadata_end..metadata_end + 4]);
        let file_length = u32::from_le_bytes(file_length_bytes);
        let file_start = metadata_end + 4;
        let file_end = file_start + (file_length) as usize;
        let file_bytes = &package_bytes[file_start..file_end];
        let file = PackageFile::try_from(file_bytes)?;

        Ok(Package { metadata, file })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::PathBuf};

    use super::Package;
    use crate::{
        package::{PackageFile, PackageMetadata},
        test::{
            create_readable_temporary_file, create_test_package, get_last_commit_message,
            initialize_test_repository, TEST_FILE_BYTES, TEST_METADATA_BYTES, TEST_PACKAGE_BYTES,
            TEST_PACKAGE_METADATA,
        },
    };
    use anyhow::{Ok, Result};
    use tempfile::tempdir;

    #[test]
    fn package_file_can_be_saved() -> Result<()> {
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let bytes = TEST_FILE_BYTES.clone();
        println!("{:?}", bytes);
        let mut package_file = PackageFile::try_from(&bytes as &[u8])?;

        let test_name = String::from("test-name");
        let expected_file_path: PathBuf =
            [package_folder.clone(), test_name.clone()].iter().collect();
        package_file.save(package_folder, test_name)?;

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

        assert_eq!(package.file.metadata()?.len(), 14);
        insta::assert_debug_snapshot!(package.metadata);
        Ok(())
    }
}
