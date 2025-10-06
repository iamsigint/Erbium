// src/main.rs

// Module declarations
pub mod core;
pub mod crypto;
pub mod evm;
pub mod node;
pub mod p2p;
pub mod rpc;
pub mod smart_contracts;
pub mod storage;
pub mod utils;

use crate::node::runner::Node;

#[tokio::main]
async fn main() {
    println!("--- Initializing Erbium Node ---");
    
    // Create a new node instance. This will handle loading/creating the blockchain.
    let node = Node::new();
    
    // Run the node. This will start the P2P service and run forever.
    node.run().await;
}