// src/crypto/hash.rs

use serde::Serialize;
use sha2::{Digest, Sha256};

// A generic function to calculate the SHA-256 hash of any data structure
// that can be serialized.
pub fn calculate_hash<T: Serialize>(data: &T) -> String {
    // First, serialize the data structure into a JSON string.
    let data_as_string = serde_json::to_string(data).expect("Failed to serialize data.");
    
    // Create a new SHA-256 hasher.
    let mut hasher = Sha256::new();
    
    // Write the string data as bytes into the hasher.
    hasher.update(data_as_string.as_bytes());
    
    // Finalize the hash computation and get the result.
    let hash_result = hasher.finalize();
    
    // Format the hash result as a hexadecimal string and return it.
    format!("{:x}", hash_result)
}