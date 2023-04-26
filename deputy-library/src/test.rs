use crate::package::Package;
use anyhow::{anyhow, Ok, Result};
use byte_unit::Byte;
use filetime::{set_file_times, FileTime};
use rand::Rng;
use rayon::current_num_threads;
use std::{
    fs::{set_permissions, Permissions},
    io::Write,
    os::unix::prelude::PermissionsExt,
};
use tempfile::{Builder, NamedTempFile, TempDir};

lazy_static! {
    pub static ref TEST_INVALID_PACKAGE_TOML_SCHEMA: &'static str = r#"
        [package]
        name = "test_package_1"
        description = "This is a package"
        version = "1.0.4"
        authors = ["Robert robert@exmaple.com"]
        license = "very bad licence"
        readme = "readme"
        [content]
        type = "vm"
        [virtual-machine]
        operating_system = "Invalid OS and missing Architecture"
        type = "OVA"
        file_path = "src/some-image.ova"
        "#;
    pub static ref TEST_VALID_PACKAGE_TOML_SCHEMA: &'static str = r#"
        [package]
        name = "test_package_1-0-4"
        description = "This package does nothing at all, and we spent 300 manhours on it..."
        version = "1.0.4"
        authors = ["Robert robert@exmaple.com", "Bobert the III bobert@exmaple.com", "Miranda Rustacean miranda@rustacean.rust" ]
        license = "Apache-2.0"
        readme = "readme"
        [content]
        type = "vm"
        [virtual-machine]
        accounts = [{name = "user1", password = "password1"},{name = "user2", password = "password2"}]
        default_account = "user1"
        operating_system = "Debian"
        architecture = "arm64"
        type = "OVA"
        file_path = "src/some-image.ova"
        "#;
}

pub struct TempArchive {
    pub root_dir: TempDir,
    pub target_dir: TempDir,
    pub src_dir: TempDir,
    pub target_file: NamedTempFile,
    pub src_file: NamedTempFile,
    pub toml_file: NamedTempFile,
}

impl TempArchive {
    pub fn builder() -> TempArchiveBuilder {
        TempArchiveBuilder::new()
    }
}

impl TryInto<Package> for &TempArchive {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Package> {
        let toml_path = self.toml_file.path().to_path_buf();
        Package::from_file(toml_path, 0)
    }
}

#[derive(Default)]
pub struct TempArchiveBuilder {
    is_large: bool,
    zero_filetimes: bool,
    zero_fileowner: bool,
    all_allowed_permission: bool,
    package_name: String,
    package_version: String,
}

impl TempArchiveBuilder {
    pub fn new() -> TempArchiveBuilder {
        TempArchiveBuilder {
            is_large: false,
            zero_filetimes: true,
            zero_fileowner: true,
            all_allowed_permission: true,
            package_name: String::from("test_package_1"),
            package_version: String::from("1.0.4"),
        }
    }

    pub fn is_large(mut self, value: bool) -> Self {
        self.is_large = value;
        self
    }

    pub fn zero_filetimes(mut self, value: bool) -> Self {
        self.zero_filetimes = value;
        self
    }

    pub fn zero_fileowner(mut self, value: bool) -> Self {
        self.zero_fileowner = value;
        self
    }

    pub fn all_allowed_permission(mut self, value: bool) -> Self {
        self.all_allowed_permission = value;
        self
    }

    pub fn set_package_name(mut self, value: &str) -> Self {
        self.package_name = value.to_string();
        self
    }

    pub fn set_package_version(mut self, value: &str) -> Self {
        self.package_version = value.to_string();
        self
    }

    fn generate_vec(size: usize) -> Result<Vec<u8>> {
        let bytes_per_thread = size / current_num_threads();
        let mut handles = Vec::new();
        for _ in 0..current_num_threads() {
            let handle = std::thread::spawn(move || {
                let mut vec = Vec::new();
                for _ in 0..bytes_per_thread {
                    vec.push(rand::random::<u8>());
                }
                vec
            });
            handles.push(handle);
        }
        let mut final_result: Vec<u8> = Vec::new();
        while !handles.is_empty() {
            let current_thread = handles.remove(0);
            final_result.extend(
                current_thread
                    .join()
                    .map_err(|error| anyhow!("Failed to join due to: {:?}", error))?,
            );
        }
        Ok(final_result)
    }

