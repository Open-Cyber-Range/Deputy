use anyhow::Result;
use std::io::Write;
use tempfile::{tempdir, Builder, NamedTempFile, TempDir};

pub fn create_temp_configuration_file(api_address: &str) -> Result<(TempDir, NamedTempFile)> {
    let configuration_file_contents =
        format!("[repository]\nrepositories = [{{ dl = \"dllink\", api = \"{api_address}\" }}]",);
    let configuration_directory = tempdir()?;
    let mut configuration_file = Builder::new()
        .prefix("configuration")
        .suffix(".toml")
        .rand_bytes(0)
        .tempfile_in(&configuration_directory)?;
    configuration_file.write_all(configuration_file_contents.as_bytes())?;
    Ok((configuration_directory, configuration_file))
}
