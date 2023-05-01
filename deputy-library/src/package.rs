use crate::{
    archiver::{self, ArchiveStreamer},
    project::Body,
};
use actix_http::error::PayloadError;
use actix_web::web::Bytes;
use anyhow::{anyhow, Result};
use futures::{Stream, StreamExt};
use pulldown_cmark::{html::push_html, Parser};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{copy, Write},
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
    pub description: String,
    pub license: String,
    pub readme_path: String,
    pub checksum: String,
}

impl PackageMetadata {
    pub async fn readme_html(&self, package_base_path: PathBuf) -> Result<Option<String>> {
        let readme_path = self.readme_path.clone();
        let package_path = package_base_path.join(&self.name).join(&self.version);

        if let Some(mut archive_stream) =
            ArchiveStreamer::try_new(package_path, readme_path.into())?
        {
            let mut readme_markdown_string = String::new();
            while let Some(bytes) = archive_stream.next().await {
                let bytes =
                    bytes.map_err(|error| anyhow!("Failed to read archive stream: {error}"))?;
                readme_markdown_string.push_str(&String::from_utf8(bytes.to_vec())?);
            }
            let parser = Parser::new(&readme_markdown_string);
            let mut html_string = String::new();
            push_html(&mut html_string, parser);
            return Ok(Some(html_string));
        }
        Ok(None)
    }

    pub async fn from_stream(
        mut stream: impl Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static,
    ) -> Result<(Self, PackageStream)> {
        let mut bytes = Vec::new();
        let mut metadata_length: Option<u32> = None;
        let mut reminder = Vec::new();
        while let Some(chunk) = stream.next().await {
            let mut chunk = chunk?;
            if metadata_length.is_none() {
                if chunk.len() < 4 {
                    return Err(anyhow!(
                        "Chunk length is less than 4 bytes. Length: {:?}",
                        chunk.len()
                    ));
                }
                let mut u32_as_bytes = [0u8; 4];
                u32_as_bytes.copy_from_slice(&chunk[0..4]);
                metadata_length = Some(u32::from_le_bytes(u32_as_bytes));
                chunk = chunk.slice(4..);
            }
            if let Some(metadata_lenght) = metadata_length {
                if bytes.len() + chunk.len() < metadata_lenght as usize {
                    bytes.extend_from_slice(&chunk);
                } else if bytes.len() < metadata_lenght as usize {
                    let (last_metadata_bytes, first_file_bytes) =
                        chunk.split_at(metadata_lenght as usize - bytes.len());
                    bytes.extend_from_slice(last_metadata_bytes);
                    reminder.extend_from_slice(first_file_bytes);
                    break;
                } else {
                    break;
                }
            }
        }
        let metadata: PackageMetadata = serde_json::from_slice(&bytes)?;
        let reminder = vec![Ok(Bytes::from(reminder))];
        let new_body = Box::pin(futures::stream::iter(reminder));
        let stream = new_body.chain(stream).boxed_local();
        Ok((metadata, stream))
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

    #[cfg(feature = "test")]
    pub fn calculate_checksum(&mut self) -> Result<String> {
        let mut hasher = Sha256::new();
        copy(&mut self.0, &mut hasher)?;
        let hash_bytes = hasher.finalize();
        Ok(format!("{hash_bytes:x}",))
    }

    #[cfg(not(feature = "test"))]
    fn calculate_checksum(&mut self) -> Result<String> {
        let mut hasher = Sha256::new();
        copy(&mut self.0, &mut hasher)?;
        let hash_bytes = hasher.finalize();
        Ok(format!("{hash_bytes:x}",))
    }

    pub async fn from_stream(mut stream: PackageStream) -> Result<Self> {
        let mut file = tempfile::NamedTempFile::new()?;
        let mut file_size: Option<u64> = None;
        let mut intermediate_buffer: Vec<u8> = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let mut first = false;
            if file_size.is_none() {
                if chunk.len() <= 8 {
                    intermediate_buffer.append(&mut chunk.to_vec());
                } else {
                    intermediate_buffer = chunk.to_vec();
                }
                if intermediate_buffer.len() > 7 {
                    let mut u64_bytes = [0u8; 8];
                    u64_bytes.copy_from_slice(&intermediate_buffer[0..8]);
                    file_size = Some(u64::from_le_bytes(u64_bytes));
                    first = true;
                }
            }
            if file_size.is_some() {
                if first {
                    file.write_all(&intermediate_buffer[8..])?;
                } else {
                    file.write_all(&chunk)?;
                }
            }
        }
        file.flush()?;

        let new_handler = file.reopen()?;
        let temporary_path = file.into_temp_path();

        Ok(PackageFile(new_handler, Some(temporary_path)))
    }

    pub(crate) fn get_size(&self) -> Result<u64> {
        Ok(self.metadata()?.len())
    }
}

