// src/node/runner.rs

use crate::core::chain::Blockchain;
use crate::core::consensus::edfm;
use crate::crypto::keys::KeyPair;
use crate::node::config::Config;
use crate::p2p::message::P2pMessage;
use crate::p2p::service::{connect_to_peers, listen_for_peers, PendingBlocks, PreCommits, PreVotes};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Duration};

pub type Tx = broadcast::Sender<P2pMessage>;

async fn block_producer_loop(blockchain: Arc<Mutex<Blockchain>>, sender: Tx, node_address: String) {
    let mut interval = interval(Duration::from_secs(4));
    loop {
        interval.tick().await;
        let chain = blockchain.lock().await;
        if !chain.state.validators.is_empty() {
            let last_block = chain.blocks.last().unwrap();
            let seed = last_block.calculate_hash();
            if let Some(winner_address) = edfm::select_proposer(&seed, &chain.state.validators) {
                drop(chain); // Release lock before potentially long operation
                if winner_address == node_address {
                    println!("\nIt's our turn! Proposing new block...");
                    let chain = blockchain.lock().await;
                    let last_block = chain.blocks.last().unwrap();
                    let new_block =
                        crate::core::block::Block::new(last_block.header.block_number + 1, last_block.calculate_hash(), vec![]);
                    if sender.send(P2pMessage::ProposeBlock(new_block)).is_err() {
                        eprintln!("Failed to broadcast block proposal");
                    }
                }
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
        Self {
            blockchain: Arc::new(Mutex::new(Blockchain::new())),
            broadcast_tx: broadcast::channel(32).0,
            keypair,
            pending_blocks: Arc::new(Mutex::new(HashMap::new())),
            pre_votes: Arc::new(Mutex::new(HashMap::new())),
            pre_commits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self) {
        println!("--- Initializing Erbium Node ---");
        println!("My Node ID (Address): {}", self.keypair.get_address());
        let config = Config::load();
        
        let producer_task = block_producer_loop(
            Arc::clone(&self.blockchain), self.broadcast_tx.clone(), self.keypair.get_address(),
        );

        let listen_task = listen_for_peers(
            config.listen_address,
            Arc::clone(&self.blockchain), self.broadcast_tx.clone(),
            Arc::clone(&self.pending_blocks), Arc::clone(&self.pre_votes),
            Arc::clone(&self.pre_commits),
        );

        let connect_task = connect_to_peers(
            config.bootstrap_nodes, Arc::clone(&self.blockchain), self.broadcast_tx.clone(),
            Arc::clone(&self.pending_blocks), Arc::clone(&self.pre_votes),
            Arc::clone(&self.pre_commits),
        );
        
        tokio::join!(producer_task, listen_task, connect_task);
    }
}