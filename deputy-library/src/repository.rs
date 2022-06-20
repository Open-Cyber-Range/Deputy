use crate::constants::{INDEX_REPOSITORY_BRANCH, INDEX_REPOSITORY_REMOTE};
use crate::package::PackageMetadata;
use anyhow::{Error, Ok, Result};
use git2::{
    build::CheckoutBuilder, AnnotatedCommit, AutotagOption, FetchOptions, Reference, Remote,
    Repository, RepositoryInitOptions,
};
use log::{debug, info};
use semver::{Version, VersionReq};
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

pub fn find_largest_matching_version(
    repository: &Repository,
    name: &str,
    version_requirement: &str,
) -> Result<Option<String>> {
    let version_requirement = VersionReq::parse(version_requirement)?;
    let metadata_list = find_metadata_by_package_name(repository, name)?;

    let mut latest_metadata = metadata_list
        .iter()
        .map(|metadata| Ok(Version::parse(&metadata.version)?))
        .collect::<Result<Vec<Version>>>()?;
    latest_metadata.sort();
    latest_metadata.reverse();
    let largest_version = latest_metadata
        .iter()
        .find(|version| version_requirement.matches(version))
        .map(|version| version.to_string());

    Ok(largest_version)
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

    opts.initial_head(INDEX_REPOSITORY_BRANCH);
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

fn fetch_commit<'a>(
    repo: &'a Repository,
    refspecs: &[&str],
    remote: &'a mut Remote,
) -> Result<AnnotatedCommit<'a>> {
    let mut fetch_options = FetchOptions::new();
    fetch_options.download_tags(AutotagOption::All);
    remote.fetch(refspecs, Some(&mut fetch_options), None)?;
    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(
    repo: &Repository,
    reference: &mut Reference,
    commit: &AnnotatedCommit,
) -> Result<()> {
    let name = match reference.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(reference.name_bytes()).to_string(),
    };
    let message = format!("Fast-Forward: Setting {} to id: {}", name, commit.id());
    reference.set_target(commit.id(), &message)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
    Ok(())
}

fn normal_merge(
    repository: &Repository,
    local: &AnnotatedCommit,
    remote: &AnnotatedCommit,
) -> Result<()> {
    let local_tree = repository.find_commit(local.id())?.tree()?;
    let remote_tree = repository.find_commit(remote.id())?.tree()?;
    let ancestor = repository
        .find_commit(repository.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut merge_index = repository.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if merge_index.has_conflicts() {
        repository.checkout_index(Some(&mut merge_index), None)?;
        return Ok(());
    }
    let result_tree = repository.find_tree(merge_index.write_tree_to(repository)?)?;

    let message = format!("Merge: {} into {}", remote.id(), local.id());
    let signature = repository.signature()?;
    let local_commit = repository.find_commit(local.id())?;
    let remote_commit = repository.find_commit(remote.id())?;
    let _merge_commit = repository.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    repository.checkout_head(None)?;
    Ok(())
}

fn merge_commit<'a>(
    repository: &'a Repository,
    remote_branch: &str,
    fetch_commit: AnnotatedCommit<'a>,
) -> Result<()> {
    let analysis = repository.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", remote_branch);
        if let core::result::Result::Ok(mut reference) = repository.find_reference(&refname) {
            fast_forward(repository, &mut reference, &fetch_commit)?;
        }
        repository.reference(
            &refname,
            fetch_commit.id(),
            true,
            &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
        )?;
        repository.set_head(&refname)?;
        repository.checkout_head(Some(
            CheckoutBuilder::default()
                .allow_conflicts(true)
                .conflict_style_merge(true)
                .force(),
        ))?;
    } else if analysis.0.is_normal() {
        let head_commit = repository.reference_to_annotated_commit(&repository.head()?)?;
        normal_merge(repository, &head_commit, &fetch_commit)?;
    }
    Ok(())
}

pub fn get_or_create_repository(
    repository_configuration: &RepositoryConfiguration,
) -> Result<Repository> {
    if let Result::Ok(repository) = Repository::open(repository_configuration.clone().folder) {
        return Ok(repository);
    }
    info!("Initializing the repository");
    initialize_repository(repository_configuration)
}

pub fn get_or_clone_repository(url: &str, target_path: PathBuf) -> Result<Repository> {
    if let Result::Ok(repository) = Repository::open(target_path.clone()) {
        return Ok(repository);
    }
    debug!("Cloning the repository from {url} at: {:?}", target_path);
    Ok(Repository::clone(url, target_path)?)
}

pub fn pull_from_remote(repository: &Repository) -> Result<()> {
    let mut remote = repository.find_remote(INDEX_REPOSITORY_REMOTE)?;
    let commit = fetch_commit(repository, &[INDEX_REPOSITORY_BRANCH], &mut remote)?;
    merge_commit(repository, INDEX_REPOSITORY_BRANCH, commit)?;
    Ok(())
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

        insta::assert_debug_snapshot!(line);
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
        insta::assert_debug_snapshot!(line);
        insta::assert_debug_snapshot!(parent.message().unwrap());

        temporary_directory.close()?;
        Ok(())
    }
}
