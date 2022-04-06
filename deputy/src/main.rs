use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use deputy_library::{
    archiver,
    package::{Package, PackageFile, PackageMetadata},
    project::Project,
};
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[clap(name = "deputy")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Publish,
    Version,
}

fn get_sha256_checksum_from_archive(file_pathbuf: &Path) -> Result<String> {
    let mut file = File::open(file_pathbuf)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    let checksum = format!("{:x}", sha256.finalize());

    Ok(checksum)
}
fn parse_metadata_from_package(toml_path: PathBuf, archive_path: &Path) -> Result<PackageMetadata> {
    let mut file = File::open(&toml_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let deserialized_toml: Project = toml::from_str(&*contents)?;

    let metadata = PackageMetadata {
        name: deserialized_toml.package.name,
        version: deserialized_toml.package.version,
        checksum: get_sha256_checksum_from_archive(archive_path)?,
    };

    Ok(metadata)
}

async fn publishing_put_request(package_bytes: Vec<u8>) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .put("http://localhost:8080/api/v1/package")
        .body(package_bytes)
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => {
            println!("Package uploaded successfully");
        }
        _ => {
            println!("{}", response.text().await?)
        }
    }

    Ok(())
}

async fn create_and_send_package_file() -> Result<()> {
    let current_path = env::current_dir()?;
    let package_toml = PathBuf::from("package.toml");
    let toml_path: PathBuf = [&current_path, &package_toml].iter().collect();
    if !toml_path.exists() {
        return Err(anyhow!(
            "Missing package.toml file from: {:?}",
            &current_path
        ));
    }
    let current_path_str = current_path
        .as_path()
        .to_str()
        .ok_or_else(|| anyhow!("Path UTF-8 validation error"))?;
    let archive_path = archiver::create_package(current_path_str)?;
    let metadata = parse_metadata_from_package(toml_path, &archive_path)?;
    let file = File::open(&archive_path)?;
    let package = Package {
        metadata,
        file: PackageFile(file),
    };
    let package_bytes = Vec::try_from(package)?;
    publishing_put_request(package_bytes).await?;
    Ok(())
}

async fn get_version() -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/version")
        .send()
        .await?
        .text()
        .await?;
    println!("{}", response);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Publish {} => {
            create_and_send_package_file().await?;
            Ok(())
        }
        Commands::Version {} => {
            get_version().await?;
            Ok(())
        }
    }
}
