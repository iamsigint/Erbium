// src/core/state.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub stake: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub validators: HashMap<String, ValidatorInfo>,
}

impl State {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
        }
    }

    // --- NEW METHOD ---
    // Adds or updates a validator in the state.
    pub fn register_validator(&mut self, address: String, stake: u64) {
        let validator_info = ValidatorInfo {
            address: address.clone(),
            stake,
        };
        // The `insert` method will add the new validator or update it if it already exists.
        self.validators.insert(address, validator_info);
        println!("State updated. Total validators: {}", self.validators.len());
    }
}