// src/core/consensus/validator.rs

use crate::core::block::Block;

// This function checks if a new block is valid in the context of the previous block.
pub fn validate_block(new_block: &Block, prev_block: &Block) -> bool {
    // Rule 1: The block number must be sequential.
    if new_block.header.block_number != prev_block.header.block_number + 1 {
        println!("Validation Error: Invalid block number. Expected {}, got {}.", 
                 prev_block.header.block_number + 1, new_block.header.block_number);
        return false;
    }

    // Rule 2: The previous hash must match.
    let prev_block_hash = prev_block.calculate_hash();
    if new_block.header.prev_block_hash != prev_block_hash {
        println!("Validation Error: Invalid previous block hash.");
        return false;
    }

    // Rule 3: The timestamp must be greater than the previous block's.
    if new_block.header.timestamp <= prev_block.header.timestamp {
        println!("Validation Error: Timestamp is not in the future.");
        return false;
    }
    
    // All basic rules passed.
    println!("Block #{} passed validation.", new_block.header.block_number);
    true
}