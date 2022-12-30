use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::{
    constants::{self},
    package::{IndexInfo, Package},
    project::*,
};
use anyhow::{anyhow, Result};
use semver::Version;
use spdx;

pub trait Validate {
    fn validate(&mut self) -> Result<()>;
}

impl Validate for Package {
    fn validate(&mut self) -> Result<()> {
        self.index_info.validate()?;
        self.validate_checksum()?;
        Ok(())
    }
}

impl Validate for IndexInfo {
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

impl Validate for Project {
    fn validate(&mut self) -> Result<()> {
        self.validate_content()?;
        validate_name(self.package.name.clone())?;
        validate_version(self.package.version.clone())?;
        validate_vm_accounts(self.virtual_machine.clone())?;
        validate_license(self.package.license.clone())?;
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

pub fn validate_license(license: String) -> Result<()> {
    match spdx::license_id(&license) {
        Some(_) => Ok(()),
        None => Err(anyhow!(
            "License must match SPDX specifications https://spdx.dev/spdx-specification-21-web-version/#h.jxpfx0ykyb60"
        )),
    }
}

pub fn validate_vm_accounts(virtual_machine: Option<VirtualMachine>) -> Result<()> {
    match virtual_machine {
        Some(virtual_machine) => {
            if let Some(accounts) = virtual_machine.accounts {
                if let Some(default_account) = virtual_machine.default_account {
                    for account in accounts.iter() {
                        if account.name.eq_ignore_ascii_case(&default_account) {
                            return Ok(());
                        }
                    }
                    return Err(anyhow!("Default account not found under accounts"));
                }
                return Err(anyhow!("Accounts defined but no default account assigned"));
            }
            if virtual_machine.accounts.is_none() && virtual_machine.default_account.is_some() {
                return Err(anyhow!("Default account assigned but no accounts defined"));
            }
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn validate_package_toml<P: AsRef<Path> + Debug>(package_path: P) -> Result<()> {
    let mut file = File::open(package_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut deserialized_toml: Project = toml::from_str(&contents)?;
    deserialized_toml.validate()?;
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

    fn create_incorrect_name_version_license_toml() -> Result<(NamedTempFile, Project)> {
        let toml_content = br#"
            [package]
            name = "this is incorrect formatting"
            description = "description"
            version = "version 23"
            license = "Very bad licence"
            readme = "readme.md"
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
        let (file, deserialized_toml) = create_incorrect_name_version_license_toml()?;
        assert!(validate_name(deserialized_toml.package.name).is_err());
        file.close()?;
        Ok(())
    }

    #[test]
    fn negative_result_version_field() -> Result<()> {
        let (file, deserialized_toml) = create_incorrect_name_version_license_toml()?;
        assert!(validate_version(deserialized_toml.package.version).is_err());
        file.close()?;
        Ok(())
    }

    #[test]
    fn negative_result_license_field() -> Result<()> {
        let (file, deserialized_toml) = create_incorrect_name_version_license_toml()?;
        assert!(validate_license(deserialized_toml.package.license).is_err());
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

    #[test]
    fn negative_result_on_mismatched_default_account() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-package"
            description = "description"
            version = "1.2.3"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "vm"
            [virtual-machine]
            default_account = "user404"
            type = "OVA"
            file_path = "some-path"
            "#;
        let (file, _) = create_temp_file(toml_content)?;
        assert!(validate_package_toml(&file.path()).is_err());
        file.close()?;
        Ok(())
    }

    #[test]
    fn negative_result_on_missing_default_account() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-package"
            description = "description"
            version = "1.2.3"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "vm"
            [virtual-machine]
            accounts = [{name = "user1", password = "password1"},{name = "user2", password = "password2"}]
            type = "OVA"
            file_path = "some-path"
            "#;
        let (file, _) = create_temp_file(toml_content)?;
        assert!(validate_package_toml(&file.path()).is_err());
        file.close()?;
        Ok(())
    }

    #[test]
    fn feature_type_package_is_parsed_and_passes_validation() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-feature"
            description = "description"
            version = "1.0.0"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "feature"
            [feature]
            type = "configuration"
            action = "ping 8.8.8.8"
            assets = [
            ["src/configs/my-cool-config1.yml", "/var/opt/my-cool-service1", "744"],
            ["src/configs/my-cool-config2.yml", "/var/opt/my-cool-service2", "777"],
            ["src/configs/my-cool-config3.yml", "/var/opt/my-cool-service3"],
            ]
            "#;
        let (file, project) = create_temp_file(toml_content)?;

        assert!(validate_package_toml(&file.path()).is_ok());
        insta::with_settings!({sort_maps => true}, {
                insta::assert_toml_snapshot!(project);
        });

        file.close()?;
        Ok(())
    }

    #[test]
    fn inject_type_package_is_parsed_and_passes_validation() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-feature"
            description = "description"
            version = "1.0.0"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "inject"
            [inject]
            action = "ping 8.8.8.8"
            assets = [
            ["src/configs/my-cool-config1.yml", "/var/opt/my-cool-service1", "744"],
            ["src/configs/my-cool-config2.yml", "/var/opt/my-cool-service2", "777"],
            ["src/configs/my-cool-config3.yml", "/var/opt/my-cool-service3"],
            ]
            "#;
        let (file, project) = create_temp_file(toml_content)?;

        assert!(validate_package_toml(&file.path()).is_ok());
        insta::with_settings!({sort_maps => true}, {
                insta::assert_toml_snapshot!(project);
        });

        file.close()?;
        Ok(())
    }

    #[test]
    fn condition_type_package_is_parsed_and_passes_validation() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-condition"
            description = "description"
            version = "1.0.0"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "condition"
            [condition]
            action = "executable/path.sh"
            interval = 30
            assets = [
                ["src/configs/my-cool-config1.yml", "/var/opt/my-cool-service1", "744"],
                ["src/configs/my-cool-config2.yml", "/var/opt/my-cool-service2", "777"],
                ["src/configs/my-cool-config3.yml", "/var/opt/my-cool-service3"],
                ]
            "#;
        let (file, project) = create_temp_file(toml_content)?;

        assert!(validate_package_toml(&file.path()).is_ok());
        insta::with_settings!({sort_maps => true}, {
                insta::assert_toml_snapshot!(project);
        });

        file.close()?;
        Ok(())
    }

    #[test]
    fn event_type_package_is_parsed_and_passes_validation() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-condition"
            description = "description"
            version = "1.0.0"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "event"
            [event]
            action = "ping 1.3.3.7"
            assets = [
            ["src/configs/my-cool-config1.yml", "/var/opt/my-cool-service1", "744"],
            ["src/configs/my-cool-config2.yml", "/var/opt/my-cool-service2", "777"],
            ["src/configs/my-cool-config3.yml", "/var/opt/my-cool-service3"],
            ]
            "#;
        let (file, project) = create_temp_file(toml_content)?;

        assert!(validate_package_toml(&file.path()).is_ok());
        insta::with_settings!({sort_maps => true}, {
                insta::assert_toml_snapshot!(project);
        });

        file.close()?;
        Ok(())
    }

    #[test]
    fn negative_result_on_content_type_not_matching_content() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-condition"
            description = "description"
            version = "1.0.0"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "feature"
            [condition]
            action = "executable/path.sh"
            interval = 30
            assets = [
                ["src/configs/my-cool-config1.yml", "/var/opt/my-cool-service1", "744"],
                ["src/configs/my-cool-config2.yml", "/var/opt/my-cool-service2", "777"],
                ["src/configs/my-cool-config3.yml", "/var/opt/my-cool-service3"],
                ]
            "#;
        let (file, _) = create_temp_file(toml_content)?;

        assert!(validate_package_toml(&file.path()).is_err());
        file.close()?;
        Ok(())
    }

    #[test]
    fn negative_result_on_multiple_contents() -> Result<()> {
        let toml_content = br#"
            [package]
            name = "my-cool-condition"
            description = "description"
            version = "1.0.0"
            license = "Apache-2.0"
            readme = "readme.md"
            [content]
            type = "feature"
            [feature]
            type = "configuration"
            action = "ping 8.8.8.8"
            assets = [
            ["src/configs/my-cool-config1.yml", "/var/opt/my-cool-service1", "744"],
            ["src/configs/my-cool-config2.yml", "/var/opt/my-cool-service2", "777"],
            ["src/configs/my-cool-config3.yml", "/var/opt/my-cool-service3"],
            ]
            [condition]
            action = "executable/path.sh"
            interval = 30
            assets = [
                ["src/configs/my-cool-config1.yml", "/var/opt/my-cool-service1", "744"],
                ["src/configs/my-cool-config2.yml", "/var/opt/my-cool-service2", "777"],
                ["src/configs/my-cool-config3.yml", "/var/opt/my-cool-service3"],
                ]
            "#;
        let (file, _) = create_temp_file(toml_content)?;

        assert!(validate_package_toml(&file.path()).is_err());
        file.close()?;
        Ok(())
    }
}
