use crate::{
    archiver,
    project::Body,
    repository::{find_metadata_by_package_name, update_index_repository},
    StorageFolders,
};

use actix_http::error::PayloadError;
use actix_web::web::Bytes;
use anyhow::{anyhow, Result};
use futures::{Stream, StreamExt};
use git2::Repository;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{copy, BufReader, Read, Write},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    pin::Pin,
};
use tempfile::TempPath;
use tokio::fs::File as TokioFile;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub readme: String,
    pub license: String,
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

    pub fn validate_version(toml_path: &Path, registry_repository: &Repository) -> Result<()> {
        let package_body = Body::create_from_toml(toml_path)?;
        let metadata = PackageMetadata {
            name: package_body.name,
            version: package_body.version,
            checksum: "".to_string(),
            readme: "".to_string(),
            license: "".to_string(),
        };

        if let Ok(is_valid) = metadata.is_latest_version(registry_repository) {
            if !is_valid {
                return Err(anyhow::anyhow!(
                    "Package version on the server is either same or later"
                ));
            }
        } else {
            return Err(anyhow::anyhow!("Failed to validate versioning"));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct PackageFile(pub File, pub Option<TempPath>);

impl PackageFile {
    fn save(&self, package_folder: &str, name: &str, version: &str) -> Result<()> {
        let package_folder_path: PathBuf = [&package_folder, &name].iter().collect();
        fs::create_dir_all(&package_folder_path)?;
        let final_file_path: PathBuf = package_folder_path.join(version);
        let original_path: PathBuf = self
            .1
            .as_ref()
            .ok_or_else(|| anyhow!("Temporary file path not found"))?
            .to_path_buf();

        fs::copy(original_path, final_file_path)?;
        Ok(())
    }

    fn calculate_checksum(&mut self) -> Result<String> {
        let mut hasher = Sha256::new();
        copy(&mut self.0, &mut hasher)?;
        let hash_bytes = hasher.finalize();
        Ok(format!("{:x}", hash_bytes))
    }

    pub async fn from_stream(
        mut stream: impl Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static,
        is_end: bool,
    ) -> Result<Self> {
        let mut file = tempfile::NamedTempFile::new()?;
        match is_end {
            true => {
                while let Some(chunk) = stream.next().await {
                    file.write_all(&chunk?)?;
                    file.flush()?;
                }
            }
            false => {
                if let Some(chunk) = stream.next().await {
                    file.write_all(&chunk?)?;
                }
            }
        }

        let new_handler = file.reopen()?;
        let temporary_path = file.into_temp_path();

        Ok(PackageFile(new_handler, Some(temporary_path)))
    }

    pub(crate) fn get_size(&self) -> Result<u64> {
        Ok(self.metadata()?.len())
    }

    fn from_bytes(pointer: usize, bytes: &[u8]) -> Result<(Self, usize), anyhow::Error> {
        let mut file_length_bytes: [u8; 4] = Default::default();
        file_length_bytes.copy_from_slice(
            bytes
                .get(pointer..pointer + 4)
                .ok_or_else(|| anyhow::anyhow!("Could not find package toml length"))?,
        );
        let file_length = u32::from_le_bytes(file_length_bytes);
        let file_start = pointer + 4;
        let file_end = file_start + (file_length) as usize;
        let file_bytes = bytes
            .get(file_start..file_end)
            .ok_or_else(|| anyhow::anyhow!("Could not find package toml"))?;
        let package_toml = Self::try_from(file_bytes)?;
        Ok((package_toml, file_end))
    }
}

#[derive(Debug)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub file: PackageFile,
    pub package_toml: PackageFile,
    pub readme: Option<PackageFile>,
}

impl Package {
    pub fn new(
        metadata: PackageMetadata,
        package_toml: PackageFile,
        readme: Option<PackageFile>,
        file: PackageFile,
    ) -> Self {
        Self {
            metadata,
            package_toml,
            readme,
            file,
        }
    }

    pub fn save(&self, storage_folders: &StorageFolders, repository: &Repository) -> Result<()> {
        update_index_repository(repository, &self.metadata)?;
        self.file.save(
            &storage_folders.package_folder,
            &self.metadata.name,
            &self.metadata.version,
        )?;

        self.package_toml.save(
            &storage_folders.toml_folder,
            &self.metadata.name,
            &self.metadata.version,
        )?;

        if let Some(readme) = &self.readme {
            readme.save(
                &storage_folders.readme_folder,
                &self.metadata.name,
                &self.metadata.version,
            )?;
        }
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

    fn gather_metadata(toml_path: &Path, archive_path: &Path) -> Result<PackageMetadata> {
        let package_body = Body::create_from_toml(toml_path)?;
        let archive_file = File::open(&archive_path)?;
        let metadata = PackageMetadata {
            name: package_body.name,
            version: package_body.version,
            checksum: PackageFile(archive_file, None).calculate_checksum()?,
            readme: "".to_string(),
            license: package_body.license,
        };
        Ok(metadata)
    }

    pub fn from_file(
        optional_readme_path: Option<PathBuf>,
        package_toml_path: PathBuf,
        compression: u32,
    ) -> Result<Self> {
        let archive_path = archiver::create_package(&package_toml_path, compression)?;
        let metadata = Self::gather_metadata(&package_toml_path, &archive_path)?;
        let file = File::open(&archive_path)?;
        let package_toml = File::open(package_toml_path)?;
        let optional_readme = match optional_readme_path {
            Some(readme_path) => Some(PackageFile(File::open(readme_path)?, None)),
            None => None,
        };
        Ok(Package {
            metadata,
            file: PackageFile(file, None),
            package_toml: PackageFile(package_toml, None),
            readme: optional_readme,
        })
    }

    pub fn get_size(&self) -> Result<u64> {
        self.file.get_size()
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
        let temporary_path = file.into_temp_path();
        Ok(PackageFile(new_handler, Some(temporary_path)))
    }
}

impl TryFrom<Package> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(package: Package) -> Result<Self> {
        let mut payload: Vec<u8> = Vec::new();
        let package_file = package.file;
        let metadata_bytes = Vec::try_from(&package.metadata)?;
        payload.extend(metadata_bytes);
        let toml_bytes = Vec::try_from(package.package_toml)?;
        payload.extend(toml_bytes);

        let readme_bytes: Vec<u8> = match package.readme {
            Some(readme) => Vec::try_from(readme)?,
            None => {
                let mut readme_length_bytes = Vec::new();
                readme_length_bytes.extend_from_slice(&0_u32.to_le_bytes());
                readme_length_bytes
            }
        };
        payload.extend(readme_bytes);
        let file_bytes = Vec::try_from(package_file)?;
        payload.extend(file_bytes);

        Ok(payload)
    }
}
pub type PackageStream = Pin<Box<dyn Stream<Item = Result<Bytes, PayloadError>>>>;

pub trait Streamer {
    fn to_stream(self) -> PackageStream;
}
pub trait FromBytes
where
    Self: Sized,
{
    fn from_bytes(bytes: Bytes) -> Result<Self>;
}
impl Streamer for u64 {
    fn to_stream(self) -> PackageStream {
        let mut formatted_bytes = Vec::new();
        formatted_bytes.extend_from_slice(&self.to_le_bytes());
        let bytes = vec![Ok(Bytes::from(formatted_bytes))];
        Box::pin(futures::stream::iter(bytes))
    }
}

impl FromBytes for u64 {
    fn from_bytes(bytes: Bytes) -> Result<Self> {
        let mut length_bytes: [u8; 8] = Default::default();
        length_bytes.copy_from_slice(
            bytes
                .get(0..8)
                .ok_or_else(|| anyhow::anyhow!("Could not get bytes slice"))?,
        );
        Ok(u64::from_le_bytes(length_bytes))
    }
}

impl Streamer for TokioFile {
    fn to_stream(self) -> PackageStream {
        let stream = FramedRead::new(self, BytesCodec::new()).map(|bytes| match bytes {
            Ok(bytes) => Ok(bytes.freeze()),
            Err(err) => Err(PayloadError::Io(err)),
        });
        Box::pin(stream)
    }
}

impl Package {
    pub async fn to_stream(self) -> Result<PackageStream> {
        let mut metadata_bytes_with_lenght = Vec::try_from(&self.metadata)?;
        if metadata_bytes_with_lenght.len() < 4 {
            return Err(anyhow!("Metadata is too short"));
        }
        let metadata_bytes = metadata_bytes_with_lenght.drain(4..).collect::<Vec<_>>();
        let stream: PackageStream =
            Box::pin(futures::stream::iter(vec![Ok(Bytes::from(metadata_bytes))]));

        let archive_file = TokioFile::from(self.file.0);
        let archive_size = archive_file.metadata().await?.len();
        let toml_file = TokioFile::from(self.package_toml.0);
        let toml_size = toml_file.metadata().await?.len();

        let (optional_readme_file, readme_size) = match self.readme {
            Some(readme) => {
                let readme_file = TokioFile::from(readme.0);
                let readme_size = readme_file.metadata().await?.len();
                (Some(readme_file), readme_size)
            }
            None => (None, 0_u64),
        };

        let stream = stream
            .chain(toml_size.to_stream())
            .chain(toml_file.to_stream())
            .chain(readme_size.to_stream());

        if let Some(readme_file) = optional_readme_file {
            let stream = stream
                .chain(readme_file.to_stream())
                .chain(archive_size.to_stream())
                .chain(archive_file.to_stream());
            return Ok(stream.boxed_local());
        } else {
            let stream = stream
                .chain(archive_size.to_stream())
                .chain(archive_file.to_stream());
            return Ok(stream.boxed_local());
        }
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
            .ok_or_else(|| anyhow::anyhow!("Could not find metadata"))?;
        let metadata = PackageMetadata::try_from(metadata_bytes)?;

        let (package_toml, package_toml_end) =
            PackageFile::from_bytes(metadata_end, package_bytes)?;
        let (readme, readme_end) = PackageFile::from_bytes(package_toml_end, package_bytes)?;
        let (file, _) = PackageFile::from_bytes(readme_end, package_bytes)?;

        Ok(Package {
            metadata,
            package_toml,
            readme: Some(readme),
            file,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PackageFile, PackageMetadata};
    use crate::{
        test::{
            create_readable_temporary_file, create_test_package, get_last_commit_message,
            initialize_test_repository, TEST_FILE_BYTES, TEST_METADATA_BYTES,
            TEST_PACKAGE_METADATA,
        },
        StorageFolders,
    };
    use anyhow::{Ok, Result};
    use futures::StreamExt;
    use std::{fs::File, io::Read, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn package_file_can_be_saved() -> Result<()> {
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let bytes = TEST_FILE_BYTES.clone();

        let package_file = PackageFile::try_from(&bytes as &[u8])?;

        let test_name = "test-name";
        let test_version = "1.0.0";
        let expected_file_path: PathBuf = [
            package_folder.clone(),
            test_name.to_string(),
            test_version.to_string(),
        ]
        .iter()
        .collect();
        package_file.save(&package_folder, test_name, test_version)?;

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
        let test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let (repository_directory, repository) = initialize_test_repository();
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let toml_folder = tempdir()?.path().to_str().unwrap().to_string();
        let readme_folder = tempdir()?.path().to_str().unwrap().to_string();
        let storage_folders = StorageFolders {
            package_folder,
            toml_folder,
            readme_folder,
        };
        test_package.save(&storage_folders, &repository)?;

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
        let test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let (repository_directory, repository) = initialize_test_repository();
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let toml_folder = tempdir()?.path().to_str().unwrap().to_string();
        let readme_folder = tempdir()?.path().to_str().unwrap().to_string();
        let storage_folders = StorageFolders {
            package_folder,
            toml_folder,
            readme_folder,
        };
        test_package.save(&storage_folders, &repository)?;

        let mut new_test_package = create_test_package()?;
        new_test_package.metadata.version = String::from("4.0.0");
        new_test_package.save(&storage_folders, &repository)?;

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
        let test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let (repository_directory, repository) = initialize_test_repository();
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let toml_folder = tempdir()?.path().to_str().unwrap().to_string();
        let readme_folder = tempdir()?.path().to_str().unwrap().to_string();
        let storage_folders = StorageFolders {
            package_folder,
            toml_folder,
            readme_folder,
        };
        test_package.save(&storage_folders, &repository)?;

        let mut new_test_package = create_test_package()?;
        new_test_package.metadata.version = String::from("4.0.0");

        assert!(new_test_package.metadata.is_latest_version(&repository)?);
        target_directory.close()?;
        repository_directory.close()?;
        Ok(())
    }

    #[test]
    fn is_not_the_latest_package_version() -> Result<()> {
        let test_package = create_test_package()?;
        let target_directory = tempdir()?;
        let (repository_directory, repository) = initialize_test_repository();
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let toml_folder = tempdir()?.path().to_str().unwrap().to_string();
        let readme_folder = tempdir()?.path().to_str().unwrap().to_string();
        let storage_folders = StorageFolders {
            package_folder,
            toml_folder,
            readme_folder,
        };
        test_package.save(&storage_folders, &repository)?;

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
        let (temporary_file, path) = create_readable_temporary_file("Some content\n")?;
        let file_bytes = Vec::try_from(PackageFile(temporary_file, Some(path)))?;

        insta::assert_debug_snapshot!(file_bytes);
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

    #[actix_web::test]
    async fn package_is_converted_to_stream() -> Result<()> {
        let package = create_test_package()?;
        let mut byte_stream = package.to_stream().await?;
        let mut counter = 0;
        let mut bytes = Vec::new();
        while let Some(chunk) = byte_stream.next().await {
            let chunk = chunk.unwrap();
            bytes.append(&mut chunk.to_vec());
            counter += 1;
            if counter == 1 {
                PackageMetadata::try_from(bytes.as_slice())?;
            }
        }
        assert_eq!(counter, 7);
        insta::assert_debug_snapshot!(bytes);
        Ok(())
    }
}
