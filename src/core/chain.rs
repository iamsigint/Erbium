// src/core/chain.rs

use crate::core::block::Block;
use crate::core::consensus::validator;
use crate::storage::db::Storage;

#[derive(Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    storage: Storage,
}

impl Blockchain {
    // A função 'new' continua a mesma.
    pub fn new() -> Self {
        let storage = Storage::new();

        if let Some(tip_hash) = storage.get_tip_hash() {
            println!("Found existing blockchain. Loading from disk...");
            let last_block = storage.read_block(&tip_hash).expect("Failed to read tip block.");
            Self {
                blocks: vec![last_block],
                storage,
            }
        } else {
            println!("No existing blockchain found. Creating Genesis Block...");
            let genesis_block = Block::create_genesis_block();
            storage.write_block(&genesis_block);
            Self {
                blocks: vec![genesis_block],
                storage,
            }
        }
    }

    // --- ESTA FUNÇÃO FOI ATUALIZADA ---
    // Agora ela aceita um `Block` completo e retorna `true` ou `false`.
    pub fn add_block(&mut self, block: Block) -> bool {
        let last_block = self.blocks.last().expect("Blockchain is empty!");
        
        // A lógica de validação agora usa o bloco recebido como parâmetro.
        if !validator::validate_block(&block, last_block) {
            println!("Validation failed. Rejecting new block.");
            return false;
        }
        
        // Se a validação passar, o bloco é salvo e adicionado.
        self.storage.write_block(&block);
        self.blocks.push(block);
        println!("New block successfully validated and added to the chain.");
        true
    }
}