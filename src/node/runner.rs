// src/node/runner.rs

use crate::core::chain::Blockchain;
use crate::core::consensus::edfm;
use crate::crypto::keys::KeyPair;
use crate::node::config::Config;
use crate::p2p::message::P2pMessage;
use crate::p2p::service::{connect_to_peers, listen_for_peers, PendingBlocks, PreCommits, PreVotes};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Duration};

pub type Tx = broadcast::Sender<P2pMessage>;

/// Main block production loop - simplified version without complex consensus
async fn block_producer_loop(
    blockchain: Arc<Mutex<Blockchain>>, 
    sender: Tx, 
    node_address: String,
) {
    let mut interval = interval(Duration::from_secs(10));
    
    println!("⏰ Block producer started - checking every 10 seconds");
    
    // WAIT FOR INITIAL SYNC - Don't produce blocks immediately
    println!("🔄 Waiting 15 seconds for initial network synchronization...");
    tokio::time::sleep(Duration::from_secs(15)).await;
    println!("✅ Starting block production after sync period");
    
    loop {
        interval.tick().await;
        
        let mut chain = blockchain.lock().await;
        
        // Only produce blocks if we have registered validators
        if chain.state.validators.is_empty() {
            println!("⏳ No validators yet, waiting...");
            continue;
        }
        
        // DEBUG: Show current chain status
        println!("📊 Current chain: {} blocks, last block #{}", 
                 chain.blocks.len(), chain.blocks.last().unwrap().header.block_number);
        
        let last_block = chain.blocks.last().unwrap();
        let seed = last_block.calculate_hash();
        
        // Select proposer based on EDFM consensus
        if let Some(winner_address) = edfm::select_proposer(&seed, &chain.state.validators) {
            if winner_address == node_address {
                println!("\n🎯 IT'S OUR TURN! Creating block #{}", last_block.header.block_number + 1);
                
                let new_block = crate::core::block::Block::new(
                    last_block.header.block_number + 1,
                    last_block.calculate_hash(),
                    vec![],
                );
                
                // DEBUG: Show block details for troubleshooting
                println!("📦 New block details - Prev Hash: {}, Number: {}", 
                         &new_block.header.prev_block_hash[..8], new_block.header.block_number);
                
                // Add block to local chain first
                if chain.add_block(new_block.clone()) {
                    println!("✅ Block #{} added to local chain", new_block.header.block_number);
                    
                    // Broadcast simple block message to network
                    let message = P2pMessage::NewBlock(new_block);
                    if let Err(e) = sender.send(message) {
                        eprintln!("❌ Failed to broadcast block: {}", e);
                    } else {
                        println!("📤 Block broadcasted to network");
                    }
                }
            } else {
                println!("⏳ Not our turn - proposer is: {}", winner_address);
            }
        }
    }
}

pub struct Node {
    blockchain: Arc<Mutex<Blockchain>>,
    broadcast_tx: Tx,
    keypair: KeyPair,
    pending_blocks: PendingBlocks,
    pre_votes: PreVotes,
    pre_commits: PreCommits,
}

impl Node {
    pub fn new() -> Self {
        let keypair = KeyPair::new();
        let (broadcast_tx, _) = broadcast::channel(32);
        
        Self {
            blockchain: Arc::new(Mutex::new(Blockchain::new())),
            broadcast_tx,
            keypair,
            pending_blocks: Arc::new(Mutex::new(HashMap::new())),
            pre_votes: Arc::new(Mutex::new(HashMap::new())),
            pre_commits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Main node initialization and execution function
    pub async fn run(&self) {
        println!("--- Initializing Erbium Node ---");
        println!("My Node ID (Address): {}", self.keypair.get_address());
        let config = Config::load();
        
        // Register as validator FIRST before starting P2P
        {
            let mut chain = self.blockchain.lock().await;
            if !chain.state.validators.contains_key(&self.keypair.get_address()) {
                println!("👤 Registering as validator with stake 100000");
                chain.state.register_validator(self.keypair.get_address().clone(), 100000);
                chain.save_state();
            }
        }
        
        // Start P2P services AFTER validator registration
        let listen_task = listen_for_peers(
            config.listen_address.clone(),
            Arc::clone(&self.blockchain), 
            self.broadcast_tx.clone(),
            Arc::clone(&self.pending_blocks), 
            Arc::clone(&self.pre_votes),
            Arc::clone(&self.pre_commits),
        );

        let connect_task = connect_to_peers(
            config.bootstrap_nodes.clone(), 
            Arc::clone(&self.blockchain), 
            self.broadcast_tx.clone(),
            Arc::clone(&self.pending_blocks), 
            Arc::clone(&self.pre_votes),
            Arc::clone(&self.pre_commits),
        );

        // Start simplified block producer
        let producer_task = block_producer_loop(
            Arc::clone(&self.blockchain), 
            self.broadcast_tx.clone(), 
            self.keypair.get_address(),
        );

        println!("🚀 Node started successfully!");
        tokio::join!(producer_task, listen_task, connect_task);
    }
}