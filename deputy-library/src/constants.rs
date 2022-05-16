use crate::project::SubType;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref VALID_NAME: Regex = Regex::new(r#"^[a-zA-Z0-9_-]+$"#).unwrap();
    pub static ref VALID_VM_TYPES: &'static [&'static SubType] = &[&SubType::Packer];
}

pub const CONFIG_FILE_PATH_ENV_KEY: &str = "DEPUTY_CONFIG";
pub const SHA256_LENGTH: usize = 64;
pub const PACKAGE_UPLOAD_PATH: &str = "/api/v1/package";
pub const PACKAGE_TOML: &str = "package.toml";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum OperatingSystem {
    AlmaLinux,
    AmazonLinux,
    Asianux,
    CentOS,
    Debian,
    DebianGNULinux,
    EComStation,
    Fedora,
    Flatcar,
    FreeBSD,
    KylinLinuxAdvancedServer,
    MacOs,
    MiracleLinux,
    NeoKylinLinuxAdvancedServer,
    OpenSuse,
    OracleLinux,
    OSX,
    Pardus,
    Photon,
    RedHatEnterpriseLinux,
    RockyLinux,
    SCOOpenServer,
    SCOUnixWare,
    Solaris,
    SUSELinuxEnterprise,
    Ubuntu,
    Windows10,
    Windows11,
    Windows2000,
    Windows7,
    Windows8,
    WindowsServer2003,
    WindowsServer2008,
    WindowsServer2012,
    WindowsServer2016,
    WindowsServer2019,
    WindowsServer2022,
    WindowsVista,
    WindowsXP,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Architecture {
    Amd64,
    Arm64,
    Armhf,
    I386,

    #[serde(other)]
    Unknown,
}
