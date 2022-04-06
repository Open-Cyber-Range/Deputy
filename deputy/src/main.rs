use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use deputy_library::{
    archiver,
    package::{Package, PackageFile},
};
use reqwest::StatusCode;
use std::{env, fs::File, path::PathBuf};

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

fn find_toml(mut toml_path: PathBuf) -> Result<PathBuf> {
    if toml_path.exists() {
        Ok(toml_path)
    } else if toml_path.pop() && toml_path.pop() {
        println!("{:?}", &toml_path);
        let toml_path = toml_path.join(PathBuf::from("package.toml"));
        Ok(find_toml(toml_path)?)
    } else {
        Err(anyhow!("Could not find package.toml"))
    }
}

async fn create_and_send_package_file() -> Result<()> {
    let current_path = env::current_dir()?;
    let package_toml = PathBuf::from("package.toml");
    let toml_path = [&current_path, &package_toml].iter().collect();
    let toml_path = find_toml(toml_path)?;
    if let Some(package_root) = toml_path.parent() {
        let archive_path = archiver::create_package(package_root.to_path_buf())?;
        let metadata = Package::parse_metadata(toml_path, &archive_path)?;
        let file = File::open(&archive_path)?;
        let package = Package {
            metadata,
            file: PackageFile(file),
        };
        let package_bytes = Vec::try_from(package)?;
        publishing_put_request(package_bytes).await?;
        Ok(())
    } else {
        Err(anyhow!("Directory error"))
    }
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
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
