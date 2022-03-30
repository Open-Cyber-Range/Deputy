use anyhow::{anyhow, Result};
use crate::validation;
use std::fs::File;
use ignore::{DirEntry, WalkBuilder};
use std::io::{prelude::*, Seek, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use zip::{result::ZipError, write::FileOptions, CompressionMethod};

/// Creates an archive of the given folder if it contains a valid package.toml file 
/// and saves it in "../target/package/<folder_name>.package"
pub fn create_package(root_directory: &str) -> Result<()> {
    let toml_path: PathBuf = [root_directory, "package.toml"].iter().collect();

    if !Path::new(root_directory).is_dir() {
        return Err(anyhow!(ZipError::FileNotFound));
    } else if !Path::new(&toml_path).is_file() {
        return Err(anyhow!("Missing package.toml file"));
    }

    validation::package_toml(toml_path)?;

    let destination_file_path = create_destination_file_path(root_directory)?;
    let zip_file = File::create(&destination_file_path)?;

    let mut walkdir = WalkBuilder::new(&root_directory);

    walkdir.filter_entry(|entry|!entry.path().ends_with("target"));

    zip_dir(&mut walkdir.build().filter_map(|e| e.ok()), root_directory, zip_file)?;

    Ok(())
}

fn create_destination_file_path(root_directory: &str) -> Result<PathBuf> {
    let package_full_path = std::fs::canonicalize(PathBuf::from(root_directory))?;
    let parent_directory_name = package_full_path
        .file_name()
        .ok_or_else(|| anyhow!("Invalid or root directory"))?
        .to_str()
        .ok_or_else(||anyhow!("UTF-8 conversion error"))?;
    let mut package_name = PathBuf::from(parent_directory_name);
    package_name.set_extension("package");
    
    let destination_directory: PathBuf = ["..","target","package"].iter().collect();
    let destination_file: PathBuf = [&destination_directory, &package_name].iter().collect();
        if !&destination_directory.exists() {
            std::fs::create_dir_all(destination_directory)?;
    };
    Ok(destination_file)
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Bzip2)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use tempfile::{Builder, TempDir, NamedTempFile};
    use zip_extensions::*;

    struct Project {
        root_dir: TempDir,
        target_dir: TempDir,
        _src_dir: TempDir,
        _target_file: NamedTempFile,
        _src_file: NamedTempFile,
        _toml_file: NamedTempFile,
    }
    #[test]
    fn archive_was_created() -> Result<()> {
        let temp_project = create_temp_project()?;

        let archive_path = create_archive_path(&temp_project)?;
        let root_directory_string = get_root_directory_string(&temp_project);

        create_package(root_directory_string)?;

        assert!(Path::new(&archive_path).is_file());

        temp_project.root_dir.close()?;
        fs::remove_file(archive_path)?;
        Ok(())
    }

    #[test]
    fn target_folder_exists_and_was_excluded_from_archive() -> Result<()> {
        let temp_project = create_temp_project()?;

        let archive_path = create_archive_path(&temp_project)?;
        let root_directory_string = get_root_directory_string(&temp_project);

        create_package(root_directory_string)?;

        let extraction_dir = Builder::new()
            .prefix("extracts")
            .rand_bytes(0)
            .tempdir_in(&temp_project.target_dir)
            .unwrap();

        zip_extract(&archive_path, &extraction_dir.path().to_path_buf())?;

        let target_dir = temp_project.target_dir.path().is_dir();
        let extracted_target_dir = extraction_dir.path().join("/target").exists();

        assert!(target_dir);
        assert!(!extracted_target_dir);

        temp_project.root_dir.close()?;
        fs::remove_file(archive_path)?;
        Ok(())
    }

    fn get_root_directory_string(temp_project: &Project) -> &str {
        let root_directory_string = temp_project.root_dir.path().to_str().unwrap();
        root_directory_string
    }

    fn create_archive_path(temp_project: &Project) -> Result<PathBuf, anyhow::Error> {
        let parent_directory_name = temp_project.root_dir.path()
            .file_name()
            .ok_or_else(|| anyhow!("Invalid or root directory"))?
            .to_str()
            .ok_or_else(||anyhow!("UTF-8 conversion error"))?;
        let mut package_name = PathBuf::from(parent_directory_name);
        package_name.set_extension("package");
        let destination_directory: PathBuf = ["..","target","package"].iter().collect();
        let archive_path: PathBuf = [destination_directory, package_name].iter().collect();
        Ok(archive_path)
    }

    fn create_temp_project() -> Result<Project> {
        let toml_content = 
            br#"
                [package]
                name = "test_package_1-0-4"
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
        toml_file.write_all(toml_content)?;

        let temp_project = Project {
            root_dir: dir,
            target_dir,
            _src_dir:src_dir,
            _target_file:target_file,
            _src_file:src_file,
            _toml_file:toml_file,
        };

    Ok(temp_project)
    }
}
