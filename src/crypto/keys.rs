// src/crypto/keys.rs

use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::rngs::OsRng;

// Represents a key pair for a node.
#[derive(Debug)]
pub struct KeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl KeyPair {
    // Generates a new random key pair.
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        Self { secret_key, public_key }
    }

    // Derives the Erbium address from the public key.
    // For now, it's just a hex representation of the public key.
    pub fn get_address(&self) -> String {
        // We take the first 20 bytes (40 hex chars) for a shorter address, similar to Ethereum.
        let full_hex = hex::encode(self.public_key.serialize_uncompressed());
        format!("0x{}", &full_hex[full_hex.len()-40..])
    }
}

// We need the `hex` crate for this. Add `hex = "0.4"` to your Cargo.toml dependencies.