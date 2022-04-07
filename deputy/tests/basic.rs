#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::{Builder, TempDir};

    #[test]
    fn test_version() -> Result<()> {
        let mut cmd = Command::cargo_bin("deputy")?;

        cmd.arg("version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));

        Ok(())
    }

    #[test]
    fn error_on_missing_package_toml_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir.into_path().canonicalize()?;

        let mut cmd = Command::cargo_bin("deputy")?;
        cmd.arg("publish");
        cmd.current_dir(temp_dir);
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: Could not find package.toml",
        ));

        Ok(())
    }
    #[test]
    fn error_on_missing_package_toml_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let _package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;
        let temp_dir = temp_dir.into_path().canonicalize()?;

        let mut cmd = Command::cargo_bin("deputy")?;
        cmd.arg("publish");
        cmd.current_dir(temp_dir);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error: missing field `package`"));

        Ok(())
    }

    #[test]
    fn finds_invalid_package_toml_from_parent_folder() -> Result<()> {
        let root_temp_dir = TempDir::new()?;
        let deep_path: PathBuf = ["some", "where", "many", "layers", "deep"].iter().collect();
        let deep_path = root_temp_dir.path().join(deep_path);
        std::fs::create_dir_all(&deep_path)?;

        let _package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&root_temp_dir)?;

        let mut cmd = Command::cargo_bin("deputy")?;
        cmd.arg("publish");
        cmd.current_dir(deep_path.as_path());
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error: missing field `package`"));

        Ok(())
    }
}
