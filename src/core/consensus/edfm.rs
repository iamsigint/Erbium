// src/core/consensus/edfm.rs

use crate::crypto::hash::calculate_hash;
use std::collections::HashMap;

/// EDFM (Ethereum-inspired Dynamic Federated Model) consensus
/// Selects a proposer based on stake and random seed
pub fn select_proposer(seed: &str, validators: &HashMap<String, crate::core::state::ValidatorInfo>) -> Option<String> {
    if validators.is_empty() {
        return None;
    }

    // Use only the block hash as seed for consistency across nodes
    // This ensures all nodes select the same proposer
    // Convert &str to String para resolver o problema de Sized
    let seed_string = seed.to_string();
    let hash = calculate_hash(&seed_string);
    
    // Convert hash to a number
    let hash_num = u64::from_str_radix(&hash[..16], 16).unwrap_or(0);
    
    // Calculate total stake
    let total_stake: u64 = validators.values().map(|v| v.stake).sum();
    
    if total_stake == 0 {
        return None;
    }

    // Select proposer based on weighted random
    let mut cumulative_stake = 0u64;
    let target = hash_num % total_stake;
    
    for (address, validator) in validators {
        cumulative_stake += validator.stake;
        if cumulative_stake > target {
            println!("🎲 EDFM selected proposer: {} (stake: {}, target: {})", 
                     address, validator.stake, target);
            return Some(address.clone());
        }
    }
    
    // Fallback: return first validator
    validators.keys().next().cloned()
}