// src/core/transaction.rs

use serde::{Deserialize, Serialize};

// THIS IS THE FIX: Add `Clone` to the derive macro here as well.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub fee: u64,
    pub signature: String,
}