// src/storage/db.rs

use rocksdb::{DB, Options};
use crate::core::block::Block;
use std::fmt;

const DB_PATH: &str = "./database";
// A special key to store the hash of the last block in the chain.
const TIP_KEY: &str = "tip";

pub struct Storage {
    db: Option<DB>,
}

impl fmt::Debug for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let db_status = if self.db.is_some() { "Some(Connected)" } else { "None" };
        f.debug_struct("Storage").field("db", &db_status).finish()
    }
}

impl Storage {
    pub fn new() -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        match DB::open(&opts, DB_PATH) {
            Ok(db) => {
                println!("Successfully opened database at {}", DB_PATH);
                Self { db: Some(db) }
            },
            Err(e) => {
                println!("Failed to open database: {}", e);
                Self { db: None }
            }
        }
    }

    // Writes a block and updates the chain tip.
    pub fn write_block(&self, block: &Block) {
        if let Some(db) = &self.db {
            let block_hash = block.calculate_hash();
            let block_json = serde_json::to_string(block).expect("Failed to serialize block.");
            
            // Write the block itself, using its hash as the key.
            db.put(block_hash.as_bytes(), block_json.as_bytes()).expect("Failed to write block.");
            // Also, update the 'tip' key to store the hash of this new last block.
            db.put(TIP_KEY.as_bytes(), block_hash.as_bytes()).expect("Failed to write tip.");
        }
    }

    // Reads a block from the database using its hash as the key.
    pub fn read_block(&self, hash: &str) -> Option<Block> {
        if let Some(db) = &self.db {
            match db.get(hash.as_bytes()) {
                Ok(Some(block_bytes)) => {
                    let block_json = String::from_utf8(block_bytes).unwrap();
                    let block: Block = serde_json::from_str(&block_json).unwrap();
                    Some(block)
                },
                _ => None,
            }
        } else {
            None
        }
    }
    
    // Gets the hash of the last block in the chain.
    pub fn get_tip_hash(&self) -> Option<String> {
        if let Some(db) = &self.db {
            match db.get(TIP_KEY.as_bytes()) {
                Ok(Some(tip_bytes)) => Some(String::from_utf8(tip_bytes).unwrap()),
                _ => None,
            }
        } else {
            None
        }
    }
}