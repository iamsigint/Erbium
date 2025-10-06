// src/p2p/message.rs

use serde::{Deserialize, Serialize};
use crate::core::block::Block;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pMessage {
    Ping(String),
    Pong(String),
    RequestChain,
    RespondChain(Vec<Block>),
    NewBlock(Block),
    // A message sent upon first connection to share the node's state.
    Status {
        block_number: u64,
    },
}