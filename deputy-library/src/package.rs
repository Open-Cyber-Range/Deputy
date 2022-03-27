use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Read},
    ops::Deref,
};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub checksum: String,
}

pub struct PackageFile<'a>(&'a File);

impl<'a> Deref for PackageFile<'a> {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        self.0
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

impl<'a> TryFrom<PackageFile<'a>> for Vec<u8> {
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

pub fn package_to_bytes(package_metadata: &PackageMetadata, file: &File) -> Result<Vec<u8>> {
    let mut payload: Vec<u8> = Vec::new();
    let package_file = PackageFile(file);

    let metadata_bytes = Vec::try_from(package_metadata)?;
    payload.extend(metadata_bytes);
    let file_bytes = Vec::try_from(package_file)?;
    payload.extend(file_bytes);

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use crate::{
        package::{package_to_bytes, PackageFile, PackageMetadata},
        test::{create_readable_temporary_file, TEST_PACKAGE_METADATA},
    };
    use anyhow::Result;

    #[test]
    fn metadata_is_converted_to_bytes() -> Result<()> {
        let package_metadata: &PackageMetadata = &TEST_PACKAGE_METADATA;
        let metadata_bytes = Vec::try_from(package_metadata)?;

        insta::assert_debug_snapshot!(metadata_bytes);
        Ok(())
    }

    #[test]
    fn file_is_converted_to_bytes() -> Result<()> {
        let temporary_file = create_readable_temporary_file("Some content\n")?;
        let metadata_bytes = Vec::try_from(PackageFile(&temporary_file))?;

        insta::assert_debug_snapshot!(metadata_bytes);
        Ok(())
    }

    #[test]
    fn package_is_converted_to_bytes() -> Result<()> {
        let temporary_file = create_readable_temporary_file("Some content\n")?;
        let package_bytes = package_to_bytes(&TEST_PACKAGE_METADATA, &temporary_file)?;
        insta::assert_debug_snapshot!(package_bytes);

        Ok(())
    }
}
