use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
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

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let executor = Executor::try_new(Configuration::get_configuration()?)?;

    let result = match args.command {
        Commands::Publish => executor.publish().await,
    };
    if let Err(error) = result {
        eprintln!("{} {}", "Error:".red(), error);
        std::process::exit(1);
    }

    Ok(())
}
