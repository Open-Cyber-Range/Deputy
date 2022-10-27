use crate::package::{Package, PackageFile, IndexInfo};
use anyhow::{anyhow, Ok, Result};
use byte_unit::Byte;
use git2::{Repository, RepositoryInitOptions};
use port_check::free_local_port;
use rand::Rng;
use rayon::current_num_threads;
use std::{fs::File, io::Write};
use tempfile::{Builder, NamedTempFile, TempDir, TempPath};

lazy_static! {
    pub static ref TEST_INDEX_METADATA: IndexInfo = IndexInfo {
        checksum: "aa30b1cc05c10ac8a1f309e3de09de484c6de1dc7c226e2cf8e1a518369b1d73".to_string(),
        version: "0.1.0".to_string(),
        name: "some-package-name".to_string(),
    };
    pub static ref TEST_INVALID_PACKAGE_TOML_SCHEMA: &'static str = r#"
        [package]
        name = "test_package_1"
        description = "This is a package"
        version = "1.0.4"
        authors = ["Robert robert@exmaple.com"]
        license = "very bad licence"
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
    pub static ref TEST_METADATA_BYTES: Vec<u8> = vec![
        123, 34, 110, 97, 109, 101, 34, 58, 34, 115, 111, 109, 101, 45, 112, 97, 99, 107, 97, 103,
        101, 45, 110, 97, 109, 101, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58, 34, 48,
        46, 49, 46, 48, 34, 44, 34, 99, 104, 101, 99, 107, 115, 117, 109, 34, 58, 34, 97, 97, 51,
        48, 98, 49, 99, 99, 48, 53, 99, 49, 48, 97, 99, 56, 97, 49, 102, 51, 48, 57, 101, 51, 100,
        101, 48, 57, 100, 101, 52, 56, 52, 99, 54, 100, 101, 49, 100, 99, 55, 99, 50, 50, 54, 101,
        50, 99, 102, 56, 101, 49, 97, 53, 49, 56, 51, 54, 57, 98, 49, 100, 55, 51, 34, 44, 34, 118,
        105, 114, 116, 117, 97, 108, 95, 109, 97, 99, 104, 105, 110, 101, 34, 58, 123, 34, 111,
        112, 101, 114, 97, 116, 105, 110, 103, 95, 115, 121, 115, 116, 101, 109, 34, 58, 34, 85,
        98, 117, 110, 116, 117, 34, 44, 34, 97, 114, 99, 104, 105, 116, 101, 99, 116, 117, 114,
        101, 34, 58, 34, 65, 114, 109, 54, 52, 34, 125, 125
    ];
    pub static ref TEST_FILE_BYTES: Vec<u8> =
        vec![13, 0, 0, 0, 83, 111, 109, 101, 32, 99, 111, 110, 116, 101, 110, 116, 10,];
    pub static ref TEST_PACKAGE_BYTES: Vec<u8> = vec![
        195, 0, 0, 0, 123, 34, 110, 97, 109, 101, 34, 58, 34, 115, 111, 109, 101, 45, 112, 97, 99,
        107, 97, 103, 101, 45, 110, 97, 109, 101, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110,
        34, 58, 34, 48, 46, 49, 46, 48, 34, 44, 34, 99, 104, 101, 99, 107, 115, 117, 109, 34, 58,
        34, 97, 97, 51, 48, 98, 49, 99, 99, 48, 53, 99, 49, 48, 97, 99, 56, 97, 49, 102, 51, 48,
        57, 101, 51, 100, 101, 48, 57, 100, 101, 52, 56, 52, 99, 54, 100, 101, 49, 100, 99, 55, 99,
        50, 50, 54, 101, 50, 99, 102, 56, 101, 49, 97, 53, 49, 56, 51, 54, 57, 98, 49, 100, 55, 51,
        34, 44, 34, 118, 105, 114, 116, 117, 97, 108, 95, 109, 97, 99, 104, 105, 110, 101, 34, 58,
        123, 34, 111, 112, 101, 114, 97, 116, 105, 110, 103, 95, 115, 121, 115, 116, 101, 109, 34,
        58, 34, 85, 98, 117, 110, 116, 117, 34, 44, 34, 97, 114, 99, 104, 105, 116, 101, 99, 116,
        117, 114, 101, 34, 58, 34, 65, 114, 109, 54, 52, 34, 125, 125, 14, 0, 0, 0, 115, 111, 109,
        101, 32, 99, 111, 110, 116, 101, 110, 116, 32, 10
    ];
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

#[derive(Default)]
pub struct TempArchiveBuilder {
    is_large: bool,
}

impl TempArchiveBuilder {
    pub fn new() -> TempArchiveBuilder {
        TempArchiveBuilder { is_large: false }
    }

    pub fn is_large(mut self, value: bool) -> Self {
        self.is_large = value;
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
        let toml_content = r#"
                [package]
                name = "test_package_1"
                description = "This package does nothing at all, and we spent 300 manhours on it..."
                version = "1.0.4"
                authors = ["Robert robert@exmaple.com", "Bobert the III bobert@exmaple.com", "Miranda Rustacean miranda@rustacean.rust" ]
                license = "Apache-2.0"
                [content]
                type = "vm"
                [virtual-machine]
                operating_system = "Ubuntu"
                architecture = "arm64"
                type = "OVA"
                file_path = "/src/test_file.txt"
            "#;
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
        let mut toml_file = Builder::new()
            .prefix("package")
            .suffix(".toml")
            .rand_bytes(0)
            .tempfile_in(&dir)?;
        toml_file.write_all(toml_content.as_bytes())?;
        if self.is_large {
            let mut large_file = Builder::new()
                .prefix("large")
                .suffix(".txt")
                .rand_bytes(0)
                .tempfile_in(&dir)?;
            let random_bytes: Vec<u8> =
                TempArchiveBuilder::generate_vec(Byte::from_str("20MB")?.get_bytes() as usize)?;
            large_file.write_all(&random_bytes)?;
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

pub fn create_readable_temporary_file(content: &str) -> Result<(File, TempPath)> {
    let file = NamedTempFile::new()?;
    let mut other_handler = file.reopen()?;
    write!(&mut other_handler, "{}", content)?;
    Ok(file.into_parts())
}

pub fn create_test_package() -> Result<Package> {
    let (temporary_file, path) = create_readable_temporary_file("some content \n")?;
    let (package_toml, toml_path) = create_readable_temporary_file("some toml \n")?;
    let (readme_file, readme_path) = create_readable_temporary_file("some readme \n")?;
    let file = PackageFile(temporary_file, Some(path));
    let package_toml = PackageFile(package_toml, Some(toml_path));
    let readme = PackageFile(readme_file, Some(readme_path));
    Ok(Package {
        index_info: TEST_INDEX_METADATA.clone(),
        file,
        package_toml,
        readme: Some(readme),
    })
}

pub fn initialize_test_repository() -> (TempDir, Repository) {
    let td = TempDir::new().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.initial_head("master");
    let repo = Repository::init_opts(td.path(), &opts).unwrap();
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "name").unwrap();
        config.set_str("user.email", "email").unwrap();
        let mut index = repo.index().unwrap();
        let id = index.write_tree().unwrap();

        let tree = repo.find_tree(id).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }
    (td, repo)
}

pub fn get_last_commit_message(repo: &Repository) -> String {
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    head.message().unwrap().to_string()
}

pub fn generate_random_string(length: usize) -> Result<String> {
    let random_bytes = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .collect::<Vec<u8>>();
    Ok(String::from_utf8(random_bytes)?)
}

pub fn get_free_port() -> Result<u16> {
    free_local_port().ok_or_else(|| anyhow!("Failed to assign free local port"))
}
