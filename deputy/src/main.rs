use anyhow::Result;
use clap::{Parser, Subcommand};
use deputy::{
    commands::FetchOptions, configuration::Configuration, executor::Executor,
    helpers::print_error_message,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "deputy")]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Publish,
    Fetch(FetchOptions),
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let executor = Executor::try_new(Configuration::get_configuration()?)?;

    let result = match args.command {
        Commands::Publish => executor.publish().await,
        Commands::Fetch(options) => executor.fetch(options).await,
    };
    if let Err(error) = result {
        print_error_message(error);
        std::process::exit(1);
    }

    Ok(())
}
