use anyhow::{Ok, Result};
use git2::{Repository, RepositoryInitOptions};
use std::{io::Write, fs::File};
use tempfile::{Builder, TempDir,NamedTempFile};

use crate::package::{Package, PackageFile, PackageMetadata};
use crate::project::VirtualMachine;

lazy_static! {
    pub static ref TEST_PACKAGE_METADATA: PackageMetadata = PackageMetadata {
        checksum: "aa30b1cc05c10ac8a1f309e3de09de484c6de1dc7c226e2cf8e1a518369b1d73".to_string(),
        version: "0.1.0".to_string(),
        name: "some-package-name".to_string(),
        virtual_machine: Some(VirtualMachine{
            operating_system: "pop-os".to_string(),
            architecture: "x86_64".to_string(),
        })
    };
    pub static ref TEST_METADATA_BYTES: Vec<u8> = vec![123, 34, 110, 97, 109, 101, 34, 58, 34, 115, 111, 109, 101, 45, 112, 97, 99, 
        107, 97, 103, 101, 45, 110, 97, 109, 101, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58, 34, 48, 46, 49, 46, 48, 34, 44, 
        34, 99, 104, 101, 99, 107, 115, 117, 109, 34, 58, 34, 97, 97, 51, 48, 98, 49, 99, 99, 48, 53, 99, 49, 48, 97, 99, 56, 97, 49, 
        102, 51, 48, 57, 101, 51, 100, 101, 48, 57, 100, 101, 52, 56, 52, 99, 54, 100, 101, 49, 100, 99, 55, 99, 50, 50, 54, 101, 50, 99, 
        102, 56, 101, 49, 97, 53, 49, 56, 51, 54, 57, 98, 49, 100, 55, 51, 34, 44, 34, 118, 105, 114, 116, 117, 97, 108, 95, 109, 97, 99, 
        104, 105, 110, 101, 34, 58, 123, 34, 111, 112, 101, 114, 97, 116, 105, 110, 103, 95, 115, 121, 115, 116, 101, 109, 34, 58, 34, 112, 
        111, 112, 45, 111, 115, 34, 44, 34, 97, 114, 99, 104, 105, 116, 101, 99, 116, 117, 114, 101, 34, 58, 34, 120, 56, 54, 95, 54, 52, 
        34, 125, 125];
    pub static ref TEST_FILE_BYTES: Vec<u8> =
        vec![13, 0, 0, 0, 83, 111, 109, 101, 32, 99, 111, 110, 116, 101, 110, 116, 10,];
    pub static ref TEST_PACKAGE_BYTES: Vec<u8> = vec![193, 0, 0, 0, 123, 34, 110, 97, 109, 101, 34, 58, 34, 116, 101, 115, 116, 95, 
        112, 97, 99, 107, 97, 103, 101, 95, 49, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58, 34, 49, 46, 48, 46, 52, 34, 44, 34, 
        99, 104, 101, 99, 107, 115, 117, 109, 34, 58, 34, 56, 99, 99, 54, 55, 51, 57, 57, 48, 57, 57, 48, 49, 98, 56, 48, 53, 57, 54, 98, 
        48, 54, 48, 100, 51, 101, 55, 49, 54, 51, 55, 49, 56, 52, 100, 48, 54, 100, 100, 100, 102, 55, 49, 54, 100, 99, 48, 56, 55, 52, 
        56, 53, 54, 101, 52, 100, 55, 101, 97, 101, 97, 97, 52, 102, 34, 44, 34, 118, 105, 114, 116, 117, 97, 108, 95, 109, 97, 99, 104, 
        105, 110, 101, 34, 58, 123, 34, 111, 112, 101, 114, 97, 116, 105, 110, 103, 95, 115, 121, 115, 116, 101, 109, 34, 58, 34, 112, 
        111, 112, 45, 111, 115, 34, 44, 34, 97, 114, 99, 104, 105, 116, 101, 99, 116, 117, 114, 101, 34, 58, 34, 120, 56, 54, 95, 54, 52, 
        34, 125, 125, 241, 3, 0, 0, 80, 75, 3, 4, 20, 0, 0, 0, 0, 0, 242, 99, 171, 84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 
        115, 114, 99, 47, 80, 75, 3, 4, 46, 0, 0, 0, 12, 0, 242, 99, 171, 84, 191, 157, 169, 44, 133, 1, 0, 0, 203, 2, 0, 0, 17, 0, 0, 0, 
        115, 114, 99, 47, 116, 101, 115, 116, 95, 102, 105, 108, 101, 46, 116, 120, 116, 66, 90, 104, 54, 49, 65, 89, 38, 83, 89, 140, 54, 
        76, 226, 0, 0, 4, 87, 128, 0, 80, 64, 5, 37, 35, 75, 0, 63, 231, 255, 64, 48, 1, 154, 9, 72, 106, 96, 32, 73, 226, 64, 141, 12, 
        141, 168, 97, 130, 96, 76, 4, 52, 100, 211, 1, 41, 144, 146, 105, 163, 81, 178, 64, 3, 32, 226, 41, 167, 16, 126, 117, 232, 228, 
        136, 196, 69, 196, 30, 213, 215, 176, 254, 22, 115, 63, 52, 184, 212, 150, 187, 96, 239, 172, 78, 113, 224, 206, 115, 132, 81, 
        197, 214, 215, 244, 245, 119, 160, 138, 29, 162, 232, 234, 204, 206, 109, 56, 66, 116, 48, 100, 202, 71, 85, 162, 222, 142, 200, 
        74, 73, 182, 100, 52, 66, 69, 54, 178, 237, 67, 104, 184, 237, 81, 194, 134, 33, 229, 54, 152, 107, 24, 183, 237, 18, 228, 20, 37, 
        44, 123, 129, 155, 12, 17, 211, 147, 10, 201, 5, 92, 41, 187, 3, 112, 227, 26, 117, 34, 130, 10, 242, 95, 186, 206, 153, 41, 114, 
        45, 51, 6, 6, 40, 47, 81, 62, 243, 24, 1, 125, 135, 129, 22, 136, 63, 66, 192, 255, 138, 91, 183, 100, 151, 50, 11, 182, 234, 126, 
        183, 198, 6, 79, 97, 85, 175, 126, 92, 206, 159, 38, 33, 11, 19, 146, 101, 156, 180, 53, 130, 198, 108, 94, 166, 211, 204, 47, 
        130, 211, 194, 63, 119, 162, 82, 55, 244, 178, 235, 5, 24, 68, 170, 16, 33, 15, 19, 112, 76, 174, 117, 84, 168, 28, 168, 201, 239, 
        81, 41, 37, 108, 87, 44, 20, 152, 122, 103, 1, 245, 113, 175, 26, 242, 67, 152, 189, 242, 19, 89, 176, 99, 233, 3, 12, 12, 12, 32, 
        44, 17, 57, 188, 134, 56, 190, 30, 187, 232, 96, 107, 3, 30, 110, 18, 20, 112, 84, 137, 58, 44, 237, 19, 64, 201, 120, 4, 44, 220, 
        207, 117, 137, 104, 81, 81, 177, 144, 139, 130, 248, 165, 203, 24, 138, 153, 48, 21, 151, 137, 62, 0, 157, 1, 142, 165, 239, 128, 
        163, 193, 65, 117, 9, 52, 110, 141, 83, 208, 86, 227, 25, 204, 136, 187, 22, 175, 254, 46, 228, 138, 112, 161, 33, 24, 108, 153, 
        196, 80, 75, 3, 4, 46, 0, 0, 0, 12, 0, 242, 99, 171, 84, 167, 77, 17, 131, 48, 1, 0, 0, 48, 2, 0, 0, 12, 0, 0, 0, 112, 97, 99, 
        107, 97, 103, 101, 46, 116, 111, 109, 108, 66, 90, 104, 54, 49, 65, 89, 38, 83, 89, 188, 144, 14, 133, 0, 0, 1, 223, 128, 0, 84, 
        80, 7, 109, 66, 80, 34, 20, 10, 190, 239, 223, 224, 48, 1, 26, 139, 67, 83, 73, 232, 213, 30, 131, 210, 70, 158, 137, 166, 131, 
        212, 6, 134, 130, 167, 169, 234, 52, 122, 152, 154, 26, 0, 0, 0, 2, 83, 72, 167, 162, 122, 65, 61, 49, 170, 109, 67, 210, 7, 168, 
        122, 106, 122, 136, 70, 144, 112, 113, 37, 6, 102, 98, 32, 220, 234, 16, 218, 102, 118, 205, 54, 139, 41, 78, 170, 253, 241, 170, 
        125, 20, 217, 12, 167, 44, 220, 217, 44, 145, 163, 99, 57, 144, 100, 71, 145, 154, 4, 212, 182, 5, 107, 177, 235, 10, 114, 113, 
        241, 209, 192, 10, 134, 53, 208, 139, 137, 44, 153, 114, 36, 172, 126, 185, 226, 64, 165, 225, 163, 107, 60, 44, 31, 204, 169, 184, 
        194, 15, 45, 66, 54, 182, 228, 24, 30, 94, 147, 177, 167, 95, 30, 10, 125, 250, 109, 144, 251, 214, 165, 197, 106, 231, 218, 129, 
        125, 37, 248, 76, 163, 66, 107, 245, 222, 60, 199, 193, 91, 173, 220, 165, 122, 222, 23, 112, 180, 84, 170, 211, 190, 254, 196, 
        158, 44, 231, 84, 219, 50, 141, 102, 237, 180, 183, 12, 34, 32, 53, 181, 106, 181, 100, 218, 211, 46, 211, 252, 49, 125, 192, 236, 
        184, 129, 81, 56, 210, 148, 57, 129, 243, 229, 184, 200, 222, 34, 243, 194, 6, 76, 144, 198, 167, 204, 180, 54, 103, 5, 161, 6, 214, 
        132, 161, 246, 20, 72, 77, 201, 2, 61, 32, 71, 98, 148, 17, 152, 132, 63, 139, 185, 34, 156, 40, 72, 94, 72, 7, 66, 128, 80, 75, 
        1, 2, 46, 3, 20, 0, 0, 0, 0, 0, 242, 99, 171, 84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 237, 65, 
        0, 0, 0, 0, 115, 114, 99, 47, 80, 75, 1, 2, 46, 3, 46, 0, 0, 0, 12, 0, 242, 99, 171, 84, 191, 157, 169, 44, 133, 1, 0, 0, 203, 2, 
        0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 164, 129, 34, 0, 0, 0, 115, 114, 99, 47, 116, 101, 115, 116, 95, 102, 105, 108, 101, 
        46, 116, 120, 116, 80, 75, 1, 2, 46, 3, 46, 0, 0, 0, 12, 0, 242, 99, 171, 84, 167, 77, 17, 131, 48, 1, 0, 0, 48, 2, 0, 0, 12, 0, 
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 164, 129, 214, 1, 0, 0, 112, 97, 99, 107, 97, 103, 101, 46, 116, 111, 109, 108, 80, 75, 5, 6, 0, 0, 
        0, 0, 3, 0, 3, 0, 171, 0, 0, 0, 48, 3, 0, 0, 0, 0];
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
                [virtual-machine]
                operating_system = "pop-os"
                architecture = "x86_64"
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
