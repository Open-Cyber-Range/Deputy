use crate::constants::PACKAGE_TOML;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn find_toml(current_path: PathBuf) -> Result<PathBuf> {
    let mut toml_path = current_path.join(PACKAGE_TOML);
    if toml_path.is_file() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        Ok(find_toml(toml_path)?)
    } else {
        Err(anyhow!("Could not find package.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tempfile::{Builder, TempDir};

    #[test]
    fn successfully_found_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_toml = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&temp_dir)?;

        assert!(find_toml(temp_dir.path().to_path_buf())?.is_file());
        package_toml.close()?;
        temp_dir.close()?;
        Ok(())
    }
}
