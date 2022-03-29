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
pub fn create_package(src_dir: &str) -> Result<()> {
    let toml_path = src_dir.to_string() + "/package.toml";

    if !Path::new(src_dir).is_dir() {
        return Err(anyhow!(ZipError::FileNotFound));
    } else if !Path::new(&toml_path).is_file() {
        return Err(anyhow!("Missing package.toml file"));
    }

    validation::package_toml(toml_path)?;

    let package_full_path = std::fs::canonicalize(PathBuf::from(src_dir))?;
    let package_name = package_full_path
        .to_str()
        .unwrap()
        .split('/')
        .last()
        .unwrap()
        .to_owned()
        + ".package";

    let destination_dir_str = "../target/package/".to_string();
    let destination_file_str = destination_dir_str.clone() + package_name.as_str();
    let destination_file = Path::new(&destination_file_str);

    if !Path::new(&destination_dir_str).exists() {
        std::fs::create_dir_all("../target/package/")?;
    };
    let file = File::create(&destination_file)?;
    let mut walkdir = WalkBuilder::new(&src_dir);

    walkdir.filter_entry(|entry|!entry.path().ends_with("target"));

    zip_dir(&mut walkdir.build().filter_map(|e| e.ok()), src_dir, file)?;

    let archive_size = std::fs::metadata(destination_file)?.len();
    println!("Created archive: {:?} ({} bytes)", destination_file_str, archive_size);
    Ok(())
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
        let dir = temp_project.root_dir;
    
        let dir_str = &dir.into_path().to_str().unwrap().to_owned();
        let package_name = dir_str.split('/').last().unwrap().to_owned() + ".package";
        let archive_path = "../target/package/".to_string() + &package_name;
        
        create_package(dir_str)?;

        assert!(Path::new(&archive_path).is_file());

        fs::remove_file(archive_path)?;
        Ok(())
    }

    #[test]
    fn target_folder_exists_and_was_excluded_from_archive() -> Result<()> {
        let temp_project = create_temp_project()?;
        let target_dir = temp_project.target_dir.path().is_dir();

        let dir_str = temp_project.root_dir.into_path().to_str().unwrap().to_owned();
        let package_name = dir_str.split('/').last().unwrap().to_owned() + ".package";
        let archive_path = "../target/package/".to_string() + &package_name;

        create_package(&dir_str)?;

        let temp_extract_dir = extract_archive(Path::new(&archive_path));
        let temp_extract_dir = temp_extract_dir.path().join("/target").exists();

        assert!(target_dir);
        assert!(!temp_extract_dir);

        fs::remove_file(archive_path)?;
        Ok(())
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

        let dir = TempDir::new()?;
        let target_dir = Builder::new()
            .prefix("target")
            .rand_bytes(0)
            .tempdir_in(&dir)?;
        let target_file = Builder::new()
            .prefix("test_target_file")
            .suffix(".txt")
            .rand_bytes(0)
            .tempfile_in(&target_dir)?;
        let src_dir = Builder::new()
            .prefix("src")
            .rand_bytes(0)
            .tempdir_in(&dir)?;
        let src_file = Builder::new()
            .prefix("test_file")
            .suffix(".txt")
            .rand_bytes(0)
            .tempfile_in(&src_dir)?;
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

    fn extract_archive(zip_path: &Path) -> TempDir {
        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let extraction_dir = Builder::new()
            .prefix("extracts")
            .rand_bytes(0)
            .tempdir()
            .unwrap();

        let extraction_dir_path = &extraction_dir.path();
        let extraction_dir_str = extraction_dir_path.as_os_str();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let filename = file.enclosed_name().unwrap();
            let mut outpath = PathBuf::from(extraction_dir_str);
            outpath.push(filename);

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
    extraction_dir
    }
}
