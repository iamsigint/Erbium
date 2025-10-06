// src/p2p/service.rs

use crate::core::chain::Blockchain;
use crate::node::runner::Tx;
use crate::p2p::message::P2pMessage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::{broadcast, Mutex};

// This function handles the logic for a single peer connection.
async fn handle_peer(
    socket: TcpStream,
    addr: SocketAddr,
    blockchain: Arc<Mutex<Blockchain>>,
    mut broadcast_rx: broadcast::Receiver<P2pMessage>,
) {
    println!("Handling peer: {}", addr);
    let (reader, mut writer) = socket.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        select! {
            // Branch 1: A message arrived FROM the connected peer.
            result = buf_reader.read_line(&mut line) => {
                match result {
                    Ok(0) => { println!("Peer {} disconnected.", addr); break; }
                    Ok(_) => {
                        let received_message: Result<P2pMessage, _> = serde_json::from_str(&line);

                        // --- THIS IS THE UPDATED LOGIC ---
                        match received_message {
                            Ok(P2pMessage::RequestChain) => {
                                println!("Peer {} requested the blockchain.", addr);
                                let bc = blockchain.lock().await;
                                let response = P2pMessage::RespondChain(bc.blocks.clone());
                                let response_json = serde_json::to_string(&response).unwrap();
                                writer.write_all(response_json.as_bytes()).await.unwrap();
                                writer.write_all(b"\n").await.unwrap();
                            }
                            Ok(P2pMessage::NewBlock(block)) => {
                                println!("Received a new block (#{}) from peer {}.", block.header.block_number, addr);
                                // Try to add the received block to our chain.
                                // The `add_block` function already contains all the validation logic!
                                let mut bc = blockchain.lock().await;
                                bc.add_block(block);
                            }
                            _ => {
                                println!("Received unhandled or invalid message from {}: {}", addr, line.trim());
                            }
                        }
                        line.clear();
                    }
                    Err(e) => { eprintln!("Error reading from socket: {}", e); break; }
                }
            }

            // Branch 2: A message arrived FROM our node's broadcast channel (to be sent TO the peer).
            Ok(message) = broadcast_rx.recv() => {
                println!("Broadcasting message to peer {}", addr);
                let message_json = serde_json::to_string(&message).unwrap();
                writer.write_all(message_json.as_bytes()).await.unwrap();
                writer.write_all(b"\n").await.unwrap();
            }
        }
    }
}

pub struct P2pService {
    blockchain: Arc<Mutex<Blockchain>>,
    broadcast_tx: Tx,
}

impl P2pService {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>, broadcast_tx: Tx) -> Self {
        Self {
            blockchain,
            broadcast_tx,
        }
    }

    pub async fn run(&self, bootstrap_nodes: Vec<String>) {
        let listener_address = "127.0.0.1:8008";

        let listen_task = tokio::spawn(listen_for_peers(
            listener_address.to_string(),
            Arc::clone(&self.blockchain),
            self.broadcast_tx.clone(),
        ));

        let connect_task = tokio::spawn(connect_to_peers(
            bootstrap_nodes,
            Arc::clone(&self.blockchain),
            self.broadcast_tx.clone(),
        ));

        let _ = tokio::try_join!(listen_task, connect_task);
    }
}

async fn listen_for_peers(address: String, blockchain: Arc<Mutex<Blockchain>>, broadcast_tx: Tx) {
    let listener = TcpListener::bind(&address).await.expect("Failed to bind to address");
    println!("P2P service listening on: {}", address);
    loop {
        if let Ok((socket, addr)) = listener.accept().await {
            println!("New peer connected via listener: {}", addr);
            tokio::spawn(handle_peer(
                socket,
                addr,
                Arc::clone(&blockchain),
                broadcast_tx.subscribe(),
            ));
        }
    }
}

async fn connect_to_peers(nodes: Vec<String>, blockchain: Arc<Mutex<Blockchain>>, broadcast_tx: Tx) {
    for node_addr in nodes {
        if let Ok(socket) = TcpStream::connect(&node_addr).await {
            let addr = socket.peer_addr().unwrap();
            println!("Successfully connected to bootstrap node: {}", addr);
            tokio::spawn(handle_peer(
                socket,
                addr,
                Arc::clone(&blockchain),
                broadcast_tx.subscribe(),
            ));
        } else {
            eprintln!("Failed to connect to bootstrap node {}", node_addr);
        }
    }
}