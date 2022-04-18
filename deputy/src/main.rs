use anyhow::Result;
use clap::{Parser, Subcommand};
use deputy::configuration::Configuration;
use deputy_library::package::create_and_send_package_file;
use std::env;

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let client = reqwest::Client::new();
    let api = &Configuration::get_configuration()?.repository.repositories[0].api;

    match args.command {
        Commands::Publish {} => {
            create_and_send_package_file(env::current_dir()?, client, api).await?;
            Ok(())
        }
        Commands::Version {} => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