#[derive(Debug)]
pub struct Package {
    pub metadata: PackageMetadata,
    pub file: PackageFile,
}

impl Package {
    pub fn new(metadata: PackageMetadata, file: PackageFile) -> Self {
        Self { metadata, file }
    }

    pub fn save(&self, package_folder: &str) -> Result<()> {
        self.file
            .save(package_folder, &self.metadata.name, &self.metadata.version)?;

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
        let archive_file = File::open(archive_path)?;
        Ok(PackageMetadata {
            name: package_body.name,
            version: package_body.version,
            description: package_body.description,
            license: package_body.license,
            readme_path: package_body.readme,
            checksum: PackageFile(archive_file, None).calculate_checksum()?,
        })
    }

    pub fn from_file(package_toml_path: PathBuf, compression: u32) -> Result<Self> {
        let archive_path = archiver::create_package(&package_toml_path, compression)?;
        let metadata = Self::gather_metadata(&package_toml_path, &archive_path)?;
        let file = File::open(&archive_path)?;
        let mut temp_path_option = None;
        if cfg!(feature = "test") {
            temp_path_option = Some(TempPath::from_path(archive_path));
        }

        Ok(Package {
            metadata,
            file: PackageFile(file, temp_path_option),
        })
    }

    pub fn get_size(&self) -> Result<u64> {
        self.file.get_size()
    }

    pub async fn to_stream(self) -> Result<PackageStream> {
        let metadata_bytes = Vec::try_from(&self.metadata)?;
        if metadata_bytes.len() < 4 {
            return Err(anyhow!("Metadata is too short"));
        }

        let stream: PackageStream =
            Box::pin(futures::stream::iter(vec![Ok(Bytes::from(metadata_bytes))]));

        let archive_file = TokioFile::from(self.file.0);
        let archive_size = archive_file.metadata().await?.len();

        let stream = stream
            .chain(archive_size.to_stream())
            .chain(archive_file.to_stream());

        return Ok(stream.boxed_local());
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

    fn try_from(index_info: &PackageMetadata) -> Result<Self> {
        let mut formatted_bytes = Vec::new();
        let string = serde_json::to_string(&index_info)?;
        let main_bytes = string.as_bytes();
        let length: u32 = main_bytes.len().try_into()?;

        formatted_bytes.extend_from_slice(&length.to_le_bytes());
        formatted_bytes.extend_from_slice(main_bytes);

        Ok(formatted_bytes)
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

#[cfg(test)]
mod tests {
    use crate::package::Package;
    use crate::test::TempArchive;
    use anyhow::{Ok, Result};
    use futures::StreamExt;
    use std::{fs::File, io::Read, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn package_file_can_be_saved() -> Result<()> {
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();
        let temp_project = TempArchive::builder()
            .set_package_name("some-package-name")
            .build()?;
        let package: Package = (&temp_project).try_into()?;

        let test_name = "test-name";
        let test_version = "1.0.0";
        let expected_file_path: PathBuf = [
            package_folder.clone(),
            test_name.to_string(),
            test_version.to_string(),
        ]
        .iter()
        .collect();
        package
            .file
            .save(&package_folder, test_name, test_version)?;

        let mut created_file = File::open(expected_file_path)?;
        assert!(created_file.metadata().unwrap().is_file());

        let mut file_content = Vec::new();
        created_file.read_to_end(&mut file_content)?;

        target_directory.close()?;
        Ok(())
    }

    #[test]
    fn package_can_be_saved() -> Result<()> {
        let archive = TempArchive::builder()
            .set_package_name("some-package-name")
            .build()?;
        let test_package: Package = (&archive).try_into()?;
        let target_directory = tempdir()?;
        let package_folder = target_directory.path().to_str().unwrap().to_string();

        test_package.save(&package_folder)?;

        assert!(target_directory.path().join("some-package-name").exists());
        target_directory.close()?;
        Ok(())
    }

    #[test]
    fn metadata_is_converted_to_bytes() -> Result<()> {
        let archive = TempArchive::builder().build()?;
        let package: Package = (&archive).try_into()?;
        let metadata = package.metadata;
        let _metadata_bytes = Vec::try_from(&metadata)?;
        Ok(())
    }

    #[actix_web::test]
    async fn package_is_converted_to_stream() -> Result<()> {
        let archive = TempArchive::builder().build()?;
        let package: Package = (&archive).try_into()?;
        let mut byte_stream = package.to_stream().await?;
        let mut counter = 0;
        let mut bytes = Vec::new();
        while let Some(chunk) = byte_stream.next().await {
            let chunk = chunk.unwrap();
            bytes.append(&mut chunk.to_vec());
            counter += 1;
        }
        assert_eq!(counter, 3);
        Ok(())
    }
}
