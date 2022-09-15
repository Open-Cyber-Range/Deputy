use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::{
    constants::{self},
    package::{Package, PackageMetadata},
    project::*,
};
use anyhow::{anyhow, Result};
use semver::Version;

pub trait Validate {
    fn validate(&mut self) -> Result<()>;
}

impl Validate for Package {
    fn validate(&mut self) -> Result<()> {
        self.metadata.validate()?;
        self.validate_checksum()?;
        Ok(())
    }
}

impl Validate for PackageMetadata {
    fn validate(&mut self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Package name is empty"));
        }
        if self.version.is_empty() {
            return Err(anyhow!("Package version is empty"));
        }
        if self.version.parse::<Version>().is_err() {
            return Err(anyhow!("Package version is not valid"));
        }
        if self.checksum.is_empty() {
            return Err(anyhow!("Package checksum is empty"));
        }
        if !(self.checksum.len() == constants::SHA256_LENGTH as usize
            && self.checksum.chars().all(|c| c.is_ascii_hexdigit()))
        {
            return Err(anyhow!("Package checksum is not valid"));
        }
        Ok(())
    }
}

pub fn validate_name(name: String) -> Result<()> {
    if !constants::VALID_NAME.is_match(&name)? {
        return Err(anyhow!(
            "Name {:?} must be one word of alphanumeric, `-`, or `_` characters.",
            name
        ));
    };
    Ok(())
}

pub fn validate_version(version: String) -> Result<()> {
    match Version::parse(version.as_str()) {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!(
            "Version {:?} must match Semantic Versioning 2.0.0 https://semver.org/",
            version
        )),
    }
}

pub fn validate_package_toml<P: AsRef<Path> + Debug>(package_path: P) -> Result<()> {
    let mut file = File::open(package_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let deserialized_toml: Project = toml::from_str(&*contents)?;
    validate_name(deserialized_toml.package.name)?;
    validate_version(deserialized_toml.package.version)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        project::enums::{Architecture, OperatingSystem},
        test::{TEST_INVALID_PACKAGE_TOML_SCHEMA, TEST_VALID_PACKAGE_TOML_SCHEMA},
    };

    use anyhow::Ok;
    use std::io::Write;
    use tempfile::{Builder, NamedTempFile};

    fn create_temp_file(toml_content: &[u8]) -> Result<(NamedTempFile, Project)> {
        let mut file = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .tempfile()?;
        file.write_all(toml_content)?;
        let deserialized_toml = deserialize_toml(&file)?;
        Ok((file, deserialized_toml))
    }

    fn deserialize_toml(file: &NamedTempFile) -> Result<Project> {
        let mut contents = String::new();
        let mut read_file = File::open(file.path())?;
        read_file.read_to_string(&mut contents)?;
        let deserialized_toml: Project = toml::from_str(&*contents)?;
        Ok(deserialized_toml)
    }

    fn create_incorrect_name_and_version_toml() -> Result<(NamedTempFile, Project)> {
        let toml_content = br#"
            [package]
            name = "this is incorrect formatting"
            description = "description"
            version = "version 23"
            [content]
            type = "vm"
            "#;
        let (file, deserialized_toml) = create_temp_file(toml_content)?;
        Ok((file, deserialized_toml))
    }

    #[test]
    fn positive_result_all_fields_correct() -> Result<()> {
        let (file, _deserialized_toml) =
            create_temp_file(TEST_VALID_PACKAGE_TOML_SCHEMA.as_bytes())?;
        assert!(validate_package_toml(&file.path()).is_ok());
        file.close()?;
        Ok(())
    }

    #[test]
    fn negative_result_name_field() -> Result<()> {
        let (file, deserialized_toml) = create_incorrect_name_and_version_toml()?;
        assert!(validate_name(deserialized_toml.package.name).is_err());
        file.close()?;
        Ok(())
    }

    #[test]
    fn negative_result_version_field() -> Result<()> {
        let (file, deserialized_toml) = create_incorrect_name_and_version_toml()?;
        assert!(validate_version(deserialized_toml.package.version).is_err());
        file.close()?;
        Ok(())
    }

    #[test]
    fn missing_architecture_field_is_given_value_none() -> Result<()> {
        let (file, deserialized_toml) =
            create_temp_file(TEST_INVALID_PACKAGE_TOML_SCHEMA.as_bytes())?;
        if let Some(virtual_machine) = &deserialized_toml.virtual_machine {
            assert!(virtual_machine.architecture.is_none());
        }
        file.close()?;
        Ok(())
    }

    #[test]
    fn invalid_operating_system_field_is_given_value_unknown() -> Result<()> {
        let (file, deserialized_toml) =
            create_temp_file(TEST_INVALID_PACKAGE_TOML_SCHEMA.as_bytes())?;
        if let Some(virtual_machine) = deserialized_toml.virtual_machine {
            if let Some(operating_system) = virtual_machine.operating_system {
                assert_eq!(operating_system, OperatingSystem::Unknown);
            }
        }
        file.close()?;
        Ok(())
    }

    #[test]
    fn valid_operating_system_and_architecture_fields_are_parsed_correctly() -> Result<()> {
        let (file, deserialized_toml) =
            create_temp_file(TEST_VALID_PACKAGE_TOML_SCHEMA.as_bytes())?;
        if let Some(virtual_machine) = deserialized_toml.virtual_machine {
            if let Some(operating_system) = virtual_machine.operating_system {
                assert_eq!(operating_system, OperatingSystem::Debian);
            }
            if let Some(architecture) = virtual_machine.architecture {
                assert_eq!(architecture, Architecture::arm64);
            }
        }
        file.close()?;
        Ok(())
    }
}
