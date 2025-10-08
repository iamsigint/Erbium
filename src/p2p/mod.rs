// src/p2p/mod.rs

pub mod peer;
pub mod message;
pub mod service;
pub mod discovery;
pub mod sync;

pub use sync::ChainSynchronizer;