// src/core/consensus/edfm.rs

use crate::core::state::ValidatorInfo;
use std::collections::HashMap;
use crate::crypto::hash::calculate_hash;

// Selects a proposer for the current slot based on a seed.
// This is a simple, deterministic lottery.
pub fn select_proposer(seed: &str, validators: &HashMap<String, ValidatorInfo>) -> Option<String> {
    if validators.is_empty() {
        return None;
    }

    // 1. Get a list of validator addresses and sort them to ensure consistent order.
    let mut sorted_validators: Vec<_> = validators.keys().collect();
    sorted_validators.sort();

    // 2. Create a "lottery number" from the seed.
    let hash = calculate_hash(&seed);
    // We take the first 8 bytes of the hash and convert them to a u64 number.
    let hash_as_u64 = u64::from_str_radix(&hash[..16], 16).unwrap_or(0);

    // 3. The winner is determined by the modulo operator.
    let winner_index = hash_as_u64 as usize % sorted_validators.len();
    let winner_address = sorted_validators[winner_index];

    Some(winner_address.clone())
}