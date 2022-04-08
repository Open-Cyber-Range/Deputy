use anyhow::{Ok, Result};
use git2::{Repository, RepositoryInitOptions};
use std::fs::File;
use std::io::Write;
use tempfile::{Builder, TempDir,NamedTempFile};

use crate::{package::{Package, PackageFile, PackageMetadata}};

lazy_static! {
    pub static ref TEST_PACKAGE_METADATA: PackageMetadata = PackageMetadata {
        checksum: "aa30b1cc05c10ac8a1f309e3de09de484c6de1dc7c226e2cf8e1a518369b1d73".to_string(),
        version: "0.1.0".to_string(),
        name: "some-package-name".to_string(),
    };
    pub static ref TEST_METADATA_BYTES: Vec<u8> = vec![
        123, 34, 110, 97, 109, 101, 34, 58, 34, 115, 111, 109, 101, 45, 112, 97, 99, 107, 97, 103,
        101, 45, 110, 97, 109, 101, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58, 34, 48,
        46, 49, 46, 48, 34, 44, 34, 99, 104, 101, 99, 107, 115, 117, 109, 34, 58, 34, 100, 56, 54,
        55, 48, 48, 49, 100, 98, 48, 101, 50, 98, 54, 101, 48, 52, 57, 54, 102, 57, 102, 97, 99,
        57, 54, 57, 51, 48, 101, 50, 100, 52, 50, 50, 51, 51, 101, 99, 100, 51, 99, 97, 48, 52, 49,
        51, 101, 48, 55, 53, 51, 100, 52, 99, 55, 54, 57, 53, 100, 50, 56, 57, 99, 34, 125,
    ];
    pub static ref TEST_FILE_BYTES: Vec<u8> =
        vec![13, 0, 0, 0, 83, 111, 109, 101, 32, 99, 111, 110, 116, 101, 110, 116, 10,];
    pub static ref TEST_PACKAGE_BYTES: Vec<u8> = vec![
        124, 0, 0, 0, 123, 34, 110, 97, 109, 101, 34, 58, 34, 115, 111, 109, 101, 45, 112, 97, 99,
        107, 97, 103, 101, 45, 110, 97, 109, 101, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110,
        34, 58, 34, 48, 46, 49, 46, 48, 34, 44, 34, 99, 104, 101, 99, 107, 115, 117, 109, 34, 58,
        34, 100, 56, 54, 55, 48, 48, 49, 100, 98, 48, 101, 50, 98, 54, 101, 48, 52, 57, 54, 102,
        57, 102, 97, 99, 57, 54, 57, 51, 48, 101, 50, 100, 52, 50, 50, 51, 51, 101, 99, 100, 51,
        99, 97, 48, 52, 49, 51, 101, 48, 55, 53, 51, 100, 52, 99, 55, 54, 57, 53, 100, 50, 56, 57,
        99, 34, 125, 14, 0, 0, 0, 115, 111, 109, 101, 32, 99, 111, 110, 116, 101, 110, 116, 32, 10,
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

pub fn create_readable_temporary_file(content: &str) -> Result<File> {
    let mut temporary_file = Builder::new().append(true).tempfile()?;
    write!(&mut temporary_file, "{}", content)?;
    Ok(File::open(temporary_file.path())?)
}

pub fn create_test_package() -> Result<Package> {
    let temporary_file = create_readable_temporary_file("some content \n")?;
    let file = PackageFile(temporary_file);

    Ok(Package {
        metadata: TEST_PACKAGE_METADATA.clone(),
        file,
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
pub fn create_temp_project() -> Result<TempArchive> {
        let toml_content = 
            r#"
                [package]
                name = "test_package_1"
                description = "This package does nothing at all, and we spent 300 manhours on it..."
                version = "1.0.4"
                authors = ["Robert robert@exmaple.com", "Bobert the III bobert@exmaple.com", "Miranda Rustacean miranda@rustacean.rust" ]
                [content]
                type = "vm"
                sub_type = "packer"
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
