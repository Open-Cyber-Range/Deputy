use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use super::PackageMetadata;
use anyhow::{Error, Ok, Result};
use git2::{build::CheckoutBuilder, Repository};

static HEAD_REF: &str = "HEAD";
#[cfg(windows)]
static LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
static LINE_ENDING: &str = "\n";

fn generate_package_path(package_name: &str) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    match package_name.len() {
        1 => path.push("1"),
        2 => path.push("2"),
        3 => {
            path.push("3");
            path.push(package_name[0..1].to_string());
        }
        _ => {
            path.push(package_name[0..2].to_string());
            path.push(package_name[2..4].to_string());
        }
    };
    path.push(package_name);
    Ok(path)
}

fn create_or_find_package_file(
    repository: &Repository,
    package_name: &str,
) -> Result<(File, PathBuf)> {
    let mut full_path = repository
        .path()
        .parent()
        .ok_or_else(|| Error::msg("Repository root not found"))?
        .to_owned();
    let relative_repository_path = generate_package_path(package_name)?;
    full_path.push(relative_repository_path.clone());

    create_dir_all(
        full_path
            .parent()
            .ok_or_else(|| Error::msg("Correct package folder not found"))?,
    )?;
    Ok((
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(full_path.clone())?,
        relative_repository_path,
    ))
}

fn write_metadata_to_a_file(mut file: &File, package_metadata: &PackageMetadata) -> Result<()> {
    let mut metadata_string = serde_json::to_string(package_metadata)?;
    metadata_string.push_str(LINE_ENDING);
    file.write_all(metadata_string.as_bytes())?;
    Ok(())
}

fn create_a_package_commit(
    repository: &Repository,
    file_path: &Path,
    package_metadata: &PackageMetadata,
) -> Result<()> {
    let mut index = repository.index()?;
    index.add_path(file_path)?;
    let tree_id = index.write_tree()?;
    let tree = repository.find_tree(tree_id)?;
    let signature = repository.signature()?;
    let head_id = repository.refname_to_id(HEAD_REF)?;
    let parent = repository.find_commit(head_id)?;
    let commit_message = format!(
        "Adding package: {}, version: {}",
        &package_metadata.name, &package_metadata.version
    );
    repository.commit(
        Some(HEAD_REF),
        &signature,
        &signature,
        &commit_message,
        &tree,
        &[&parent],
    )?;

    Ok(())
}

fn reset_repository_to_last_good_state(repository: &Repository) -> Result<()> {
    let mut checkout_options = CheckoutBuilder::new();
    checkout_options.force();
    repository.checkout_head(Some(&mut checkout_options))?;
    Ok(())
}

