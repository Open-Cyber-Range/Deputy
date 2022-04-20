use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::{
    constants,
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

fn validate_type(content: Content) -> Result<()> {
    let is_valid = match content.content_type {
        ContentType::VM => constants::VALID_VM_TYPES.contains(&&content.sub_type),
    };

    if !is_valid {
        return Err(anyhow!("Sub-type mismatch with the type"));
    }
    Ok(())
}

pub fn validate_package_toml<P: AsRef<Path> + Debug>(package_path: P) -> Result<()> {
    let mut file = File::open(package_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let deserialized_toml: Project = toml::from_str(&*contents)?;
    validate_name(deserialized_toml.package.name)?;
    validate_version(deserialized_toml.package.version)?;
    validate_type(deserialized_toml.content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
sub_type = "packer"
"#;
        let (file, deserialized_toml) = create_temp_file(toml_content)?;
        Ok((file, deserialized_toml))
    }

    #[test]
    fn positive_result_all_fields_correct() -> Result<()> {
        let toml_content = br#"
[package]
name = "test_package_1-0-4"
description = "This package does nothing at all, and we spent 300 manhours on it..."
version = "1.0.4"
authors = ["Robert robert@exmaple.com", "Bobert the III bobert@exmaple.com", "Miranda Rustacean miranda@rustacean.rust" ]
[content]
type = "vm"
sub_type = "packer"
"#;
        let (file, _deserialized_toml) = create_temp_file(toml_content)?;
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
    #[should_panic]
    fn negative_result_content_type_field() {
        std::panic::set_hook(Box::new(|_| {}));
        let toml_content = br#"
[package]
name = "package"
description = "Package description"
version = "1.0.0"
[content]
type = "virtuelle machine"
sub_type = "packer"
"#;
        let (file, deserialized_toml) = create_temp_file(toml_content).unwrap();
        assert!(validate_type(deserialized_toml.content).is_err());
        file.close().unwrap();
    }

    #[test]
    #[should_panic]
    fn negative_result_content_subtype_field() {
        std::panic::set_hook(Box::new(|_| {}));
        let toml_content = br#"
[package]
name = "package"
description = "Package description"
version = "1.0.0"
[content]
type = "vm"
sub_type = "something_wrong"
"#;
        let (file, deserialized_toml) = create_temp_file(toml_content).unwrap();
        assert!(validate_type(deserialized_toml.content).is_err());
        file.close().unwrap();
    }
}
