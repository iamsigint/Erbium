// src/p2p/service.rs

use crate::core::block::Block;
use crate::core::chain::Blockchain;
use crate::core::consensus::validator;
use crate::node::runner::Tx;
use crate::p2p::message::P2pMessage;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::{broadcast, Mutex};

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
    println!("[{}] Handling new peer connection.", addr);
    let (reader, mut writer) = socket.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    // Handshake
    {
        let bc = blockchain.lock().await;
        let our_block_number = bc.blocks.last().unwrap().header.block_number;
        let status_msg = P2pMessage::Status { block_number: our_block_number };
        if let Ok(json) = serde_json::to_string(&status_msg) {
            let _ = writer.write_all(json.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
        }
    }

    loop {
        select! {
            result = buf_reader.read_line(&mut line) => {
                if result.is_err() || result.unwrap_or(0) == 0 { break; }
                
                // Trim the line to remove newline characters before parsing
                if let Ok(msg) = serde_json::from_str::<P2pMessage>(line.trim()) {
                    match msg {
                        P2pMessage::Status { block_number } => {
                            let bc = blockchain.lock().await;
                            if block_number > bc.blocks.last().unwrap().header.block_number {
                                let request = P2pMessage::RequestChain;
                                if let Ok(json) = serde_json::to_string(&request) {
                                    let _ = writer.write_all(json.as_bytes()).await;
                                    let _ = writer.write_all(b"\n").await;
                                }
                            }
                        }
                        P2pMessage::RequestChain => {
                            let bc = blockchain.lock().await;
                            let response = P2pMessage::RespondChain(bc.blocks.clone());
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = writer.write_all(json.as_bytes()).await;
                                let _ = writer.write_all(b"\n").await;
                            }
                        }
                        P2pMessage::RespondChain(blocks) => {
                            let mut bc = blockchain.lock().await;
                            if !blocks.is_empty() && blocks.last().unwrap().header.block_number > bc.blocks.last().unwrap().header.block_number {
                                bc.blocks = blocks;
                            }
                        }
                        P2pMessage::RegisterValidator { address, stake } => {
                            let mut bc = blockchain.lock().await;
                            if !bc.state.validators.contains_key(&address) {
                                bc.state.register_validator(address.clone(), stake);
                                if broadcast_tx.send(P2pMessage::RegisterValidator { address, stake }).is_err() {}
                            }
                        }
                        P2pMessage::ProposeBlock(block) => {
                            let bc = blockchain.lock().await;
                            if validator::validate_block(&block, bc.blocks.last().unwrap()) {
                                let block_hash = block.calculate_hash();
                                pending_blocks.lock().await.insert(block_hash.clone(), block);
                                if broadcast_tx.send(P2pMessage::PreVote { block_hash }).is_err() {}
                            }
                        }
                        P2pMessage::PreVote { block_hash } => {
                            let mut votes = pre_votes.lock().await;
                            if let Some(entry) = votes.get_mut(&block_hash) {
                                if entry.insert(addr.to_string()) {
                                    println!("[{}] Received NEW PreVote for {}. Total: {}", addr, &block_hash[..8], entry.len());
                                }
                            } else {
                                let mut new_set = HashSet::new();
                                new_set.insert(addr.to_string());
                                votes.insert(block_hash.clone(), new_set);
                                println!("[{}] Received FIRST PreVote for {}. Total: 1", addr, &block_hash[..8]);
                            }

                            let total_validators = blockchain.lock().await.state.validators.len();
                            let threshold = (total_validators * 2 / 3) + 1;
                            if votes.get(&block_hash).unwrap().len() >= threshold {
                                if broadcast_tx.send(P2pMessage::PreCommit { block_hash }).is_err() {}
                            }
                        }
                        P2pMessage::PreCommit { block_hash } => {
                            let mut commits = pre_commits.lock().await;
                            if let Some(entry) = commits.get_mut(&block_hash) {
                                if entry.insert(addr.to_string()) {
                                    println!("[{}] Received NEW PreCommit for {}. Total: {}", addr, &block_hash[..8], entry.len());
                                }
                            } else {
                                let mut new_set = HashSet::new();
                                new_set.insert(addr.to_string());
                                commits.insert(block_hash.clone(), new_set);
                                println!("[{}] Received FIRST PreCommit for {}. Total: 1", addr, &block_hash[..8]);
                            }

                            let total_validators = blockchain.lock().await.state.validators.len();
                            let threshold = (total_validators * 2 / 3) + 1;
                            if commits.get(&block_hash).unwrap().len() >= threshold {
                                println!("FINALIZING BLOCK {}", &block_hash[..8]);
                                let mut bc = blockchain.lock().await;
                                if let Some(block) = pending_blocks.lock().await.remove(&block_hash) {
                                    bc.add_block(block);
                                }
                            }
                        }
                    }
                }
                line.clear();
            },
            Ok(msg) = broadcast_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = writer.write_all(json.as_bytes()).await;
                    let _ = writer.write_all(b"\n").await;
                }
            }
        }
    }
    println!("[{}] Peer disconnected.", addr);
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
    println!("P2P service listening on: {}", address);
    loop {
        if let Ok((socket, addr)) = listener.accept().await {
            tokio::spawn(handle_peer(
                socket, addr, Arc::clone(&blockchain), broadcast_tx.clone(),
                broadcast_tx.subscribe(), Arc::clone(&pending_blocks),
                Arc::clone(&pre_votes), Arc::clone(&pre_commits),
            ));
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
    for node_addr in nodes {
        if let Ok(socket) = TcpStream::connect(&node_addr).await {
            let addr = socket.peer_addr().unwrap();
            tokio::spawn(handle_peer(
                socket, addr, Arc::clone(&blockchain), broadcast_tx.clone(),
                broadcast_tx.subscribe(), Arc::clone(&pending_blocks),
                Arc::clone(&pre_votes), Arc::clone(&pre_commits),
            ));
        }
    }
}