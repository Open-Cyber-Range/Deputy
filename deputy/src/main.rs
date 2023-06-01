use anyhow::Result;
use clap::{Parser, Subcommand};
use deputy::{
    commands::{
        ChecksumOptions, FetchOptions, InspectOptions, NormalizeVersionOptions, PublishOptions,
    },
    configuration::Configuration,
    executor::Executor,
    helpers::print_error_message,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "deputy")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Publish(PublishOptions),
    Fetch(FetchOptions),
    Checksum(ChecksumOptions),
    Inspect(InspectOptions),
    NormalizeVersion(NormalizeVersionOptions),
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let executor = Executor::new(Configuration::get_configuration()?);

    let result = match args.command {
        Commands::Publish(options) => executor.publish(options).await,
        Commands::Fetch(options) => executor.fetch(options).await,
        Commands::Checksum(options) => executor.checksum(options).await,
        Commands::Inspect(options) => executor.inspect(options).await,
        Commands::NormalizeVersion(options) => executor.normalize_version(options).await,
    };
    if let Err(error) = result {
        print_error_message(error);
        std::process::exit(1);
    }

    Ok(())
}
