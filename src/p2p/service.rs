// src/p2p/service.rs

use crate::core::block::Block;
use crate::core::chain::Blockchain;
use crate::core::consensus::validator;
use crate::node::runner::Tx;
use crate::p2p::message::P2pMessage;
use crate::p2p::ChainSynchronizer;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::{broadcast, Mutex};
use tokio::time::Duration;

pub type PendingBlocks = Arc<Mutex<HashMap<String, Block>>>;
pub type PreVotes = Arc<Mutex<HashMap<String, HashSet<String>>>>;
pub type PreCommits = Arc<Mutex<HashMap<String, HashSet<String>>>>;

async fn handle_peer(
    socket: TcpStream,
    addr: SocketAddr,
    blockchain: Arc<Mutex<Blockchain>>,
    broadcast_tx: Tx,
    mut broadcast_rx: broadcast::Receiver<P2pMessage>,
    pending_blocks: PendingBlocks,
    pre_votes: PreVotes,
    pre_commits: PreCommits,
) {
    println!("[{}] 🔄 Handling new peer connection", addr);
    let (reader, mut writer) = socket.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    // Perform initial handshake with peer
    {
        let bc = blockchain.lock().await;
        let our_block_number = bc.blocks.last().unwrap().header.block_number;
        let status_msg = P2pMessage::Status { block_number: our_block_number };
        if let Ok(json) = serde_json::to_string(&status_msg) {
            let _ = writer.write_all(json.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
        }
        println!("[{}] Handshake sent - our block height: {}", addr, our_block_number);
    }

    loop {
        select! {
            result = buf_reader.read_line(&mut line) => {
                if result.is_err() || result.unwrap_or(0) == 0 { break; }
                
                if let Ok(msg) = serde_json::from_str::<P2pMessage>(line.trim()) {
                    match msg {
                        P2pMessage::Status { block_number } => {
                            println!("[{}] 📊 Received status - peer height: {}", addr, block_number);
                            let bc = blockchain.lock().await;
                            let our_block_number = bc.blocks.last().unwrap().header.block_number;
                            
                            if block_number > our_block_number {
                                println!("[{}] 🔄 Peer is ahead ({} > {}), requesting chain sync", addr, block_number, our_block_number);
                                let request = P2pMessage::RequestChain;
                                if let Ok(json) = serde_json::to_string(&request) {
                                    let _ = writer.write_all(json.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                            } else if block_number < our_block_number && ChainSynchronizer::should_sync_chain(&blockchain).await {
                                println!("[{}] 📤 We are ahead ({} > {}), offering our chain", addr, our_block_number, block_number);
                                let response = P2pMessage::RespondChain(bc.blocks.clone());
                                if let Ok(json) = serde_json::to_string(&response) {
                                    let _ = writer.write_all(json.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                            } else {
                                println!("[{}] ✅ Chains are synchronized at height {}", addr, our_block_number);
                            }
                        }
                        P2pMessage::RequestChain => {
                            println!("[{}] 📤 Received chain request, sending our chain", addr);
                            let bc = blockchain.lock().await;
                            let response = P2pMessage::RespondChain(bc.blocks.clone());
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                            }
                        }
                        P2pMessage::RespondChain(blocks) => {
                            println!("[{}] 🔄 Received chain response with {} blocks", addr, blocks.len());
                            if ChainSynchronizer::sync_chain(Arc::clone(&blockchain), blocks).await {
                                println!("[{}] ✅ Blockchain synchronized successfully", addr);
                            } else {
                                println!("[{}] ⚠️  Chain synchronization not needed or failed", addr);
                            }
                        }
                        P2pMessage::RegisterValidator { address, stake } => {
                            println!("[{}] 👤 Registering validator: {}", addr, address);
                            let mut bc = blockchain.lock().await;
                            if !bc.state.validators.contains_key(&address) {
                                bc.state.register_validator(address.clone(), stake);
                                bc.save_state();
                                
                                // Forward validator registration to other peers
                                match broadcast_tx.send(P2pMessage::RegisterValidator { address: address.clone(), stake }) {
                                    Ok(_) => println!("[{}] 📤 Validator registration forwarded", addr),
                                    Err(e) => eprintln!("[{}] ❌ Failed to forward validator registration: {}", addr, e),
                                }
                            } else {
                                println!("[{}] ✅ Validator already registered: {}", addr, address);
                            }
                        }
                        P2pMessage::ProposeBlock(block) => {
                            let bc = blockchain.lock().await;
                            let last_block = bc.blocks.last().unwrap();
                            let expected_block_number = last_block.header.block_number + 1;
                            
                            println!("[{}] 📦 Received block proposal #{} (expected #{})", 
                                     addr, block.header.block_number, expected_block_number);
                            
                            // Check if the proposed block is the next expected one
                            if block.header.block_number == expected_block_number {
                                if validator::validate_block(&block, last_block) {
                                    let block_hash = block.calculate_hash();
                                    println!("[{}] ✅ Valid block proposal #{} with hash {}", 
                                             addr, block.header.block_number, &block_hash[..8]);
                                    pending_blocks.lock().await.insert(block_hash.clone(), block);
                                    
                                    // Broadcast PreVote for this block
                                    let block_hash_clone = block_hash.clone();
                                    match broadcast_tx.send(P2pMessage::PreVote { block_hash: block_hash_clone }) {
                                        Ok(_) => println!("[{}] ✅ PreVote broadcasted for block {}", addr, &block_hash[..8]),
                                        Err(e) => eprintln!("[{}] ❌ Failed to broadcast PreVote: {}", addr, e),
                                    }
                                } else {
                                    println!("[{}] ❌ Invalid block proposal #{} - validation failed", addr, block.header.block_number);
                                }
                            } else {
                                println!("[{}] ⚠️  Out-of-order block proposal. Expected #{}, got #{}", 
                                         addr, expected_block_number, block.header.block_number);
                            }
                        }

P2pMessage::NewBlock(block) => {
    println!("[{}] 📦 Received new block #{}", addr, block.header.block_number);
    let mut bc = blockchain.lock().await;
    let last_block = bc.blocks.last().unwrap();
    
    // DEBUG: Show what we're comparing for better troubleshooting
    println!("[{}] 🔍 Sync check - Our last: #{} (hash: {}), Received: #{} (prev_hash: {})", 
             addr, 
             last_block.header.block_number, 
             &last_block.calculate_hash()[..8],
             block.header.block_number,
             &block.header.prev_block_hash[..8]);
    
    // Check if this is the next expected block
    if block.header.block_number == last_block.header.block_number + 1 {
        // Clone the block before moving it to add_block
        let block_clone = block.clone();
        if bc.add_block(block_clone) {
            println!("[{}] ✅ New block #{} added to chain", addr, block.header.block_number);
        } else {
            println!("[{}] ❌ Failed to add block #{} to chain", addr, block.header.block_number);
        }
    } else {
        println!("[{}] ⚠️  Block #{} out of order (expected #{})", 
                 addr, block.header.block_number, last_block.header.block_number + 1);
        
        // If we're behind, request full chain sync
        if block.header.block_number > last_block.header.block_number + 1 {
            println!("[{}] 🔄 We are behind, requesting full chain sync", addr);
            let request = P2pMessage::RequestChain;
            if let Ok(json) = serde_json::to_string(&request) {
                let _ = writer.write_all(json.as_bytes()).await;
                let _ = writer.write_all(b"\n").await;
            }
        }
    }
}

                        P2pMessage::PreVote { block_hash } => {
                            let mut votes = pre_votes.lock().await;
                            let entry = votes.entry(block_hash.clone()).or_insert_with(HashSet::new);
                            if entry.insert(addr.to_string()) {
                                let vote_count = entry.len();
                                println!("[{}] ✅ Received NEW PreVote for block {}. Total: {}", addr, &block_hash[..8], vote_count);
                                
                                let total_validators = blockchain.lock().await.state.validators.len();
                                let threshold = (total_validators * 2 / 3) + 1;
                                
                                if vote_count >= threshold {
                                    println!("[{}] 🎯 PreVote threshold reached for block {} ({}/{})", 
                                             addr, &block_hash[..8], vote_count, total_validators);
                                    
                                    // Broadcast PreCommit once threshold is reached
                                    let block_hash_clone = block_hash.clone();
                                    match broadcast_tx.send(P2pMessage::PreCommit { block_hash: block_hash_clone }) {
                                        Ok(_) => println!("[{}] 📤 PreCommit broadcasted for block {}", addr, &block_hash[..8]),
                                        Err(e) => eprintln!("[{}] ❌ Failed to broadcast PreCommit: {}", addr, e),
                                    }
                                }
                            }
                        }
                        P2pMessage::PreCommit { block_hash } => {
                            let mut commits = pre_commits.lock().await;
                            let entry = commits.entry(block_hash.clone()).or_insert_with(HashSet::new);
                            if entry.insert(addr.to_string()) {
                                let commit_count = entry.len();
                                println!("[{}] ✅ Received NEW PreCommit for block {}. Total: {}", addr, &block_hash[..8], commit_count);
                                
                                let total_validators = blockchain.lock().await.state.validators.len();
                                let threshold = (total_validators * 2 / 3) + 1;
                                
                                if commit_count >= threshold {
                                    println!("[{}] 🎉 FINALIZING BLOCK {} (PreCommits: {}/{})", 
                                             addr, &block_hash[..8], commit_count, total_validators);
                                    
                                    let mut bc = blockchain.lock().await;
                                    if let Some(block) = pending_blocks.lock().await.remove(&block_hash) {
                                        if bc.add_block(block) {
                                            bc.save_state();
                                            
                                            // Clear consensus structures for next round
                                            pre_votes.lock().await.clear();
                                            pre_commits.lock().await.clear();
                                            pending_blocks.lock().await.clear();
                                            println!("[{}] 🧹 Consensus structures cleared for next round", addr);
                                        }
                                    } else {
                                        println!("[{}] ⚠️  Block {} not found in pending blocks", addr, &block_hash[..8]);
                                    }
                                }
                            }
                        }
                    }
                }
                line.clear();
            },
            Ok(msg) = broadcast_rx.recv() => {
                // Forward messages received from broadcast channel to this peer
                if let Ok(json) = serde_json::to_string(&msg) {
                    if let Err(e) = writer.write_all(json.as_bytes()).await {
                        eprintln!("[{}] ❌ Failed to write to peer: {}", addr, e);
                        break;
                    }
                    if let Err(e) = writer.write_all(b"\n").await {
                        eprintln!("[{}] ❌ Failed to write newline to peer: {}", addr, e);
                        break;
                    }
                }
            }
        }
    }
    println!("[{}] 🔌 Peer disconnected", addr);
}

pub async fn listen_for_peers(
    address: String,
    blockchain: Arc<Mutex<Blockchain>>,
    broadcast_tx: Tx,
    pending_blocks: PendingBlocks,
    pre_votes: PreVotes,
    pre_commits: PreCommits,
) {
    let listener = TcpListener::bind(&address).await.expect("Failed to bind to address");
    println!("🌐 P2P service listening on: {}", address);
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("🔗 New incoming connection from: {}", addr);
                tokio::spawn(handle_peer(
                    socket, addr, Arc::clone(&blockchain), broadcast_tx.clone(),
                    broadcast_tx.subscribe(), Arc::clone(&pending_blocks),
                    Arc::clone(&pre_votes), Arc::clone(&pre_commits),
                ));
            }
            Err(e) => {
                eprintln!("❌ Error accepting connection: {}", e);
            }
        }
    }
}

pub async fn connect_to_peers(
    nodes: Vec<String>,
    blockchain: Arc<Mutex<Blockchain>>,
    broadcast_tx: Tx,
    pending_blocks: PendingBlocks,
    pre_votes: PreVotes,
    pre_commits: PreCommits,
) {
    if nodes.is_empty() {
        println!("⚠️  No bootstrap nodes configured");
        return;
    }

    println!("🔗 Connecting to bootstrap nodes: {:?}", nodes);
    
    for node_addr in nodes {
        println!("🔄 Attempting to connect to: {}", node_addr);
        match TcpStream::connect(&node_addr).await {
            Ok(socket) => {
                let addr = socket.peer_addr().unwrap();
                println!("✅ Successfully connected to peer: {}", node_addr);
                tokio::spawn(handle_peer(
                    socket, addr, Arc::clone(&blockchain), broadcast_tx.clone(),
                    broadcast_tx.subscribe(), Arc::clone(&pending_blocks),
                    Arc::clone(&pre_votes), Arc::clone(&pre_commits),
                ));
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to peer {}: {}", node_addr, e);
            }
        }
        
        // Small delay between connection attempts
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}