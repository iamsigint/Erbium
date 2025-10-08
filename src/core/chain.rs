// src/core/chain.rs

use crate::core::block::Block;
use crate::core::consensus::validator;
use crate::core::state::State;
use crate::core::genesis;
use crate::storage::db::Storage;

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    storage: Storage,
    pub state: State,
}

impl Blockchain {
    pub fn new() -> Self {
        let storage = Storage::new();

        if let Some(tip_hash) = storage.get_tip_hash() {
            println!("Found existing blockchain. Loading from disk...");
            
            // Load all blocks from storage to reconstruct the chain
            let mut blocks = Vec::new();
            let mut current_hash = tip_hash.clone();
            
            while let Some(block) = storage.read_block(&current_hash) {
                blocks.push(block.clone());
                if block.header.block_number == 0 {
                    break; // Reached genesis block
                }
                current_hash = block.header.prev_block_hash.clone();
            }
            
            // Reverse to get blocks in correct order (genesis first)
            blocks.reverse();
            let state = storage.read_state().unwrap_or_else(State::new);
            
            println!("Loaded blockchain with {} blocks from storage", blocks.len());
            
            Self { blocks, storage, state }
        } else {
            println!("No existing blockchain found. Creating Genesis Block...");
            
            // Use shared genesis block to ensure network consistency
            let genesis_block = genesis::get_genesis_block();
            let state = State::new();
            
            // Critical: Verify we're creating the correct genesis block
            if !genesis::is_valid_genesis_block(&genesis_block) {
                panic!("❌ CRITICAL: Genesis block validation failed!");
            }
            
            storage.write_block(&genesis_block);
            storage.write_state(&state);
            
            Self {
                blocks: vec![genesis_block],
                storage,
                state,
            }
        }
    }

    /// Adds a new block to the blockchain after validation
    pub fn add_block(&mut self, block: Block) -> bool {
        let last_block = self.blocks.last().expect("Blockchain is empty!");
        
        // Validate block number sequence
        let expected_block_number = last_block.header.block_number + 1;
        if block.header.block_number != expected_block_number {
            eprintln!("Validation Error: Invalid block number. Expected {}, got {}", 
                     expected_block_number, block.header.block_number);
            return false;
        }
        
        // DEBUG: Show hash comparison for better troubleshooting
        let last_block_hash = last_block.calculate_hash();
        println!("🔍 Block validation - Last block hash: {}, New block prev_hash: {}", 
                 &last_block_hash[..8], &block.header.prev_block_hash[..8]);
        
        if !validator::validate_block(&block, last_block) {
            eprintln!("Validation Error: Block #{} failed validation", block.header.block_number);
            return false;
        }
        
        // Persist block to storage and update chain
        self.storage.write_block(&block);
        self.blocks.push(block);
        println!("✅ Block #{} successfully validated and added to the chain.", self.blocks.last().unwrap().header.block_number);
        true
    }

    /// Persists the current blockchain state to disk
    pub fn save_state(&self) {
        self.storage.write_state(&self.state);
    }

    /// Simple chain replacement for synchronization - minimal validation
    pub fn replace_chain_simple(&mut self, new_blocks: Vec<Block>) -> bool {
        // Only replace if new chain is longer
        if new_blocks.is_empty() || new_blocks.len() <= self.blocks.len() {
            return false;
        }

        println!("🔄 Replacing chain: {} blocks -> {} blocks", 
                 self.blocks.len(), new_blocks.len());
        
        // Replace the entire chain
        self.blocks = new_blocks;
        if let Some(last_block) = self.blocks.last() {
            self.storage.write_block(last_block);
        }
        self.save_state();
        true
    }
}