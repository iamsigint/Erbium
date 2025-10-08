Erbium-node/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── block.rs
│   │   ├── transaction.rs
│   │   ├── chain.rs
│   │   ├── state.rs
│   │   └── consensus/
│   │       ├── mod.rs
│   │       ├── edfm.rs
│   │       └── validator.rs
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── keys.rs
│   │   ├── hash.rs
│   │   ├── signature.rs
│   │   └── utils.rs
│   ├── evm/
│   │   ├── mod.rs
│   │   ├── gas.rs
│   │   ├── runtime.rs
│   │   └── opcodes.rs
│   ├── smart_contracts/
│   │   ├── mod.rs
│   │   ├── compiler.rs
│   │   ├── executor.rs
│   │   └── std_lib.rs
│   ├── p2p/
│   │   ├── mod.rs
│   │   ├── peer.rs
│   │   ├── message.rs
│   │   ├── sync.rs
│   │   └── discovery.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── rocksdb.rs
│   │   ├── memory.rs
│   │   └── db.rs
│   ├── rpc/
│   │   ├── mod.rs
│   │   ├── http.rs
│   │   ├── ws.rs
│   │   └── handlers.rs
│   ├── node/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── runner.rs
│   │   └── telemetry.rs
│   ├── tests/
│   │   ├── test_consensus.rs
│   │   ├── test_p2p.rs
│   │   └── test_smart_contracts.rs
│   └── utils/
│       ├── mod.rs
│       ├── time.rs
│       ├── logger.rs
│       └── config.rs
├── config/
│   ├── mod.rs
│   ├── network.toml
│   ├── genesis.json
│   └── settings.rs
├── docs/
│   ├── architecture.md
│   ├── consensus-edfm.md
│   ├── evm-architecture.md
│   └── roadmap.md
└── scripts/
    ├── build.sh
    ├── run_node.sh
    └── testnet_setup.sh