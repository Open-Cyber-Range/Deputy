use anyhow::Result;
use clap::{Parser, Subcommand};
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
    match args.command {
        Commands::Publish {} => {
            create_and_send_package_file(env::current_dir()?).await?;
            Ok(())
        }
        Commands::Version {} => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
