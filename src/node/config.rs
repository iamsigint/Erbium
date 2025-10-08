// src/node/config.rs

use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub listen_address: String,
    pub bootstrap_nodes: Vec<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_str = fs::read_to_string("config/network.toml")
            .expect("Could not read network.toml file.");
        toml::from_str(&config_str)
            .expect("Could not parse network.toml.")
    }
}