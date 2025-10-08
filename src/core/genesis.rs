// src/core/genesis.rs

use crate::core::block::{Block, Header};
use crate::crypto::hash::calculate_hash;

/// Returns the fixed genesis block for all nodes in the network
/// This ensures all nodes start with the same initial state
pub fn get_genesis_block() -> Block {
    Block {
        header: Header {
            block_number: 0,
            prev_block_hash: "0".to_string(),
            timestamp: 1728151993, // Fixed timestamp: 2025-10-05 14:13:13 UTC
            merkle_root: "0".to_string(),
            nonce: 0,
        },
        transactions: vec![],
    }
}

/// Calculate the actual hash of the genesis block
/// Run this once to get the real hash, then hardcode it
pub fn calculate_genesis_hash() -> String {
    let genesis_block = get_genesis_block();
    calculate_hash(&genesis_block)
}

/// Pre-calculated hash of the genesis block for verification
/// This must be the same across all nodes in the network
pub fn get_genesis_hash() -> String {
    // Run calculate_genesis_hash() once to get this value
    // Then hardcode it here
    "51ed27efee90a9340e9da466359d062f51332706dce89d06f5cb081dede47820".to_string()
}

/// Validates if a given block matches the expected genesis block
pub fn is_valid_genesis_block(block: &Block) -> bool {
    let actual_hash = block.calculate_hash();
    let expected_hash = get_genesis_hash();
    println!("🔍 Genesis block validation - Actual: {}, Expected: {}", actual_hash, expected_hash);
    actual_hash == expected_hash
}