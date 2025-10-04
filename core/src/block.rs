// erbium/core/src/block.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use erbium_crypto::{Keypair, Signature}; // Import our crypto tools

/// Represents a single, signed block in the blockchain.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    /// The public key of the validator who created this block.
    pub validator: String,
    /// The signature of the block's hash, signed by the validator.
    pub signature: String,
}

impl Block {
    /// Constructs and signs a new Block.
    pub fn new(index: u64, data: String, previous_hash: String, validator_keypair: &Keypair) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before the Unix epoch.")
            .as_secs();
        
        let validator_pub_key = hex::encode(validator_keypair.public.as_bytes());

        // The hash is calculated first, as it's the data we will sign.
        let data_to_hash = format!("{}{}{}{}{}", index, timestamp, data, previous_hash, validator_pub_key);
        let mut hasher = Sha256::new();
        hasher.update(data_to_hash);
        let hash = format!("{:x}", hasher.finalize());

        // Sign the calculated hash with the validator's private key.
        let signature = validator_keypair.sign(hash.as_bytes());

        Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash,
            validator: validator_pub_key,
            signature: hex::encode(signature.to_bytes()),
        }
    }
}