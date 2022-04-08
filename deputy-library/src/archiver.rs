use crate::{project::Project, validation};
use anyhow::{anyhow, Result};
use ignore::{DirEntry, WalkBuilder};
use std::fs::File;
use std::io::{prelude::*, Seek, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use zip::{write::FileOptions, CompressionMethod};

fn create_destination_file_path(root_directory: &Path) -> Result<PathBuf> {
    let toml_path = root_directory.join("package.toml");
    let mut file = File::open(toml_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let deserialized_toml: Project = toml::from_str(&*contents)?;
    let mut package_name = PathBuf::from(deserialized_toml.package.name);
    package_name.set_extension("package");

    let destination_directory: PathBuf = root_directory.join("target").join("package");
    let destination_file: PathBuf = [&destination_directory, &package_name].iter().collect();
    if !&destination_directory.exists() {
        std::fs::create_dir_all(destination_directory)?;
    };
    Ok(destination_file)
}

fn zip_dir<T>(
    directory_iterator: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default().compression_method(CompressionMethod::Bzip2);

    let mut buffer = Vec::new();
    for entry in directory_iterator {
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

/// Creates an archive of the given directory if it contains a valid `package.toml` file in its root
/// and returns a `PathBuf` in the form of: `<input_directory>/target/package/<package_name>.package`
///
/// The validation of the required `package.toml` file is done by calling [`validation::validate_package_toml`]
/// and the archives name is dervied from its `name` field.
///
/// Folders in the given directory are walked through and filtered using the `Ignore` crate which supports
/// ignore files such as `.gitignore` as well as global gitignore globs. However, folders as well as their contents that are hidden
/// or named `"target"` are always excluded.
///
/// # Example
/// ```ignore
/// create_package("my_project/summize/");
/// let mut output_file: PathBuf = ["target", "package", "summize"].iter().collect();
/// output_file.set_extension("package");
/// assert!(output_file.is_file());
/// ```
pub fn create_package(root_directory: PathBuf) -> Result<PathBuf> {
    if !root_directory.as_path().is_dir() {
        return Err(anyhow!("Invalid or missing directory"));
    }

    let toml_path = root_directory.join("package.toml");

    if !Path::new(&toml_path).is_file() {
        return Err(anyhow!("Missing package.toml file"));
    }

    validation::validate_package_toml(toml_path)?;

    let destination_file_path = create_destination_file_path(&root_directory)?;
    let zip_file = File::create(&destination_file_path)?;

    let mut walkdir = WalkBuilder::new(&root_directory);

    walkdir.filter_entry(|entry| !entry.path().ends_with("target"));

    let root_directory = root_directory
        .to_str()
        .ok_or_else(|| anyhow!("Path UTF-8 validation error"))?;

    zip_dir(
        &mut walkdir.build().filter_map(|e| e.ok()),
        root_directory,
        zip_file,
    )?;

    Ok(destination_file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::create_temp_project;
    use anyhow::Result;
    use tempfile::Builder;
    use zip_extensions::*;

    #[test]
    fn archive_was_created() -> Result<()> {
        let temp_project = create_temp_project()?;

        let root_directory = temp_project.root_dir.path();
        let archive_path = create_destination_file_path(root_directory)?;

        create_package(root_directory.to_path_buf())?;

        let archive = Path::new(&archive_path);
        assert!(archive.is_file());

        temp_project.root_dir.close()?;
        Ok(())
    }

    #[test]
    fn target_folder_exists_and_was_excluded_from_archive() -> Result<()> {
        let temp_project = create_temp_project()?;

        let root_directory = temp_project.root_dir.path();
        let archive_path = create_destination_file_path(root_directory)?;

        create_package(root_directory.to_path_buf())?;

        let extraction_dir = Builder::new()
            .prefix("extracts")
            .rand_bytes(0)
            .tempdir_in(&temp_project.target_dir)
            .unwrap();

        zip_extract(
            &archive_path.to_owned(), //??????? doesn't work without it anymore??
            &extraction_dir.path().to_path_buf(),
        )?;

        let target_dir_exists = temp_project.target_dir.path().is_dir();
        let extracted_target_dir_exists = extraction_dir.path().join("/target").exists();

        assert!(target_dir_exists);
        assert!(!extracted_target_dir_exists);

        temp_project.root_dir.close()?;
        Ok(())
    }
}
