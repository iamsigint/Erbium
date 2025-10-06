// src/node/runner.rs

use crate::core::block::Block;
use crate::core::chain::Blockchain;
use crate::p2p::message::P2pMessage;
use crate::p2p::service::P2pService;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Duration};
// THIS IS A NEW IMPORT to load the configuration file.
use crate::node::config::Config;

// The Sender side of our broadcast channel.
pub type Tx = broadcast::Sender<P2pMessage>;

// The block production loop now takes a Sender to announce new blocks.
async fn block_producer_loop(blockchain: Arc<Mutex<Blockchain>>, sender: Tx) {
    let mut interval = interval(Duration::from_secs(4));
    loop {
        interval.tick().await;
        let mut chain = blockchain.lock().await;
        
        let last_block = chain.blocks.last().unwrap();
        let new_block = Block::new(
            last_block.header.block_number + 1,
            last_block.calculate_hash(),
            vec![],
        );
        
        println!("\nProducing new block #{}...", new_block.header.block_number);
        
        let is_added = chain.add_block(new_block.clone());
        
        if is_added {
            let message = P2pMessage::NewBlock(new_block);
            if let Err(e) = sender.send(message) {
                eprintln!("Failed to broadcast new block: {}", e);
            }
        }
    }
}

pub struct Node {
    blockchain: Arc<Mutex<Blockchain>>,
    broadcast_tx: Tx,
}

impl Node {
    pub fn new() -> Self {
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        let (tx, _) = broadcast::channel(16);
        Self {
            blockchain,
            broadcast_tx: tx,
        }
    }

    // THIS FUNCTION IS THE UPDATED ONE.
    pub async fn run(&self) {
        println!("Node is starting up...");
        
        // Load the network configuration from the .toml file.
        let config = Config::load();
        
        let p2p_service = P2pService::new(
            Arc::clone(&self.blockchain),
            self.broadcast_tx.clone(),
        );
        
        tokio::join!(
            // Pass the list of bootstrap nodes to the P2P service's run method.
            p2p_service.run(config.bootstrap_nodes),
            block_producer_loop(Arc::clone(&self.blockchain), self.broadcast_tx.clone())
        );
    }
}