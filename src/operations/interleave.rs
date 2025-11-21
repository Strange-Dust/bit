use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::interleaver::{
    BlockInterleaverConfig, ConvolutionalInterleaverConfig,
    InterleaverType, SymbolInterleaverConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterleaveBitsOperation {
    pub name: String,
    pub interleaver_type: InterleaverType,
    pub block_config: Option<BlockInterleaverConfig>,
    pub convolutional_config: Option<ConvolutionalInterleaverConfig>,
    pub symbol_config: Option<SymbolInterleaverConfig>,
    pub enabled: bool,
}

impl InterleaveBitsOperation {
    pub fn new_block(name: String, config: BlockInterleaverConfig) -> Self {
        Self {
            name,
            interleaver_type: InterleaverType::Block,
            block_config: Some(config),
            convolutional_config: None,
            symbol_config: None,
            enabled: true,
        }
    }

    pub fn new_convolutional(name: String, config: ConvolutionalInterleaverConfig) -> Self {
        Self {
            name,
            interleaver_type: InterleaverType::Convolutional,
            block_config: None,
            convolutional_config: Some(config),
            symbol_config: None,
            enabled: true,
        }
    }

    pub fn new_symbol(name: String, config: SymbolInterleaverConfig) -> Self {
        Self {
            name,
            interleaver_type: InterleaverType::Symbol,
            block_config: None,
            convolutional_config: None,
            symbol_config: Some(config),
            enabled: true,
        }
    }

    pub fn description(&self) -> String {
        match self.interleaver_type {
            InterleaverType::Block => {
                if let Some(cfg) = &self.block_config {
                    let dir = match cfg.direction {
                        crate::operations::InterleaverDirection::Interleave => "Interleave",
                        crate::operations::InterleaverDirection::Deinterleave => "Deinterleave",
                    };
                    format!("Block {}×{} {}", cfg.block_size, cfg.depth, dir)
                } else {
                    "Block interleaver".to_string()
                }
            }
            InterleaverType::Convolutional => {
                if let Some(cfg) = &self.convolutional_config {
                    let dir = match cfg.direction {
                        crate::operations::InterleaverDirection::Interleave => "Interleave",
                        crate::operations::InterleaverDirection::Deinterleave => "Deinterleave",
                    };
                    format!("Conv B={} M={} {}", cfg.branches, cfg.delay_increment, dir)
                } else {
                    "Convolutional interleaver".to_string()
                }
            }
            InterleaverType::Symbol => {
                if let Some(cfg) = &self.symbol_config {
                    let dir = match cfg.direction {
                        crate::operations::InterleaverDirection::Interleave => "Interleave",
                        crate::operations::InterleaverDirection::Deinterleave => "Deinterleave",
                    };
                    format!(
                        "Symbol {}×{} ({}bit) {}",
                        cfg.block_size, cfg.depth, cfg.symbol_size, dir
                    )
                } else {
                    "Symbol interleaver".to_string()
                }
            }
        }
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.interleaver_type {
            InterleaverType::Block => {
                if let Some(cfg) = &self.block_config {
                    cfg.apply(input)
                } else {
                    input.clone()
                }
            }
            InterleaverType::Convolutional => {
                if let Some(cfg) = &self.convolutional_config {
                    cfg.apply(input)
                } else {
                    input.clone()
                }
            }
            InterleaverType::Symbol => {
                if let Some(cfg) = &self.symbol_config {
                    cfg.apply(input)
                } else {
                    input.clone()
                }
            }
        }
    }
}
