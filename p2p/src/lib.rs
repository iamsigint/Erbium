// erbium/p2p/src/lib.rs

//! Manages the peer-to-peer networking layer for the Erbium node
//! using the libp2p framework.

// --- Crates Imports ---
use async_trait::async_trait;
use libp2p::{
    futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt},
    gossipsub, mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter;
use std::io;
use tokio::{select, sync::mpsc};

// --- Internal Crate Imports ---
use erbium_core::block::Block;
use erbium_core::chain::{AppState, Blockchain};

// --- Constants ---
const BLOCKS_TOPIC: &str = "erbium-blocks";

// --- Network Message & Command Definitions ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
}

#[derive(Debug)]
pub enum NetworkEvent {
    NewBlock { block: Block, source: PeerId },
    ChainResponse { blocks: Vec<Block>, source: PeerId },
}

#[derive(Debug)]
pub enum NetworkCommand {
    BroadcastBlock(Block),
    ListPeers,
}

// --- P2P Codec Implementation ---
#[derive(Debug, Clone, Default)]
pub struct ChainCodec;

#[async_trait]
impl request_response::Codec for ChainCodec {
    type Protocol = &'static str;
    type Request = ChainRequest;
    type Response = ChainResponse;

    async fn read_request<T>(&mut self, _: &Self::Protocol, _io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        Ok(ChainRequest)
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.read_to_end(&mut vec).await?;
        Ok(serde_json::from_slice(&vec).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }

    async fn write_request<T>(&mut self, _: &Self::Protocol, _io: &mut T, _: Self::Request) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        Ok(())
    }

    async fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, response: Self::Response) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&serde_json::to_vec(&response)?).await?;
        Ok(())
    }
}

// --- Libp2p Network Behaviour ---
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ErbiumBehaviourEvent")]
pub struct ErbiumBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub req_resp: request_response::Behaviour<ChainCodec>,
}

#[derive(Debug)]
pub enum ErbiumBehaviourEvent {
    Mdns(mdns::Event),
    Gossipsub(gossipsub::Event),
    ReqResp(request_response::Event<ChainRequest, ChainResponse>),
}

impl From<mdns::Event> for ErbiumBehaviourEvent {
    fn from(v: mdns::Event) -> Self {
        Self::Mdns(v)
    }
}
impl From<gossipsub::Event> for ErbiumBehaviourEvent {
    fn from(v: gossipsub::Event) -> Self {
        Self::Gossipsub(v)
    }
}
impl From<request_response::Event<ChainRequest, ChainResponse>> for ErbiumBehaviourEvent {
    fn from(v: request_response::Event<ChainRequest, ChainResponse>) -> Self {
        Self::ReqResp(v)
    }
}

// --- Main P2P Service Task ---
pub async fn start(
    app_state: AppState,
    mut command_receiver: mpsc::UnboundedReceiver<NetworkCommand>,
    event_sender: mpsc::UnboundedSender<NetworkEvent>,
) {
    println!("[p2p] Initializing libp2p Swarm...");

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .expect("Failed to create TCP transport")
        .with_behaviour(|key| {
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub::ConfigBuilder::default().build().unwrap(),
            )
            .unwrap();

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id()).unwrap();
            
            let req_resp = request_response::Behaviour::new(
                iter::once(("/erbium/chain-exchange/1", ProtocolSupport::Full)),
                request_response::Config::default(),
            );

            ErbiumBehaviour {
                gossipsub,
                mdns,
                req_resp,
            }
        })
        .expect("Failed to create network behaviour")
        .build();

    let topic = gossipsub::IdentTopic::new(BLOCKS_TOPIC);
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .unwrap();

    println!("[p2p] Erbium Node Online.");
    println!("[p2p] Node ID: {:?}", swarm.local_peer_id());

    let mut requested_peers = HashSet::new();

    loop {
        select! {
            Some(command) = command_receiver.recv() => {
                match command {
                    NetworkCommand::ListPeers => {
                        println!("[p2p] Connected peers:");
                        let peers = swarm.behaviour().gossipsub.all_peers();
                        for peer in peers {
                            println!(" - {:?}", peer.0);
                        }
                    },
                    NetworkCommand::BroadcastBlock(block) => {
                        let json = serde_json::to_string(&block).unwrap();
                        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json.as_bytes()) {
                            println!("[p2p] Error publishing block: {:?}", e);
                        }
                    },
                }
            },
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("[p2p] Listening on: {:?}", address);
                    }
                    SwarmEvent::Behaviour(ErbiumBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _) in list {
                            if requested_peers.insert(peer_id) {
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                swarm.behaviour_mut().req_resp.send_request(&peer_id, ChainRequest);
                                println!("[p2p] Requesting chain from newly discovered peer: {:?}", peer_id);
                            }
                        }
                    },
                    SwarmEvent::Behaviour(ErbiumBehaviourEvent::ReqResp(request_response::Event::Message {
                        message: request_response::Message::Request { channel, .. },
                        ..
                    })) => {
                        let blockchain = app_state.lock().unwrap();
                        swarm.behaviour_mut().req_resp.send_response(channel, ChainResponse { blocks: blockchain.blocks.clone() }).unwrap();
                    },
                    SwarmEvent::Behaviour(ErbiumBehaviourEvent::ReqResp(request_response::Event::Message {
                        peer,
                        message: request_response::Message::Response { response, .. }
                    })) => {
                        if let Err(e) = event_sender.send(NetworkEvent::ChainResponse { blocks: response.blocks, source: peer }) {
                            println!("[p2p] Error sending chain response to app: {:?}", e);
                        }
                    },
                    SwarmEvent::Behaviour(ErbiumBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                        if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                           if let Some(source) = message.source {
                                if let Err(e) = event_sender.send(NetworkEvent::NewBlock { block, source }) {
                                    println!("[p2p] Error sending new block to app: {:?}", e);
                                }
                           }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}