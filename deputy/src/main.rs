use actix_web::web::Bytes;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use deputy_library::{
    archiver,
    package::{Package, PackageFile, PackageMetadata},
    project::Project,
};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[clap(name = "deputy")]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Publish,
}

fn get_sha256_checksum_from_file(file_pathbuf: &Path) -> Result<String> {
    let mut file = File::open(file_pathbuf)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    let checksum = format!("{:x}", sha256.finalize());

    Ok(checksum)
}
fn parse_metadata_from_package(toml_path: PathBuf) -> Result<PackageMetadata> {
    let mut file = File::open(&toml_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let deserialized_toml: Project = toml::from_str(&*contents)?;

    let metadata = PackageMetadata {
        name: deserialized_toml.package.name,
        version: deserialized_toml.package.version,
        checksum: get_sha256_checksum_from_file(&toml_path)?,
    };

    Ok(metadata)
}

async fn create_publishing_put_request(package_bytes: Vec<u8>) -> Result<()> {
    let client = reqwest::Client::new();
    // let package_bytes = Bytes::from(package_bytes);
    let response = client
        .put("http://localhost:8081/package/")
        .body(package_bytes)
        .send()
        .await?;
    println!("{:#?}", response);
    Ok(())
}

async fn get_version_request() -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8081/version")
        .send()
        .await?
        .text()
        .await?;
    println!("{:#?}", response);
    Ok(())
}
//deployer package server , dependency intergration test, dev dependency in cargo

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Publish {} => {
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

            let metadata = parse_metadata_from_package(toml_path)?;
            let file = File::open(&archive_path)?;
            let package = Package {
                metadata,
                file: PackageFile(file),
            };
            let package_bytes = Vec::try_from(package)?;

            // create_publishing_put_request(package_bytes).await?;
            get_version_request().await?;

            // by conversion comes from package::package, need to compile metadata from toml file
            // project.toml -> metadata
            // library funktsioon mis pakib metadata ja faili kokku -> impl TryFrom<PackageFile> for Vec<u8> ...

            Ok(())
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use actix_web::{body::to_bytes, test, web::Data, App};
//     use anyhow::{anyhow, Result};
//     use tempfile::{Builder, NamedTempFile, TempDir};

//     #[actix_web::test]
//     async fn successfully_add_package() -> Result<()> {
//         let (package_folder, app_state) = setup_package_server()?;
//         let app = test::init_service(App::new().app_data(app_state).service(add_package)).await;

//         let current_path_str = current_path
//             .as_path()
//             .to_str()
//             .ok_or_else(|| anyhow!("Path UTF-8 validation error"))?;
//         let archive_path = archiver::create_package(current_path_str)?;

//         let test_package = create_test_package()?;
//         let package_name = test_package.metadata.name.clone();
//         let payload = Vec::try_from(test_package)?;

//         let request = test::TestRequest::put()
//             .uri("/package")
//             .set_payload(payload)
//             .to_request();
//         let response = test::call_service(&app, request).await;
//         println!("{:#?}", response);

//         // assert!(response.status().is_success());
//         // assert!(PathBuf::from(package_folder).join(package_name).exists());
//         Ok(())
//     }
// }
