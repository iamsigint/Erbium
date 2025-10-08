// src/p2p/sync.rs

use crate::core::block::Block;
use crate::core::chain::Blockchain;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ChainSynchronizer;

impl ChainSynchronizer {
    pub async fn should_sync_chain(blockchain: &Arc<Mutex<Blockchain>>) -> bool {
        let chain = blockchain.lock().await;
        // If we only have genesis block, we should sync
        chain.blocks.len() <= 1
    }

    pub async fn sync_chain(blockchain: Arc<Mutex<Blockchain>>, peer_blocks: Vec<Block>) -> bool {
        if peer_blocks.is_empty() {
            return false;
        }

        let mut chain = blockchain.lock().await;
        
        // Only sync if peer chain is longer and valid
        if peer_blocks.len() > chain.blocks.len() {
            println!("🔄 Syncing chain: local {} blocks -> peer {} blocks", 
                     chain.blocks.len(), peer_blocks.len());
            
            // Basic validation - check if genesis blocks match
            if chain.blocks[0].calculate_hash() != peer_blocks[0].calculate_hash() {
                eprintln!("❌ Genesis blocks don't match - cannot sync");
                return false;
            }
            
            // Replace the entire chain
            chain.blocks = peer_blocks;
            chain.save_state();
            println!("✅ Chain synchronized to {} blocks", chain.blocks.len());
            true
        } else {
            false
        }
    }

    pub async fn get_chain_for_sync(blockchain: &Arc<Mutex<Blockchain>>) -> Vec<Block> {
        let chain = blockchain.lock().await;
        chain.blocks.clone()
    }
}