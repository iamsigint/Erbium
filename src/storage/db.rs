// src/storage/db.rs

use crate::core::block::Block;
use crate::core::state::State;
use rocksdb::{DB, Options};
use std::fmt; // Import the fmt module

const DB_PATH: &str = "./database";
const TIP_KEY: &str = "tip";
const STATE_KEY: &str = "state";

pub struct Storage {
    db: Option<DB>,
}

// This manual implementation of Debug is now syntactically correct.
impl fmt::Debug for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let db_status = if self.db.is_some() { "Some(Connected)" } else { "None" };
        f.debug_struct("Storage")
         .field("db", &db_status)
         .finish()
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

    pub fn write_block(&self, block: &Block) {
        if let Some(db) = &self.db {
            let block_hash = block.calculate_hash();
            let block_json = serde_json::to_string(block).unwrap();
            db.put(block_hash.as_bytes(), block_json.as_bytes()).unwrap();
            db.put(TIP_KEY.as_bytes(), block_hash.as_bytes()).unwrap();
        }
    }

    pub fn read_block(&self, hash: &str) -> Option<Block> {
        if let Some(db) = &self.db {
            if let Ok(Some(bytes)) = db.get(hash.as_bytes()) {
                if let Ok(json) = String::from_utf8(bytes) {
                    return serde_json::from_str(&json).ok();
                }
            }
        }
        None
    }
    
    pub fn get_tip_hash(&self) -> Option<String> {
        if let Some(db) = &self.db {
            if let Ok(Some(bytes)) = db.get(TIP_KEY.as_bytes()) {
                return String::from_utf8(bytes).ok();
            }
        }
        None
    }

    pub fn write_state(&self, state: &State) {
        if let Some(db) = &self.db {
            let state_json = serde_json::to_string(state).unwrap();
            db.put(STATE_KEY.as_bytes(), state_json.as_bytes()).unwrap();
            println!("DEBUG: Chain state has been persisted to disk.");
        }
    }

    pub fn read_state(&self) -> Option<State> {
        if let Some(db) = &self.db {
            if let Ok(Some(bytes)) = db.get(STATE_KEY.as_bytes()) {
                if let Ok(json) = String::from_utf8(bytes) {
                    println!("DEBUG: Found and loaded chain state from disk.");
                    return serde_json::from_str(&json).ok();
                }
            }
        }
        println!("DEBUG: No chain state found on disk. Creating a new one.");
        None
    }
}