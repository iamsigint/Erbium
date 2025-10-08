// src/p2p/message.rs

use crate::core::block::Block;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pMessage {
    Status {
        block_number: u64,
    },
    RequestChain,
    RespondChain(Vec<Block>),
    RegisterValidator {
        address: String,
        stake: u64,
    },
    ProposeBlock(Block),
    PreVote {
        block_hash: String,
    },
    PreCommit {
        block_hash: String,
    },
}