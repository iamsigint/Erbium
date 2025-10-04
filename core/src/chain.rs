// erbium/core/src/chain.rs

use super::block::Block;
use erbium_crypto::{Keypair, Signature, VerifyingKey, Verifier}; // <-- Verifier adicionado
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::convert::TryInto; // <-- Adicionado para conversão de tipos
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

/// Represents the full chain of blocks.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

/// A thread-safe, shared pointer to the blockchain state.
pub type AppState = Arc<Mutex<Blockchain>>;

impl Blockchain {
    /// Creates a new `Blockchain`, initialized with a Genesis Block.
    pub fn new() -> Self {
        let genesis_keypair = Keypair::generate();
        let genesis_block = Block::new(
            0,
            String::from("Genesis Block - The Foundation of Erbium"),
            "0".repeat(64),
            &genesis_keypair,
        );
        Blockchain {
            blocks: vec![genesis_block],
        }
    }

    /// Adds a new, signed block to the blockchain.
    pub fn add_block(&mut self, data: String, validator_keypair: &Keypair) {
        let last_block = self.blocks.last().expect("Blockchain cannot be empty.");
        let new_block = Block::new(
            last_block.index + 1,
            data,
            last_block.hash.clone(),
            validator_keypair,
        );
        self.blocks.push(new_block);
    }

    /// Helper function to recalculate the hash of a given block.
    pub fn calculate_hash_for_block(block: &Block) -> String {
        let data_to_hash = format!("{}{}{}{}{}", block.index, block.timestamp, block.data, block.previous_hash, block.validator);
        let mut hasher = Sha256::new();
        hasher.update(data_to_hash);
        format!("{:x}", hasher.finalize())
    }

    /// Validates the integrity and signatures of the entire blockchain.
    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            // Rule 1: Check if the block's hash is valid.
            if current_block.hash != Blockchain::calculate_hash_for_block(current_block) {
                eprintln!("ERROR: Hash for Block #{} is invalid!", current_block.index);
                return false;
            }

            // Rule 2: Check if the chain link is intact.
            if current_block.previous_hash != previous_block.hash {
                eprintln!("ERROR: Chain link broken at Block #{}!", current_block.index);
                return false;
            }

            // Rule 3: Check if the signature is valid.
            let public_key_bytes = hex::decode(&current_block.validator).expect("Invalid hex for public key.");
            let signature_bytes = hex::decode(&current_block.signature).expect("Invalid hex for signature.");
            
            // --- CORREÇÃO DE TIPO AQUI ---
            let public_key_array: [u8; 32] = public_key_bytes.try_into().expect("Public key has wrong length.");
            let signature_array: [u8; 64] = signature_bytes.try_into().expect("Signature has wrong length.");

            let public_key = VerifyingKey::from_bytes(&public_key_array).expect("Invalid public key.");
            let signature = Signature::from_bytes(&signature_array);

            if public_key.verify(current_block.hash.as_bytes(), &signature).is_err() {
                eprintln!("ERROR: Signature for Block #{} is invalid!", current_block.index);
                return false;
            }
        }
        true
    }
    
    // Funções de salvar e carregar (sem alterações)
    pub fn save_to_disk(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;
        let serialized_chain = serde_json::to_string_pretty(&self.blocks)?;
        file.write_all(serialized_chain.as_bytes())?;
        Ok(())
    }

    pub fn load_from_disk(file_path: &str) -> std::io::Result<Self> {
        let mut file = File::open(file_path)?;
        let mut serialized_chain = String::new();
        file.read_to_string(&mut serialized_chain)?;
        let blocks: Vec<Block> = serde_json::from_str(&serialized_chain)?;
        Ok(Blockchain { blocks })
    }
}