pub fn update_index_repository(
    repository: &Repository,
    package_metadata: &PackageMetadata,
) -> Result<()> {
    let (file, file_path) = create_or_find_package_file(repository, &package_metadata.name)?;

    write_metadata_to_a_file(&file, package_metadata)
        .or_else(|_| reset_repository_to_last_good_state(repository))?;
    create_a_package_commit(repository, &file_path, package_metadata)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::BufRead,
        path::{Path, PathBuf},
    };

    use crate::{
        repository::{create_or_find_package_file, update_index_repository},
        PackageMetadata,
    };

    use super::{
        create_a_package_commit, generate_package_path, write_metadata_to_a_file, HEAD_REF,
    };
    use anyhow::Result;
    use git2::{Repository, RepositoryInitOptions};
    use tempfile::TempDir;

    fn initialize_test_repository() -> (TempDir, Repository) {
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

    #[test]
    fn correct_package_path_for_single_character_names() -> Result<()> {
        let expected_path: PathBuf = ["1", "t"].iter().collect();
        let path = generate_package_path("t")?;

        assert_eq!(expected_path, path);
        Ok(())
    }

    #[test]
    fn correct_package_path_for_two_character_names() -> Result<()> {
        let expected_path: PathBuf = ["2", "te"].iter().collect();
        let path = generate_package_path("te")?;

        assert_eq!(expected_path, path);
        Ok(())
    }

    #[test]
    fn correct_package_path_for_three_charcter_names() -> Result<()> {
        let expected_path: PathBuf = ["3", "t", "tes"].iter().collect();
        let path = generate_package_path("tes")?;

        assert_eq!(expected_path, path);
        Ok(())
    }

    #[test]
    fn correct_package_path_for_four_or_more_charcter_names() -> Result<()> {
        let expected_path: PathBuf = ["my", "-t", "my-test"].iter().collect();
        let path = generate_package_path("my-test")?;

        assert_eq!(expected_path, path);
        Ok(())
    }

    #[test]
    fn correct_file_is_created() -> Result<()> {
        let (root_directory, repository) = initialize_test_repository();
        let expected_file_path: PathBuf = ["my", "-t", "my-test"].iter().collect();
        let (file, relative_repository_path) = create_or_find_package_file(&repository, "my-test")?;

        assert!(root_directory.path().is_dir());
        assert!(file.metadata().unwrap().is_file());
        assert_eq!(expected_file_path, relative_repository_path);
        Ok(())
    }

    #[test]
    fn metadata_is_written_to_file() -> Result<()> {
        let temporary_dir = TempDir::new()?;
        let mut file_path = temporary_dir.path().to_path_buf();
        file_path.push("test-file");

        let file = File::create(&file_path)?;
        let package_metadata = PackageMetadata {
            checksum: "d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c"
                .to_string(),
            version: "0.1.0".to_string(),
            name: "some-package-name".to_string(),
        };
        write_metadata_to_a_file(&file, &package_metadata)?;
        let read_file = File::open(file_path)?;
        let line = std::io::BufReader::new(read_file).lines().next().unwrap()?;

        let expected_string = "{\"name\":\"some-package-name\",\"version\":\"0.1.0\",\"checksum\":\"d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c\"}";
        assert_eq!(expected_string, line);
        temporary_dir.close()?;
        Ok(())
    }

    #[test]
    fn correct_commit_for_package_is_created() -> Result<()> {
        let (temporary_directory, repository) = initialize_test_repository();
        let root = repository.path().parent().unwrap();
        File::create(&root.join("test"))?;

        let package_metadata = PackageMetadata {
            checksum: "d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c"
                .to_string(),
            version: "0.1.0".to_string(),
            name: "some-package-name".to_string(),
        };
        create_a_package_commit(&repository, Path::new("test"), &package_metadata)?;
        let head_id = repository.refname_to_id(HEAD_REF)?;
        let parent = repository.find_commit(head_id)?;

        assert_eq!(
            "Adding package: some-package-name, version: 0.1.0",
            parent.message().unwrap()
        );
        temporary_directory.close()?;
        Ok(())
    }

    #[test]
    fn index_repository_is_updated_with_package() -> Result<()> {
        let (temporary_directory, repository) = initialize_test_repository();
        let root = repository.path().parent().unwrap();

        let package_metadata = PackageMetadata {
            checksum: "d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c"
                .to_string(),
            version: "0.1.0".to_string(),
            name: "some-package-name".to_string(),
        };
        update_index_repository(&repository, &package_metadata)?;
        let head_id = repository.refname_to_id(HEAD_REF)?;
        let parent = repository.find_commit(head_id)?;

        let relative_path: PathBuf = ["so", "me", "some-package-name"].iter().collect();
        let read_file = File::open(root.join(relative_path))?;
        let line = std::io::BufReader::new(read_file.try_clone()?)
            .lines()
            .next()
            .unwrap()?;

        assert!(read_file.metadata()?.is_file());
        assert_eq!(
            r#"{"name":"some-package-name","version":"0.1.0","checksum":"d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c"}"#,
            line
        );
        assert_eq!(
            "Adding package: some-package-name, version: 0.1.0",
            parent.message().unwrap()
        );
        temporary_directory.close()?;
        Ok(())
    }
}
