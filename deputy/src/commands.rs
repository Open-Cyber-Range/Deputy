use clap::{ArgEnum, Args};

#[derive(ArgEnum, Clone, Debug)]
pub enum UnpackLevel {
    Raw,
    Uncompressed,
    Regular,
}

#[derive(Debug, Args)]
pub struct FetchOptions {
    #[clap(arg_enum, short, long, default_value_t = UnpackLevel::Regular)]
    unpack_level: UnpackLevel,
    #[clap(short, long, help = "Version of the package to fetch")]
    version: Option<String>,
    #[clap(short, long, help = "Download path for the package")]
    download_path: Option<String>,
    #[clap(short, long, help = "Registry to use for package fetching")]
    registry_name: Option<String>,
}

#[derive(Debug, Args)]
pub struct PublishOptions {
    #[clap(short, long, default_value_t = 60, help = "Timeout before publish fails")]
    pub(crate) timeout: u64,
    #[clap(short, long, default_value_t = 0, help = "Compression rate before upload")]
    pub(crate) compression: u32,
}
