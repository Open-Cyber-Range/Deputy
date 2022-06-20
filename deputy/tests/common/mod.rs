use anyhow::Result;
use deputy::constants::DEFAULT_REGISTRY_NAME;
use std::io::Write;
use tempfile::{tempdir, Builder, NamedTempFile, TempDir};

pub fn create_temp_configuration_file(
    api_address: &str,
    index_repository: &str,
) -> Result<(TempDir, NamedTempFile)> {
    let configuration_file_contents = format!(
        "[registries]\n{DEFAULT_REGISTRY_NAME} = {{ index = \"{index_repository}\", api = \"{api_address}\" }}\n[package]\nindex_path = \"./index\"\ndownload_path = \"./download\"",
    );

    let configuration_directory = tempdir()?;
    let mut configuration_file = Builder::new()
        .prefix("configuration")
        .suffix(".toml")
        .rand_bytes(0)
        .tempfile_in(&configuration_directory)?;
    configuration_file.write_all(configuration_file_contents.as_bytes())?;
    Ok((configuration_directory, configuration_file))
}
