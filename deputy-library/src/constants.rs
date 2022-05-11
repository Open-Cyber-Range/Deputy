use crate::project::SubType;
use fancy_regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VALID_NAME: Regex = Regex::new(r#"^[a-zA-Z0-9_-]+$"#).unwrap();
    pub static ref VALID_VM_TYPES: &'static [&'static SubType] = &[&SubType::Packer];
}

pub const CONFIG_FILE_PATH_ENV_KEY: &str = "DEPUTY_CONFIG";
pub const SHA256_LENGTH: usize = 64;
pub const PACKAGE_UPLOAD_PATH: &str = "/api/v1/package";
pub const PACKAGE_TOML: &str = "package.toml";

pub const VALID_OPERATING_SYSTEMS: &[&str] = &[
    "pop-os",
    "Windows Vista SP2",
    "Windows Server 2008 SP2",
    "Windows 7",
    "Windows Server 2008 R2",
    "Windows 8",
    "Windows Server 2012",
    "Windows 8.1",
    "Windows Server 2012 R2",
    "Windows 10",
    "Windows Server 2016",
    "CentOS 6",
    "CentOS 7.0",
    "CentOS 7.1",
    "CentOS 7.2",
    "CentOS 7.3",
    "CentOS 7.4",
    "CentOS 7.5",
    "Red Hat Enterprise Linux 4",
    "Red Hat Enterprise Linux 5",
    "Red Hat Enterprise Linux 6",
    "Red Hat Enterprise Linux 7.0",
    "Red Hat Enterprise Linux 7.1",
    "Red Hat Enterprise Linux 7.2",
    "Red Hat Enterprise Linux 7.3",
    "Red Hat Enterprise Linux 7.4",
    "Red Hat Enterprise Linux 7.5",
    "SUSE Linux Enterprise Server 10",
    "SUSE Linux Enterprise Server 11",
    "Ubuntu 12.04 LTS",
    "Ubuntu 14.04 LTS",
    "Ubuntu 16.04 LTS",
];

pub const VALID_ARCHITECTURES: &[&str] = &[
    "x86_64",
    "ARM",
    "8051",
    "MCS-51",
    "PIC",
    "SPARC",
    "IBM POWER",
    "V850",
    "AVR",
    "MIPS",
    "M16C",
];
