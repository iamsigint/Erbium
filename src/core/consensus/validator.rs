// src/core/consensus/validator.rs

use crate::core::block::Block;

/// Validates a new block against the previous block
pub fn validate_block(new_block: &Block, previous_block: &Block) -> bool {
    // Check block number sequence
    if new_block.header.block_number != previous_block.header.block_number + 1 {
        eprintln!("❌ Invalid block number: expected {}, got {}", 
                 previous_block.header.block_number + 1, new_block.header.block_number);
        return false;
    }
    
    // Check previous block hash
    if new_block.header.prev_block_hash != previous_block.calculate_hash() {
        eprintln!("❌ Invalid previous block hash");
        return false;
    }
    
    // Check timestamp - allow blocks from the same second (network delay)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Allow blocks from up to 10 seconds in the future
    if new_block.header.timestamp > now + 10 {
        eprintln!("❌ Timestamp is too far in the future");
        return false;
    }
    
    // Allow same timestamp for network synchronization
    if new_block.header.timestamp < previous_block.header.timestamp {
        eprintln!("❌ Timestamp is before previous block");
        return false;
    }
    
    // TODO: Add more validations (merkle root, transactions, etc.)
    
    true
}