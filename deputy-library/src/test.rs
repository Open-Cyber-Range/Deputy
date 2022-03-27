use std::fs::File;
use std::io::Write;

use anyhow::{Ok, Result};
use tempfile::Builder;

use crate::package::{Package, PackageFile, PackageMetadata};

lazy_static! {
    pub static ref TEST_PACKAGE_METADATA: PackageMetadata = PackageMetadata {
        checksum: "d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c".to_string(),
        version: "0.1.0".to_string(),
        name: "some-package-name".to_string(),
    };
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
