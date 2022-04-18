use deputy_library::constants::CONFIG_FILE_PATH_ENV_KEY;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{env, fs::read_to_string};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    pub dl: String,
    pub api: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repositories {
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub repository: Repositories,
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
    use super::*;
    use anyhow::Result;
    use std::io::Write;
    use tempfile::{tempdir, Builder, NamedTempFile, TempDir};

    fn create_temp_configuration_file() -> Result<(TempDir, NamedTempFile)> {
        let configuration_file_contents = br#"    
                [repository]
                repositories = [{ dl = "dllink", api = "apilink" }]"#;
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
        if env::var(CONFIG_FILE_PATH_ENV_KEY).is_err() {
            let (configuration_directory, configuration_file) = create_temp_configuration_file()?;
            env::set_var(CONFIG_FILE_PATH_ENV_KEY, &configuration_file.path());
            let configuration = Configuration::get_configuration()?;
            env::remove_var(CONFIG_FILE_PATH_ENV_KEY);
            configuration_directory.close()?;
            assert_eq!(configuration.repository.repositories[0].api, "apilink");
            assert_eq!(configuration.repository.repositories[0].dl, "dllink");
            Ok(())
        } else {
            let configuration = Configuration::get_configuration()?;
            assert!(!configuration.repository.repositories[0].api.is_empty());
            assert!(!configuration.repository.repositories[0].dl.is_empty());
            Ok(())
        }
    }
}
