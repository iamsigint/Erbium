// src/node/config.rs

use serde::Deserialize;
use std::fs;

// This struct maps directly to the structure of our TOML file.
#[derive(Deserialize, Debug)]
pub struct Config {
    pub bootstrap_nodes: Vec<String>,
}

impl Config {
    // Loads the configuration from the `network.toml` file.
    pub fn load() -> Self {
        let config_str = fs::read_to_string("config/network.toml")
            .expect("Could not read network.toml file.");
        toml::from_str(&config_str)
            .expect("Could not parse network.toml.")
    }
}