    pub fn build(self) -> Result<TempArchive> {
        let toml_content = format!(
            r#"
                [package]
                name = "{}"
                description = "This package does nothing at all, and we spent 300 manhours on it..."
                version = "{}"
                authors = ["Robert robert@exmaple.com", "Bobert the III bobert@exmaple.com", "Miranda Rustacean miranda@rustacean.rust" ]
                license = "Apache-2.0"
                readme = "readme"
                [content]
                type = "vm"
                [virtual-machine]
                operating_system = "Ubuntu"
                architecture = "arm64"
                type = "OVA"
                file_path = "/src/test_file.txt"
            "#,
            self.package_name, self.package_version
        );
        let target_file_ipsum =
            br#"
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean consectetur nisl at aliquet pharetra. Cras fringilla
            quis leo quis tempus. Aliquam efficitur orci sapien, in luctus elit tempor id. Sed eget dui odio. Suspendisse potenti.
            Vestibulum purus quam, fringilla vitae egestas eget, convallis et ex. In ut euismod libero, eget euismod leo. Curabitur
            semper dolor mi, quis scelerisque purus fermentum eu.
            Mauris euismod felis diam, et dictum ante porttitor ac. Suspendisse lacus sapien, maximus et accumsan ultrices, porta
            vel leo. Pellentesque pulvinar enim elementum odio porta, vitae ultricies justo condimentum.
            "#;

        let src_file_ipsum =
            br#"
            Mauris elementum non quam laoreet tristique. Aenean sed nisl a quam venenatis porttitor. Nullam turpis velit, maximus
            vitae orci nec, tempus fermentum quam. Vestibulum tristique sollicitudin dignissim. Interdum et malesuada fames ac ante
            ipsum primis in faucibus. Phasellus at neque metus. Ut eleifend venenatis arcu. Vestibulum vitae elit ante. Sed fringilla
            placerat magna sollicitudin convallis. Maecenas semper est id tortor interdum, et tempus eros viverra. Fusce at quam nisl.
            Vivamus elementum at arcu et semper. Donec molestie, lorem et condimentum congue, nisl nisl mattis lorem, rhoncus dapibus
            ex massa eget felis.
            "#;
        let dir = TempDir::new()?;

        let target_dir = Builder::new()
            .prefix("target")
            .rand_bytes(0)
            .tempdir_in(&dir)?;
        let mut target_file = Builder::new()
            .prefix("test_target_file")
            .suffix(".txt")
            .rand_bytes(0)
            .tempfile_in(&target_dir)?;
        target_file.write_all(target_file_ipsum)?;
        if self.zero_filetimes {
            set_file_times(target_file.path(), FileTime::zero(), FileTime::zero())?;
        }
        if cfg!(unix) && self.zero_fileowner {
            use file_owner::PathExt;
            target_file
                .path()
                .set_owner(1000)
                .map_err(|error| anyhow!("Failed to set owner due to: {:?}", error))?;
        }

        let src_dir = Builder::new()
            .prefix("src")
            .rand_bytes(0)
            .tempdir_in(&dir)?;
        let mut src_file = Builder::new()
            .prefix("test_file")
            .suffix(".txt")
            .rand_bytes(0)
            .tempfile_in(&src_dir)?;
        src_file.write_all(src_file_ipsum)?;
        if self.zero_filetimes {
            set_file_times(src_file.path(), FileTime::zero(), FileTime::zero())?;
        }
        if cfg!(unix) && self.zero_fileowner {
            use file_owner::PathExt;
            src_file
                .path()
                .set_owner(1000)
                .map_err(|error| anyhow!("Failed to set owner due to: {:?}", error))?;
        }

        let mut toml_file = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&dir)?;
        toml_file.write_all(toml_content.as_bytes())?;
        if self.zero_filetimes {
            set_file_times(toml_file.path(), FileTime::zero(), FileTime::zero())?;
        }
        if cfg!(unix) && self.zero_fileowner {
            use file_owner::PathExt;
            toml_file
                .path()
                .set_owner(1000)
                .map_err(|error| anyhow!("Failed to set owner due to: {:?}", error))?;
        }

        if self.all_allowed_permission {
            let permissions = Permissions::from_mode(0o777);
            set_permissions(toml_file.path(), permissions.clone())?;
            set_permissions(src_file.path(), permissions.clone())?;
            set_permissions(target_file.path(), permissions)?;
        }

        if self.is_large {
            let mut large_file = Builder::new()
                .prefix("large")
                .suffix(".txt")
                .rand_bytes(0)
                .tempfile_in(&dir)?;
            let random_bytes: Vec<u8> =
                TempArchiveBuilder::generate_vec(Byte::from_str("20MB")?.get_bytes() as usize)?;
            large_file.write_all(&random_bytes)?;

            if self.zero_filetimes {
                set_file_times(large_file.path(), FileTime::zero(), FileTime::zero())?;
            }
            if self.all_allowed_permission {
                let permissions = Permissions::from_mode(0o777);
                set_permissions(large_file.path(), permissions)?;
            }
            if cfg!(unix) && self.zero_fileowner {
                use file_owner::PathExt;
                large_file
                    .path()
                    .set_owner(1000)
                    .map_err(|error| anyhow!("Failed to set owner due to: {:?}", error))?;
            }
        }

        let temp_project = TempArchive {
            root_dir: dir,
            target_dir,
            src_dir,
            target_file,
            src_file,
            toml_file,
        };

        Ok(temp_project)
    }
}

pub fn generate_random_string(length: usize) -> Result<String> {
    let random_bytes = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .collect::<Vec<u8>>();
    Ok(String::from_utf8(random_bytes)?)
}
