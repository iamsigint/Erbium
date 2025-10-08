// src/core/consensus/block_time.rs

use std::time::{SystemTime, UNIX_EPOCH};

pub struct BlockTimeConfig {
    pub target_block_time: u64,  // Target time between blocks in seconds
    pub difficulty_adjustment_blocks: u64, // How often to adjust difficulty
}

impl Default for BlockTimeConfig {
    fn default() -> Self {
        Self {
            target_block_time: 15, // 15 seconds per block (more realistic)
            difficulty_adjustment_blocks: 100,
        }
    }
}

pub struct BlockTimeManager {
    config: BlockTimeConfig,
    // Removed unused field: last_block_timestamp: u64,
}

impl BlockTimeManager {
    pub fn new() -> Self {
        Self {
            config: BlockTimeConfig::default(),
            // Removed unused field initialization
        }
    }

    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn should_produce_block(&self, last_block_timestamp: u64, current_validator_count: usize) -> bool {
        let now = Self::current_timestamp();
        let time_since_last_block = now.saturating_sub(last_block_timestamp);
        
        // Base block time adjusted by validator count
        // More validators = slightly faster block times (up to a point)
        let adjusted_block_time = self.config.target_block_time.saturating_sub(
            std::cmp::min(current_validator_count as u64 / 2, 5) // Max 5 seconds faster
        );
        
        time_since_last_block >= adjusted_block_time
    }

    pub fn calculate_next_block_time(&self, last_block_timestamp: u64) -> u64 {
        last_block_timestamp + self.config.target_block_time
    }

    pub fn get_target_block_time(&self) -> u64 {
        self.config.target_block_time
    }
}