# Erbium (ERB) - The Digital Gold 2.0

[![Rust CI](https://github.com/SIGINT-erbium/erbium/actions/workflows/rust.yml/badge.svg)](https://github.com/SIGINT-erbium/erbium/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Erbium is a next-generation Layer 1 blockchain designed to be the definitive "Digital Gold 2.0". It combines a secure, deflationary monetary policy with a high-performance, EVM-compatible smart contract platform.

This repository contains the official Rust implementation of the Erbium node (v1.0).

- **Vision:** To provide a decentralized, scalable, and future-proof platform for both a store of value and decentralized applications.
- **Consensus:** Nominated Proof of Stake (NPoS).
- **Virtual Machine:** EVM-compatible.
- **Project Wiki:** (Coming Soon)

## Project Structure

This project is a Cargo workspace composed of several crates:

- `core`: Defines the core data structures like `Block` and `Transaction`.
- `vm`: Handles the EVM integration and smart contract execution logic.
- `consensus`: Implements the NPoS consensus rules.
- `p2p`: Manages all peer-to-peer networking logic using `libp2p`.
- `node`: The main node binary that ties all the other crates together.

## Quick Start (Development)

1.  **Install Rust:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2.  **Clone the repository:** `git clone https://github.com/SIGINT-erbium/erbium.git`
3.  **Build the node:** `cd erbium && cargo build`
4.  **Run the node:** `cargo run -p erbium-node`

## Documentation

- **API Documentation:** Generated from source code comments. Run `cargo doc --open` to view.
- **Project Encyclopedia:** For architectural details, design decisions, and guides, please visit our [GitHub Wiki](https://github.com/SIGINT-erbium/erbium/wiki).
