// erbium/node/src/main.rs

//! The main entry point for the Erbium node executable.

use std::io;
use std::sync::{Arc, Mutex};
use tokio::{select, sync::mpsc};

// Import from our local crates
use erbium_core::chain::{AppState, Blockchain};
use erbium_crypto::Keypair;
use erbium_p2p::{NetworkCommand, NetworkEvent};

#[tokio::main]
async fn main() {
    println!("Starting Erbium v1.0 Node...");

    // Generate the node's unique cryptographic identity
    let node_keypair = Keypair::generate();
    println!("[App] Node Public Key: {}", hex::encode(node_keypair.public.as_bytes()));

    // Create communication channels
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();

    // Load or create the blockchain
    let blockchain = Blockchain::load_from_disk("erbium_chain.json").unwrap_or_else(|_| {
        let chain = Blockchain::new();
        chain.save_to_disk("erbium_chain.json").expect("Failed to save new blockchain.");
        chain
    });
    let app_state = Arc::new(Mutex::new(blockchain));

    // Spawn the network task in the background
    let network_app_state = app_state.clone();
    tokio::spawn(async move {
        // --- A CORREÇÃO ESTÁ AQUI ---
        erbium_p2p::start(network_app_state, command_receiver, event_sender).await;
    });

    println!("\nAvailable commands: 'create block <data>', 'peers', 'exit'\n");

    // Spawn a separate thread for blocking stdin
    let (stdin_sender, mut stdin_receiver) = mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).expect("Failed to read stdin");
            stdin_sender.send(buffer).unwrap();
        }
    });

    // Main application loop
    loop {
        select! {
            // An event came from the p2p network
            Some(event) = event_receiver.recv() => {
                let mut blockchain = app_state.lock().unwrap();
                match event {
                    NetworkEvent::NewBlock { block, source } => {
                        println!("\n[App] Received Block #{} from {:?}. Validating and adding...", block.index, source);
                        blockchain.blocks.push(block);
                        blockchain.save_to_disk("erbium_chain.json").expect("Failed to save.");
                    },
                    NetworkEvent::ChainResponse { blocks, source } => {
                        println!("\n[App] Received chain response from {:?} with {} blocks.", source, blocks.len());
                        if blocks.len() > blockchain.blocks.len() {
                           if (Blockchain { blocks: blocks.clone() }).is_chain_valid() {
                                println!("[App] Received chain is longer and valid. Replacing local chain.");
                                *blockchain = Blockchain { blocks };
                                blockchain.save_to_disk("erbium_chain.json").expect("Failed to save new chain.");
                            } else {
                                eprintln!("[App] WARNING: Received chain is invalid!");
                            }
                        }
                    }
                }
            },
            // The user typed a command
            Some(input) = stdin_receiver.recv() => {
                let mut parts = input.trim().split_whitespace();
                match parts.next() {
                    Some("create") if parts.next() == Some("block") => {
                        let data = parts.collect::<Vec<&str>>().join(" ");
                        if !data.is_empty() {
                            let mut blockchain = app_state.lock().unwrap();
                            blockchain.add_block(data, &node_keypair);
                            let new_block = blockchain.blocks.last().unwrap().clone();
                            
                            println!("[App] Block #{} created locally. Broadcasting...", new_block.index);
                            command_sender.send(NetworkCommand::BroadcastBlock(new_block)).unwrap();
                        }
                    },
                    Some("peers") => {
                        command_sender.send(NetworkCommand::ListPeers).unwrap();
                    },
                    Some("exit") => {
                        println!("Shutting down node...");
                        break;
                    },
                    _ => {}
                }
            }
        }
    }
}