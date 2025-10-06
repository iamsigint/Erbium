// src/p2p/message.rs

use serde::{Deserialize, Serialize};
use crate::core::block::Block;

// THE FIX IS HERE: We add `Clone` to the derive list and add the `NewBlock` variant.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pMessage {
    Ping(String),
    Pong(String),
    RequestChain,
    RespondChain(Vec<Block>),
    // A node broadcasts this message when it creates a new block.
    NewBlock(Block),
}