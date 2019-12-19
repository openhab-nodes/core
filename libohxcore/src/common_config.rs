//! # The command line configuration is defined in this module.

use std::net::{IpAddr};
use std::path::PathBuf;
use structopt::StructOpt;

const ROOT_DIR_NAME: &'static str = "ohx_root_dir";

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// The ohx root directory.
    /// Core services expect a "backups", "config", "interconnects", "certs", "webui", "rules" and "scripts" sub-directory.
    #[structopt(parse(from_os_str), short, long, env = "OHX_ROOT_DIRECTORY")]
    pub root_directory: Option<PathBuf>,

    /// The certificate and encryption keys storage directory. The default is ohx_root_directory/certs.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_CERTS_DIRECTORY")]
    pub certs_directory: Option<PathBuf>,

    /// Comma separated list of IP addresses to bind to for inter-process communication and network services.
    /// Binds to 0.0.0.0 if not set.
    #[structopt(short, long, env = "OHX_NETWORK_INTERFACES")]
    pub network_interfaces: Vec<IpAddr>,

    /// The address of the influxDB instance.
    #[structopt(long, env = "OHX_INFLUX_DB")]
    pub influx_addr: Option<IpAddr>,

    /// Comma separated list of addon registry urls. An addon registry must adhere to the Docker Registry APIv2.
    #[structopt(short, long, env = "OHX_ADDON_REGISTRIES")]
    pub addon_registries: Vec<String>,
}


impl Config {
    pub fn new() -> Config {
        Config {
            root_directory: None,
            certs_directory: None,
            network_interfaces: vec![],
            influx_addr: None,
            addon_registries: vec![]
        }
    }
    pub fn get_root_directory(&self) -> PathBuf {
        self.root_directory.clone().unwrap_or(std::env::current_dir().expect("Current dir to work").join(ROOT_DIR_NAME))
    }
    pub fn get_certs_directory(&self) -> PathBuf {
        self.certs_directory.clone().unwrap_or(self.get_root_directory().join("certs"))
    }
}