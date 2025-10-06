// src/core/chain.rs

use crate::core::block::Block;
use crate::core::consensus::validator;
use crate::storage::db::Storage;
use crate::core::state::State; 

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    storage: Storage,
    pub state: State, 
}

impl Blockchain {
    // The constructor now also initializes the State.
    pub fn new() -> Self {
        let storage = Storage::new();

        if let Some(tip_hash) = storage.get_tip_hash() {
            println!("Found existing blockchain. Loading from disk...");
            let last_block = storage.read_block(&tip_hash).expect("Failed to read tip block.");
            
            // TODO: In the future, we need to load the state from disk as well.
            // For now, we'll start with a fresh state on each load.
            let state = State::new(); 
            
            Self {
                blocks: vec![last_block],
                storage,
                state, 
            }
        } else {
            println!("No existing blockchain found. Creating Genesis Block...");
            let genesis_block = Block::create_genesis_block();
            storage.write_block(&genesis_block);
            
            let state = State::new();
            
            Self {
                blocks: vec![genesis_block],
                storage,
                state,
            }
        }
    }

    // The 'add_block' function remains unchanged for now.
    pub fn add_block(&mut self, block: Block) -> bool {
        let last_block = self.blocks.last().expect("Blockchain is empty!");
        
        // The validation logic now uses the received block as a parameter.
        if !validator::validate_block(&block, last_block) {
            println!("Validation failed. Rejecting new block.");
            return false;
        }
        
        // If validation passes, the block is saved and added.
        self.storage.write_block(&block);
        self.blocks.push(block);
        println!("New block successfully validated and added to the chain.");
        true
    }
}