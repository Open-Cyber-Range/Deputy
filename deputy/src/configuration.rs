use crate::constants::CONFIG_FILE_PATH_ENV_KEY;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs::read_to_string};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Registry {
    pub index: String,
    pub api: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Configuration {
    pub registries: HashMap<String, Registry>,
}

impl Configuration {
    pub fn get_configuration() -> Result<Configuration> {
        let configuration_path = env::var(CONFIG_FILE_PATH_ENV_KEY)?;
        let configuration_contents = read_to_string(configuration_path)?;
        Ok(toml::from_str(&configuration_contents)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::DEFAULT_REGISTRY_NAME;

    use super::*;
    use anyhow::Result;
    use std::io::Write;
    use tempfile::{tempdir, Builder, NamedTempFile, TempDir};

    fn create_temp_configuration_file() -> Result<(TempDir, NamedTempFile)> {
        let configuration_file_contents = br#"    
                [registries]
                main-registry = { index = "registry-index", api = "apilink" }"#;
        let configuration_directory = tempdir()?;
        let mut configuration_file = Builder::new()
            .prefix("configuration")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&configuration_directory)?;
        configuration_file.write_all(configuration_file_contents)?;
        Ok((configuration_directory, configuration_file))
    }

    #[test]
    fn read_contents_from_configuration_file() -> Result<()> {
        let (configuration_directory, configuration_file) = create_temp_configuration_file()?;
        env::set_var(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
        let configuration = Configuration::get_configuration()?;
        env::remove_var(CONFIG_FILE_PATH_ENV_KEY);
        configuration_directory.close()?;
        assert_eq!(
            configuration
                .registries
                .get(DEFAULT_REGISTRY_NAME)
                .unwrap()
                .api,
            "apilink"
        );
        assert_eq!(
            configuration
                .registries
                .get(DEFAULT_REGISTRY_NAME)
                .unwrap()
                .index,
            "registry-index"
        );
        Ok(())
    }
}
