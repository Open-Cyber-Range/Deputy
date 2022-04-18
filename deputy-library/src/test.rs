use anyhow::{Ok, Result};
use git2::{Repository, RepositoryInitOptions};
use std::{io::Write, fs::File};
use tempfile::{Builder, TempDir,NamedTempFile};

use crate::package::{Package, PackageFile, PackageMetadata};

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
            121, 0, 0, 0, 123, 34, 110, 97, 109, 101, 34, 58, 34, 116, 101, 115, 116, 95, 112, 97,
            99, 107, 97, 103, 101, 95, 49, 34, 44, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58,
            34, 49, 46, 48, 46, 52, 34, 44, 34, 99, 104, 101, 99, 107, 115, 117, 109, 34, 58, 34,
            54, 100, 57, 56, 101, 100, 57, 99, 97, 48, 57, 54, 55, 49, 98, 56, 102, 52, 102, 52,
            53, 97, 56, 49, 57, 55, 100, 51, 53, 55, 101, 102, 97, 51, 50, 50, 99, 97, 98, 102,
            101, 50, 52, 57, 97, 100, 97, 98, 98, 50, 55, 52, 102, 52, 99, 55, 101, 57, 51, 99, 57,
            50, 100, 48, 34, 125, 193, 3, 0, 0, 80, 75, 3, 4, 20, 0, 0, 0, 0, 0, 243, 57, 140, 84,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 115, 114, 99, 47, 80, 75, 3, 4, 46, 0,
            0, 0, 12, 0, 243, 57, 140, 84, 191, 157, 169, 44, 133, 1, 0, 0, 203, 2, 0, 0, 17, 0, 0,
            0, 115, 114, 99, 47, 116, 101, 115, 116, 95, 102, 105, 108, 101, 46, 116, 120, 116, 66,
            90, 104, 54, 49, 65, 89, 38, 83, 89, 140, 54, 76, 226, 0, 0, 4, 87, 128, 0, 80, 64, 5,
            37, 35, 75, 0, 63, 231, 255, 64, 48, 1, 154, 9, 72, 106, 96, 32, 73, 226, 64, 141, 12,
            141, 168, 97, 130, 96, 76, 4, 52, 100, 211, 1, 41, 144, 146, 105, 163, 81, 178, 64, 3,
            32, 226, 41, 167, 16, 126, 117, 232, 228, 136, 196, 69, 196, 30, 213, 215, 176, 254,
            22, 115, 63, 52, 184, 212, 150, 187, 96, 239, 172, 78, 113, 224, 206, 115, 132, 81,
            197, 214, 215, 244, 245, 119, 160, 138, 29, 162, 232, 234, 204, 206, 109, 56, 66, 116,
            48, 100, 202, 71, 85, 162, 222, 142, 200, 74, 73, 182, 100, 52, 66, 69, 54, 178, 237,
            67, 104, 184, 237, 81, 194, 134, 33, 229, 54, 152, 107, 24, 183, 237, 18, 228, 20, 37,
            44, 123, 129, 155, 12, 17, 211, 147, 10, 201, 5, 92, 41, 187, 3, 112, 227, 26, 117, 34,
            130, 10, 242, 95, 186, 206, 153, 41, 114, 45, 51, 6, 6, 40, 47, 81, 62, 243, 24, 1,
            125, 135, 129, 22, 136, 63, 66, 192, 255, 138, 91, 183, 100, 151, 50, 11, 182, 234,
            126, 183, 198, 6, 79, 97, 85, 175, 126, 92, 206, 159, 38, 33, 11, 19, 146, 101, 156,
            180, 53, 130, 198, 108, 94, 166, 211, 204, 47, 130, 211, 194, 63, 119, 162, 82, 55,
            244, 178, 235, 5, 24, 68, 170, 16, 33, 15, 19, 112, 76, 174, 117, 84, 168, 28, 168,
            201, 239, 81, 41, 37, 108, 87, 44, 20, 152, 122, 103, 1, 245, 113, 175, 26, 242, 67,
            152, 189, 242, 19, 89, 176, 99, 233, 3, 12, 12, 12, 32, 44, 17, 57, 188, 134, 56, 190,
            30, 187, 232, 96, 107, 3, 30, 110, 18, 20, 112, 84, 137, 58, 44, 237, 19, 64, 201, 120,
            4, 44, 220, 207, 117, 137, 104, 81, 81, 177, 144, 139, 130, 248, 165, 203, 24, 138,
            153, 48, 21, 151, 137, 62, 0, 157, 1, 142, 165, 239, 128, 163, 193, 65, 117, 9, 52,
            110, 141, 83, 208, 86, 227, 25, 204, 136, 187, 22, 175, 254, 46, 228, 138, 112, 161,
            33, 24, 108, 153, 196, 80, 75, 3, 4, 46, 0, 0, 0, 12, 0, 243, 57, 140, 84, 123, 190,
            251, 217, 0, 1, 0, 0, 186, 1, 0, 0, 12, 0, 0, 0, 112, 97, 99, 107, 97, 103, 101, 46,
            116, 111, 109, 108, 66, 90, 104, 54, 49, 65, 89, 38, 83, 89, 62, 47, 223, 173, 0, 0, 1,
            223, 128, 0, 84, 80, 5, 108, 2, 80, 34, 20, 10, 190, 239, 223, 224, 48, 0, 217, 24,
            104, 73, 148, 205, 33, 226, 104, 210, 50, 25, 6, 212, 218, 130, 41, 232, 153, 54, 141,
            64, 0, 0, 0, 6, 154, 77, 77, 170, 122, 77, 52, 211, 71, 168, 104, 200, 26, 122, 128,
            131, 109, 130, 133, 36, 193, 16, 0, 200, 5, 142, 8, 20, 238, 57, 165, 14, 21, 74, 47,
            148, 222, 87, 26, 209, 216, 229, 73, 238, 206, 113, 81, 122, 98, 64, 184, 103, 17, 23,
            182, 193, 106, 213, 199, 164, 171, 98, 18, 66, 109, 120, 101, 4, 156, 1, 181, 134, 122,
            68, 32, 216, 15, 132, 199, 112, 9, 128, 188, 145, 179, 152, 10, 112, 39, 137, 116, 180,
            171, 103, 38, 158, 217, 111, 144, 65, 48, 184, 199, 121, 1, 116, 151, 236, 62, 242, 80,
            250, 106, 116, 85, 140, 226, 187, 224, 4, 110, 28, 96, 240, 202, 10, 145, 68, 130, 70,
            84, 58, 179, 11, 165, 31, 142, 46, 13, 22, 35, 240, 53, 103, 174, 159, 135, 88, 6, 192,
            197, 182, 67, 85, 144, 215, 189, 202, 58, 209, 16, 45, 34, 20, 202, 229, 129, 177, 0,
            250, 94, 24, 136, 202, 201, 28, 197, 21, 66, 106, 107, 97, 12, 3, 241, 119, 36, 83,
            133, 9, 3, 226, 253, 250, 208, 80, 75, 1, 2, 46, 3, 20, 0, 0, 0, 0, 0, 243, 57, 140,
            84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 237, 65, 0,
            0, 0, 0, 115, 114, 99, 47, 80, 75, 1, 2, 46, 3, 46, 0, 0, 0, 12, 0, 243, 57, 140, 84,
            191, 157, 169, 44, 133, 1, 0, 0, 203, 2, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            164, 129, 34, 0, 0, 0, 115, 114, 99, 47, 116, 101, 115, 116, 95, 102, 105, 108, 101,
            46, 116, 120, 116, 80, 75, 1, 2, 46, 3, 46, 0, 0, 0, 12, 0, 243, 57, 140, 84, 123, 190,
            251, 217, 0, 1, 0, 0, 186, 1, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 164, 129, 214,
            1, 0, 0, 112, 97, 99, 107, 97, 103, 101, 46, 116, 111, 109, 108, 80, 75, 5, 6, 0, 0, 0,
            0, 3, 0, 3, 0, 171, 0, 0, 0, 0, 3, 0, 0, 0, 0,
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
