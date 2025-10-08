// src/p2p/message.rs

use serde::{Deserialize, Serialize};
use crate::core::block::Block;

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
    NewBlock(Block),
    PreVote {
        block_hash: String,
    },
    PreCommit {
        block_hash: String,
    },
}