use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub hostname: String,
    pub package_folder: String,
    pub database_url: String,
}

pub fn read_configuration(arguments: Vec<String>) -> Result<Configuration> {
    let file_path = arguments
        .get(1)
        .ok_or_else(|| Error::msg("Configuration path argument missing"))?;

    let configuration_string = read_to_string(file_path)?;
    Ok(serde_yaml::from_str(&configuration_string)?)
}

#[cfg(test)]
mod tests {
    use super::read_configuration;
    use anyhow::Result;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn can_parse_the_configuration() -> Result<()> {
        let temporary_directory = tempdir()?;
        let file_path = temporary_directory.path().join("test-config.yml");
        let path_string = file_path.clone().into_os_string().into_string().unwrap();
        let mut file = File::create(file_path)?;
        writeln!(
            file,
            r#"
hostname: localhost:8080
package_folder: /tmp/packages
database_url: mysql://mysql_user:mysql_pass@deputy-mariadb:3307/deputy
    "#
        )?;
        let arguments = vec![String::from("program-name"), path_string];
        let configuration = read_configuration(arguments)?;
        insta::assert_debug_snapshot!(configuration);
        Ok(())
    }
}
