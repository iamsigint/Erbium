// src/core/block.rs

use crate::core::transaction::Transaction;
use serde::{Deserialize, Serialize};

// THE FIX IS HERE: We need to add `, Clone` to this line.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Header {
    pub block_number: u64,
    pub prev_block_hash: String,
    pub timestamp: u64,
    pub merkle_root: String,
    pub nonce: u32,
}

// AND THE FIX IS HERE: We also add `, Clone` to this line.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

impl Block {
    // This is a general-purpose constructor for new blocks.
    pub fn new(block_number: u64, prev_block_hash: String, transactions: Vec<Transaction>) -> Self {
        let mut block = Block {
            header: Header {
                block_number,
                prev_block_hash,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                merkle_root: String::new(), // Placeholder for now
                nonce: 0,
            },
            transactions,
        };

        // We will calculate a real Merkle root later. For now, let's hash the transactions.
        block.header.merkle_root = crate::crypto::hash::calculate_hash(&block.transactions);
        block
    }

    // Creates the very first block of the chain, the Genesis Block.
    pub fn create_genesis_block() -> Self {
        let genesis_header = Header {
            block_number: 0,
            // The genesis block has no previous block, so the hash is just "0".
            prev_block_hash: "0".to_string(),
            // Using a fixed timestamp for reproducibility.
            timestamp: 1728151993, // Represents 2025-10-05 14:13:13 UTC
            merkle_root: "0".to_string(),
            nonce: 0,
        };

        Block {
            header: genesis_header,
            // The genesis block has no transactions.
            transactions: Vec::new(),
        }
    }

    // Calculates and returns the SHA-256 hash of the entire block.
    pub fn calculate_hash(&self) -> String {
        // We use our generic hash function from the crypto module.
        crate::crypto::hash::calculate_hash(self)
    }
}