use anyhow::{Ok, Result};
use git2::{Repository, RepositoryInitOptions};
use std::fs::File;
use std::io::Write;
use tempfile::{Builder, TempDir};

use crate::package::{Package, PackageFile, PackageMetadata};

lazy_static! {
    pub static ref TEST_PACKAGE_METADATA: PackageMetadata = PackageMetadata {
        checksum: "d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c".to_string(),
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
