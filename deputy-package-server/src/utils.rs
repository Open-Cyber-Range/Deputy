use anyhow::Result;
use flate2::read::MultiGzDecoder;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::{fs::File, io::Read};
use tar::Archive;

pub fn get_file_content_by_path(package: DirEntry, filepath: &Path) -> Result<Vec<String>> {
    let versions = fs::read_dir(package.path())?;
    let mut result_vec: Vec<String> = Vec::new();
    for version in versions {
        let file = File::open(version?.path())?;
        let tarfile = MultiGzDecoder::new(file);
        let mut archive = Archive::new(tarfile);
        for entry in archive.entries()? {
            let mut entry = entry?;
            if entry.path()?.to_str() == filepath.to_str() {
                let mut buffer = String::new();
                entry.read_to_string(&mut buffer)?;
                result_vec.push(buffer)
            }
        }
    }
    Ok(result_vec)
}
