use crate::{constants::COMPERSSION_CHUNK_SIZE, project::Project, validation};
use anyhow::{anyhow, Result};
use gzp::{
    deflate::Mgzip,
    par::{
        compress::{ParCompress, ParCompressBuilder},
        decompress::{ParDecompress, ParDecompressBuilder},
    },
    Compression, ZWriter,
};
use ignore::{DirEntry, WalkBuilder};
use std::fs::{create_dir_all, remove_file, rename, File};
use std::io::{prelude::*, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};

fn get_destination_file_path(toml_path: &PathBuf) -> Result<PathBuf> {
    let mut file = File::open(toml_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let deserialized_toml: Project = toml::from_str(&contents)?;

    let root_directory = toml_path
        .parent()
        .ok_or_else(|| anyhow!("Could not find root directory"))?;
    let destination_directory: PathBuf = root_directory.join("target/package");
    create_dir_all(destination_directory.clone())?;
    let mut destination_file = destination_directory.join(deserialized_toml.package.name);
    destination_file.set_extension("package");

    Ok(destination_file)
}

type ParallelCompression = ParCompress<Mgzip>;
type ParallelDecompression = ParDecompress<Mgzip>;

pub fn decompress_archive(compressed_file_path: &Path) -> Result<PathBuf> {
    let archive_path = compressed_file_path.with_extension("tar");
    let mut archive_file = File::create(&archive_path)?;
    let compressed_file = File::open(compressed_file_path)?;
    let mut parallel_decompressor: ParallelDecompression = ParDecompressBuilder::new()
        .num_threads(num_cpus::get())?
        .from_reader(compressed_file);
    let mut buffer = Vec::with_capacity(COMPERSSION_CHUNK_SIZE);
    loop {
        let mut limit = (&mut parallel_decompressor).take(COMPERSSION_CHUNK_SIZE as u64);
        limit.read_to_end(&mut buffer)?;
        if buffer.is_empty() {
            break;
        }
        archive_file.write_all(&buffer)?;
        buffer.clear();
    }
    Ok(archive_path)
}

fn compress_archive(archive_path: &Path) -> Result<PathBuf> {
    let archive_file = File::open(&archive_path)?;
    let compressed_file_path = archive_path.with_extension("tar.gz");
    let compressed_file = File::create(&compressed_file_path)?;
    let mut parallel_compressor: ParallelCompression = ParCompressBuilder::new()
        .num_threads(num_cpus::get())?
        .compression_level(Compression::none())
        .from_writer(compressed_file);
    let mut buffer = Vec::with_capacity(COMPERSSION_CHUNK_SIZE);
    loop {
        let mut limit = (&archive_file).take(COMPERSSION_CHUNK_SIZE as u64);
        limit.read_to_end(&mut buffer)?;
        if buffer.is_empty() {
            break;
        }
        parallel_compressor.write_all(&buffer)?;
        buffer.clear();
    }
    parallel_compressor.finish()?;
    Ok(compressed_file_path)
}

fn create_archive(
    directory_iterator: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    destination_file_path: &Path,
) -> Result<PathBuf> {
    let archive_path = destination_file_path.with_extension("tar");
    let destination_file = File::create(&archive_path)?;
    let mut archiver = Builder::new(destination_file);

    for entry in directory_iterator {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        if path.is_file() {
            archiver.append_path_with_name(path, name)?;
        }
    }

    archiver.finish()?;
    Ok(archive_path)
}

pub fn unpack_archive(archive_path: &Path, destination: &Path) -> Result<()> {
    let archive_file = File::open(archive_path)?;
    let mut archiver = Archive::new(archive_file);
    create_dir_all(destination)?;
    archiver.unpack(destination)?;

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
pub fn create_package(toml_path: &PathBuf) -> Result<PathBuf> {
    let root_directory = toml_path
        .parent()
        .ok_or_else(|| anyhow!("Invalid or missing directory"))?
        .to_owned();
    validation::validate_package_toml(toml_path)?;

    let mut walkdir = WalkBuilder::new(&root_directory);
    walkdir.filter_entry(|entry| !entry.path().ends_with("target"));

    let root_directory = root_directory
        .to_str()
        .ok_or_else(|| anyhow!("Path UTF-8 validation error"))?;

    let destination_file_path = get_destination_file_path(toml_path)?;
    let archive_path = create_archive(
        &mut walkdir.build().filter_map(|e| e.ok()),
        root_directory,
        &destination_file_path,
    )?;
    let compressed_file_path = compress_archive(&archive_path)?;
    remove_file(&archive_path)?;
    rename(&compressed_file_path, &destination_file_path)?;

    Ok(destination_file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TempArchive;
    use anyhow::Result;
    use tar::Archive;
    use tempfile::Builder;

    #[test]
    fn archive_was_created() -> Result<()> {
        let temp_project = TempArchive::builder().build()?;

        let toml_file_path = temp_project.toml_file.path().to_path_buf();
        let archive_path = get_destination_file_path(&toml_file_path)?;

        create_package(&toml_file_path)?;

        let archive = Path::new(&archive_path);
        assert!(archive.is_file());

        temp_project.root_dir.close()?;
        Ok(())
    }

    #[test]
    fn target_folder_exists_and_was_excluded_from_archive() -> Result<()> {
        let temp_project = TempArchive::builder().build()?;
        let toml_file_path = temp_project.toml_file.path().to_path_buf();

        let compressed_file_path = create_package(&toml_file_path)?;
        let archive_path = decompress_archive(&compressed_file_path)?;
        let extraction_dir = Builder::new()
            .prefix("extracts")
            .rand_bytes(0)
            .tempdir_in(&temp_project.target_dir)
            .unwrap();
        let mut archive = Archive::new(File::open(&archive_path)?);
        archive.unpack(extraction_dir.path())?;

        let target_dir_exists = temp_project.target_dir.path().is_dir();
        let extracted_target_dir_exists = extraction_dir.path().join("/target").exists();

        assert!(target_dir_exists);
        assert!(!extracted_target_dir_exists);

        temp_project.root_dir.close()?;
        Ok(())
    }
}
