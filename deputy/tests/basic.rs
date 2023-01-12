#![allow(dead_code)]

mod test_backend;

#[cfg(test)]
mod tests {
    use crate::test_backend::TestBackEndBuilder;
    use anyhow::Result;
    use assert_cmd::prelude::*;
    use deputy_library::constants::CONFIGURATION_FOLDER_PATH_ENV_KEY;
    use predicates::prelude::*;
    use std::{env, process::Command};

    #[test]
    fn test_version() -> Result<()> {
        let mut command = Command::cargo_bin("deputy")?;
        let (configuration_directory, configuration_file) =
            TestBackEndBuilder::create_temp_configuration_file(
                "does-not-matter",
                "does-not-matter",
            )?;
        command.arg("--version");
        command.env(
            CONFIGURATION_FOLDER_PATH_ENV_KEY,
            &configuration_directory.path(),
        );
        command
            .assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
        configuration_file.close()?;
        configuration_directory.close()?;
        Ok(())
    }
}
