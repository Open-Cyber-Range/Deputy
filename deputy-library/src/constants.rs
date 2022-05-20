use fancy_regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref VALID_NAME: Regex = Regex::new(r#"^[a-zA-Z0-9_-]+$"#).unwrap();
}

pub const SHA256_LENGTH: usize = 64;

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

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Architecture {
    amd64,
    arm64,
    armhf,
    i386,

    #[serde(other)]
    Unknown,
}
