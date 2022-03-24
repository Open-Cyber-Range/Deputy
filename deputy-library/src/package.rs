use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::PackageMetadata;
use anyhow::Result;

fn format_metadata(package_metadata: &PackageMetadata) -> Result<Vec<u8>> {
    let mut formatted_bytes = Vec::new();
    let string = serde_json::to_string(&package_metadata)?;
    let main_bytes = string.as_bytes();
    let length: u32 = main_bytes.len().try_into()?;

    formatted_bytes.extend_from_slice(&length.to_le_bytes());
    formatted_bytes.extend_from_slice(main_bytes);

    Ok(formatted_bytes)
}

fn format_file(file: &File) -> Result<Vec<u8>> {
    let mut formatted_bytes = Vec::new();
    let mut reader = BufReader::new(file);
    let mut file_buffer = Vec::new();
    reader.read_to_end(&mut file_buffer)?;

    let length: u32 = file_buffer.len().try_into()?;
    formatted_bytes.extend_from_slice(&length.to_le_bytes());
    formatted_bytes.extend(file_buffer);

    Ok(formatted_bytes)
}

pub fn package_to_bytes(package_metadata: &PackageMetadata, file: &File) -> Result<Vec<u8>> {
    let mut payload: Vec<u8> = Vec::new();

    let metadata_bytes = format_metadata(package_metadata)?;
    payload.extend(metadata_bytes);
    let file_bytes = format_file(file)?;
    payload.extend(file_bytes);

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::format_metadata;
    use crate::{
        package::{format_file, package_to_bytes},
        test::{create_readable_temporary_file, TEST_PACKAGE_METADATA},
    };
    use anyhow::Result;

    #[test]
    fn metadata_is_converted_to_bytes() -> Result<()> {
        let metadata_bytes = format_metadata(&TEST_PACKAGE_METADATA)?;

        insta::assert_debug_snapshot!(metadata_bytes);
        Ok(())
    }

    #[test]
    fn file_is_converted_to_bytes() -> Result<()> {
        let temporary_file = create_readable_temporary_file("Some content\n")?;
        let metadata_bytes = format_file(&temporary_file)?;

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
