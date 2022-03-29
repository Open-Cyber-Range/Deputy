use crate::package::PackageMetadata;
use anyhow::{Error, Ok, Result};
use git2::{build::CheckoutBuilder, Repository, RepositoryInitOptions};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

static HEAD_REF: &str = "HEAD";
#[cfg(windows)]
static LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
static LINE_ENDING: &str = "\n";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositoryConfiguration {
    pub folder: String,
    pub username: String,
    pub email: String,
}

fn generate_package_path(package_name: &str) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    match package_name.len() {
        1 => path.push("1"),
        2 => path.push("2"),
        3 => {
            path.push("3");
            path.push(&package_name[0..1]);
        }
        _ => {
            path.push(&package_name[0..2]);
            path.push(&package_name[2..4]);
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

fn write_metadata_to_file(mut file: &File, package_metadata: &PackageMetadata) -> Result<()> {
    let mut metadata_string = serde_json::to_string(package_metadata)?;
    metadata_string.push_str(LINE_ENDING);
    file.write_all(metadata_string.as_bytes())?;
    Ok(())
}

fn create_package_commit(
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

fn find_package_file_by_name(repository: &Repository, name: &str) -> Result<Option<File>> {
    let mut full_path = repository
        .path()
        .parent()
        .ok_or_else(|| Error::msg("Repository root not found"))?
        .to_owned();
    let relative_repository_path = generate_package_path(name)?;
    full_path.push(relative_repository_path);

    let file = OpenOptions::new().read(true).open(full_path.clone()).ok();
    Ok(file)
}

pub fn find_metadata_by_package_name(
    repository: &Repository,
    name: &str,
) -> Result<Vec<PackageMetadata>> {
    let mut metadata_list = Vec::new();
    if let Some(mut file) = find_package_file_by_name(repository, name)? {
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).ok();
        for line in file_content.lines() {
            if let Result::Ok(package_metadata) = serde_json::from_str::<PackageMetadata>(line) {
                metadata_list.push(package_metadata);
            }
        }
    }
    Ok(metadata_list)
}

pub fn update_index_repository(
    repository: &Repository,
    package_metadata: &PackageMetadata,
) -> Result<()> {
    let (file, file_path) = create_or_find_package_file(repository, &package_metadata.name)?;

    write_metadata_to_file(&file, package_metadata)
        .or_else(|_| reset_repository_to_last_good_state(repository))?;
    create_package_commit(repository, &file_path, package_metadata)?;
    Ok(())
}

fn initialize_repository(repository_configuration: &RepositoryConfiguration) -> Result<Repository> {
    let mut opts = RepositoryInitOptions::new();

    opts.initial_head("master");
    let path = Path::new(&repository_configuration.folder);
    let repository = Repository::init_opts(path, &opts)?;
    {
        let mut config = repository.config()?;
        config.set_str("user.name", &repository_configuration.username)?;
        config.set_str("user.email", &repository_configuration.email)?;
        let mut index = repository.index()?;
        let id = index.write_tree()?;

        let tree = repository.find_tree(id)?;
        let sig = repository.signature()?;
        repository.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;
    }
    Ok(repository)
}

pub fn get_or_create_repository(
    repository_configuration: &RepositoryConfiguration,
) -> Result<Repository> {
    if let Result::Ok(repository) = Repository::open(repository_configuration.clone().folder) {
        return Ok(repository);
    }
    initialize_repository(repository_configuration)
}

#[cfg(test)]
mod tests {
    use crate::{
        repository::{
            create_or_find_package_file, find_metadata_by_package_name, find_package_file_by_name,
            get_or_create_repository, initialize_repository, update_index_repository,
            RepositoryConfiguration,
        },
        test::{initialize_test_repository, TEST_PACKAGE_METADATA},
    };
    use std::{
        fs::File,
        io::BufRead,
        path::{Path, PathBuf},
    };

    use super::{create_package_commit, generate_package_path, write_metadata_to_file, HEAD_REF};
    use anyhow::Result;
    use tempfile::TempDir;

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
        write_metadata_to_file(&file, &TEST_PACKAGE_METADATA)?;
        let read_file = File::open(file_path)?;
        let line = std::io::BufReader::new(read_file).lines().next().unwrap()?;

        let expected_string = "{\"name\":\"some-package-name\",\"version\":\"0.1.0\",\"checksum\":\"d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c\"}";
        assert_eq!(expected_string, line);
        temporary_dir.close()?;
        Ok(())
    }

    #[test]
    fn repository_is_created_when_none_exists() -> Result<()> {
        let temporary_directory = TempDir::new()?;
        let path = temporary_directory.path().to_str().unwrap().to_owned();
        let repository_configuration = RepositoryConfiguration {
            folder: path,
            username: String::from("some-username"),
            email: String::from("some-email"),
        };

        let repository = get_or_create_repository(&repository_configuration)?;
        let configuration = repository.config().unwrap();
        let name = configuration.get_entry("user.name").unwrap();
        let email = configuration.get_entry("user.email").unwrap();

        assert_eq!(name.value().unwrap(), "some-username");
        assert_eq!(email.value().unwrap(), "some-email");
        temporary_directory.close()?;
        Ok(())
    }

    #[test]
    fn repository_is_returned_when_one_exists() -> Result<()> {
        let (root_directory, test_repository) = initialize_test_repository();
        let path = root_directory.path().to_str().unwrap().to_owned();
        let repository_configuration = RepositoryConfiguration {
            folder: path,
            username: String::from("some-username"),
            email: String::from("some-email"),
        };

        let repository = get_or_create_repository(&repository_configuration)?;
        let configuration = repository.config().unwrap();
        let name = configuration.get_entry("user.name").unwrap();
        let email = configuration.get_entry("user.email").unwrap();

        let test_configuration = test_repository.config().unwrap();
        let test_name = test_configuration.get_entry("user.name").unwrap();
        let test_email = test_configuration.get_entry("user.email").unwrap();

        assert_eq!(name.value().unwrap(), test_name.value().unwrap());
        assert_eq!(email.value().unwrap(), test_email.value().unwrap());
        root_directory.close()?;
        Ok(())
    }

    #[test]
    fn repository_is_initialized() -> Result<()> {
        let temporary_directory = TempDir::new()?;
        let path = temporary_directory.path().to_str().unwrap().to_owned();
        let repository_configuration = RepositoryConfiguration {
            folder: path,
            username: String::from("some-username"),
            email: String::from("some-email"),
        };
        let repository = initialize_repository(&repository_configuration)?;

        let head_id = repository.refname_to_id(HEAD_REF)?;
        let parent = repository.find_commit(head_id)?;

        assert_eq!("initial", parent.message().unwrap());
        temporary_directory.close()?;
        Ok(())
    }

    #[test]
    fn non_existing_package_file_is_not_found() -> Result<()> {
        let (root_directory, repository) = initialize_test_repository();
        let file_option = find_package_file_by_name(&repository, "my-test")?;

        assert!(file_option.is_none());
        root_directory.close()?;
        Ok(())
    }

    #[test]
    fn existing_package_file_is_found() -> Result<()> {
        let (root_directory, repository) = initialize_test_repository();
        update_index_repository(&repository, &TEST_PACKAGE_METADATA)?;
        let file_option = find_package_file_by_name(&repository, &TEST_PACKAGE_METADATA.name)?;

        assert!(file_option.is_some());
        root_directory.close()?;
        Ok(())
    }

    #[test]
    fn package_metadata_is_found() -> Result<()> {
        let (root_directory, repository) = initialize_test_repository();
        update_index_repository(&repository, &TEST_PACKAGE_METADATA)?;
        let metadata_list =
            find_metadata_by_package_name(&repository, &TEST_PACKAGE_METADATA.name)?;

        insta::assert_debug_snapshot!(metadata_list);
        root_directory.close()?;
        Ok(())
    }

    #[test]
    fn correct_commit_for_package_is_created() -> Result<()> {
        let (temporary_directory, repository) = initialize_test_repository();
        let root = repository.path().parent().unwrap();
        File::create(&root.join("test"))?;

        create_package_commit(&repository, Path::new("test"), &TEST_PACKAGE_METADATA)?;
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

        update_index_repository(&repository, &TEST_PACKAGE_METADATA)?;
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
