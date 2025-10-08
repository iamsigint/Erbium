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
│   │   ├── genesis.rs              ✅ NOVO
│   │   └── consensus/
│   │       ├── mod.rs
│   │       ├── edfm.rs              ✅ ATUALIZADO
│   │       ├── validator.rs         ✅ ATUALIZADO
│   │       └── block_time.rs        ✅ NOVO
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
│   │   ├── mod.rs                   ✅ ATUALIZADO
│   │   ├── peer.rs
│   │   ├── message.rs               ✅ ATUALIZADO
│   │   ├── service.rs               ✅ ATUALIZADO
│   │   ├── sync.rs                  ✅ NOVO
│   │   └── discovery.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── rocksdb.rs
│   │   ├── memory.rs
│   │   └── db.rs                    ✅ ATUALIZADO
│   ├── rpc/
│   │   ├── mod.rs
│   │   ├── http.rs
│   │   ├── ws.rs
│   │   └── handlers.rs
│   ├── node/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── runner.rs                ✅ ATUALIZADO
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