use anyhow::Result;
use awc::Client;
use clap::{Parser, Subcommand};
use deputy::configuration::Configuration;
use deputy::executor::Executor;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "deputy")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Publish,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let client = Client::new();
    let executor = Executor::new(Configuration::get_configuration()?, client);

    match args.command {
        Commands::Publish => {
            executor.publish().await?;
        }
    };
    Ok(())
}
