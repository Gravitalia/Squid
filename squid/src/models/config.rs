use serde::Deserialize;

/// The data in the configuration file for setting up Squid.
#[derive(Deserialize, Debug)]
pub struct Config {
    pub port: Option<u16>,
}
