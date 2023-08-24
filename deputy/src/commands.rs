use crate::constants::{
    fetching::{DEFAULT_PACKAGE_VERSION_REQUIREMENT, DEFAULT_SAVE_PATH},
    inspecting::DEFAULT_PACKAGE_PATH,
    DEFAULT_REGISTRY_NAME,
};
use clap::{ArgEnum, Args};

#[derive(ArgEnum, Clone, Debug)]
pub enum UnpackLevel {
    Raw,
    Uncompressed,
    Regular,
}

#[derive(Debug, Args)]
pub struct FetchOptions {
    pub package_name: String,
    #[clap(arg_enum, short, long, default_value_t = UnpackLevel::Regular)]
    pub unpack_level: UnpackLevel,
    #[clap(short, long, default_value = DEFAULT_PACKAGE_VERSION_REQUIREMENT, help = "Version of the package to fetch")]
    pub version_requirement: String,
    #[clap(short, long, default_value = DEFAULT_SAVE_PATH, help = "Save path for the package")]
    pub save_path: String,
    #[clap(
        short,
        long,
        default_value = DEFAULT_REGISTRY_NAME,
        help = "Registry to use for package fetching"
    )]
    pub registry_name: String,
}

#[derive(Debug, Args)]
pub struct ChecksumOptions {
    pub package_name: String,
    #[clap(short, long, default_value = DEFAULT_PACKAGE_VERSION_REQUIREMENT, help = "Version of the package to fetch")]
    pub version_requirement: String,
    #[clap(
        short,
        long,
        default_value = DEFAULT_REGISTRY_NAME,
        help = "Registry to use for package fetching"
    )]
    pub registry_name: String,
}

#[derive(Debug, Args)]
pub struct PublishOptions {
    #[clap(
        short,
        long,
        default_value_t = 300,
        help = "Timeout before publish fails"
    )]
    pub(crate) timeout: u64,
    #[clap(
        short,
        long,
        default_value_t = 0,
        help = "Compression rate before upload"
    )]
    pub(crate) compression: u32,
    #[clap(
        short,
        long,
        default_value = DEFAULT_REGISTRY_NAME,
        help = "Registry to use for publishing"
    )]
    pub registry_name: String,
}

#[derive(Debug, Args)]
pub struct InspectOptions {
    #[clap(short, long, default_value = DEFAULT_PACKAGE_PATH, help = "Path for the package")]
    pub package_path: String,
    #[clap(long, help = "Pretty print output")]
    pub pretty: bool,
}

#[derive(Debug, Args)]
pub struct NormalizeVersionOptions {
    pub package_name: String,
    #[clap(short, long, default_value = DEFAULT_PACKAGE_VERSION_REQUIREMENT, help = "Version of the package to fetch")]
    pub version_requirement: String,
    #[clap(
        short,
        long,
        default_value = DEFAULT_REGISTRY_NAME,
        help = "Registry to use for versioning"
    )]
    pub registry_name: String,
}

#[derive(Debug, Args)]
pub struct LoginOptions {
    #[clap(
        short,
        long,
        default_value = DEFAULT_REGISTRY_NAME,
        help = "Registry to use for publishing"
    )]
    pub registry_name: String,
}

#[derive(Debug, Args)]
pub struct YankOptions {
    pub package_name: String,
    #[clap(short, long, help = "Version of the package to yank")]
    pub version_requirement: String,
    #[clap(
    short,
    long,
    default_value = DEFAULT_REGISTRY_NAME,
    help = "Registry to use for package fetching"
    )]
    pub registry_name: String,
    #[clap(short, long, help = "Undo yank?")]
    pub undo: bool,
}
