use anyhow::Result;
use clap::{Parser, Subcommand};
use deputy::{
    commands::{
        ChecksumOptions, FetchOptions, InspectOptions, LoginOptions, NormalizeVersionOptions,
        OwnerOptions, OwnerSubcommands, PublishOptions, YankOptions,
    },
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
    Login(LoginOptions),
    Yank(YankOptions),
    Owner(OwnerOptions),
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let executor = Executor::try_new()?;

    let result = match args.command {
        Commands::Publish(options) => executor.publish(options).await,
        Commands::Fetch(options) => executor.fetch(options).await,
        Commands::Checksum(options) => executor.checksum(options).await,
        Commands::Inspect(options) => executor.inspect(options).await,
        Commands::NormalizeVersion(options) => executor.normalize_version(options).await,
        Commands::Login(options) => executor.login(options).await,
        Commands::Yank(options) => executor.yank(options).await,
        Commands::Owner(options) => match options.subcommands.clone() {
            OwnerSubcommands::Add {
                user_email,
                package_name,
            } => executor.add_owner(options, user_email, package_name).await,
            OwnerSubcommands::Remove {
                user_email,
                package_name,
            } => {
                executor
                    .remove_owner(options, user_email, package_name)
                    .await
            }
            OwnerSubcommands::List { package_name } => {
                executor.list_owners(options, package_name).await
            }
        },
    };
    if let Err(error) = result {
        print_error_message(error);
        std::process::exit(1);
    }

    Ok(())
}
