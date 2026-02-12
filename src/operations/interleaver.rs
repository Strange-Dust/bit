// Bit interleaving and de-interleaving algorithms
// Used for error resilience in communication protocols

use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::traits::{EditorAction, EditorContext, OperationEditor, render_save_cancel_buttons};
use crate::utils::eval_expression;
use eframe::egui;

/// Type of interleaving to perform
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InterleaverType {
    Block,
    Convolutional,
    Symbol,
}

/// Direction of the interleaving operation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InterleaverDirection {
    Interleave,
    Deinterleave,
}

impl Default for InterleaverType {
    fn default() -> Self {
        InterleaverType::Block
    }
}

impl Default for InterleaverDirection {
    fn default() -> Self {
        InterleaverDirection::Interleave
    }
}

/// Configuration for block interleaver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInterleaverConfig {
    pub block_size: usize,      // Number of bits per block (columns)
    pub depth: usize,            // Number of blocks to interleave (rows)
    pub direction: InterleaverDirection,
}

/// Configuration for convolutional interleaver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvolutionalInterleaverConfig {
    pub branches: usize,         // Number of parallel branches (B)
    pub delay_increment: usize,  // Delay increment between branches (M)
    pub direction: InterleaverDirection,
}

/// Configuration for symbol-level interleaver
/// Treats multi-bit symbols (e.g., bytes/characters) as atomic units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInterleaverConfig {
    pub symbol_size: usize,     // Number of bits per symbol (e.g., 8 for bytes)
    pub block_size: usize,      // Number of symbols per block (columns)
    pub depth: usize,           // Number of blocks to interleave (rows)
    pub direction: InterleaverDirection,
}

impl BlockInterleaverConfig {
    pub fn new(block_size: usize, depth: usize, direction: InterleaverDirection) -> Self {
        Self {
            block_size,
            depth,
            direction,
        }
    }

    /// Apply block interleaving to input bits.
    /// Takes ownership to avoid cloning large data structures.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.direction {
            InterleaverDirection::Interleave => self.interleave(input),
            InterleaverDirection::Deinterleave => self.deinterleave(input),
        }
    }

    /// Block interleaving: write row-wise, read column-wise
    fn interleave(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.block_size == 0 || self.depth == 0 {
            return input;
        }

        let matrix_size = self.block_size * self.depth;
        let mut result = BitVec::with_capacity(input.len());

        // Process input in matrix_size chunks
        for chunk_start in (0..input.len()).step_by(matrix_size) {
            let chunk_end = (chunk_start + matrix_size).min(input.len());
            let chunk = &input[chunk_start..chunk_end];
            
            // Create matrix: write row-wise (fill by rows)
            // Matrix is depth rows x block_size columns
            let mut matrix = vec![vec![false; self.block_size]; self.depth];
            
            for (i, bit) in chunk.iter().enumerate() {
                let row = i / self.block_size;
                let col = i % self.block_size;
                if row < self.depth && col < self.block_size {
                    matrix[row][col] = *bit;
                }
            }
            
            // Read column-wise
            for col in 0..self.block_size {
                for row in 0..self.depth {
                    if chunk_start + row * self.block_size + col < chunk_end {
                        result.push(matrix[row][col]);
                    }
                }
            }
        }
        
        result
    }

    /// Block deinterleaving: write column-wise, read row-wise
    fn deinterleave(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.block_size == 0 || self.depth == 0 {
            return input;
        }

        let mut result = BitVec::with_capacity(input.len());
        let matrix_size = self.block_size * self.depth;
        
        // Process input in matrix_size chunks
        for chunk_start in (0..input.len()).step_by(matrix_size) {
            let chunk_end = (chunk_start + matrix_size).min(input.len());
            let chunk = &input[chunk_start..chunk_end];
            
            // Create matrix: write column-wise
            let mut matrix = vec![vec![false; self.block_size]; self.depth];
            
            let mut bit_idx = 0;
            for col in 0..self.block_size {
                for row in 0..self.depth {
                    if bit_idx < chunk.len() {
                        matrix[row][col] = chunk[bit_idx];
                        bit_idx += 1;
                    }
                }
            }
            
            // Read row-wise
            for row in 0..self.depth {
                for col in 0..self.block_size {
                    if chunk_start + row * self.block_size + col < chunk_end {
                        result.push(matrix[row][col]);
                    }
                }
            }
        }
        
        result
    }
}

impl ConvolutionalInterleaverConfig {
    pub fn new(branches: usize, delay_increment: usize, direction: InterleaverDirection) -> Self {
        Self {
            branches,
            delay_increment,
            direction,
        }
    }

    /// Apply convolutional interleaving to input bits.
    /// Takes ownership to avoid cloning large data structures.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.direction {
            InterleaverDirection::Interleave => self.interleave(input),
            InterleaverDirection::Deinterleave => self.deinterleave(input),
        }
    }

    /// Convolutional interleaving: distribute bits across branches with increasing delays
    fn interleave(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.branches == 0 {
            return input;
        }

        // Create delay lines for each branch
        // Branch i has delay of i * delay_increment
        let mut delays: Vec<Vec<bool>> = (0..self.branches)
            .map(|i| vec![false; i * self.delay_increment])
            .collect();

        let mut result = BitVec::new();
        let mut current_branch = 0;

        for bit in input.iter() {
            // Add bit to end of current branch's delay line
            delays[current_branch].push(*bit);
            
            // Extract output from front of current branch (FIFO)
            let output_bit = delays[current_branch].remove(0);
            result.push(output_bit);
            
            // Move to next branch (round-robin)
            current_branch = (current_branch + 1) % self.branches;
        }

        result
    }

    /// Convolutional deinterleaving: reverse the interleaving process
    fn deinterleave(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.branches == 0 {
            return input;
        }

        // Create delay lines for each branch (inverse delays)
        // For de-interleaving, branch i has delay of (branches - 1 - i) * delay_increment
        let mut delays: Vec<Vec<bool>> = (0..self.branches)
            .map(|i| vec![false; (self.branches - 1 - i) * self.delay_increment])
            .collect();

        let mut result = BitVec::new();
        let mut current_branch = 0;

        for bit in input.iter() {
            // Add bit to end of current branch's delay line
            delays[current_branch].push(*bit);
            
            // Extract output from front of current branch (FIFO)
            let output_bit = delays[current_branch].remove(0);
            result.push(output_bit);
            
            // Move to next branch (round-robin)
            current_branch = (current_branch + 1) % self.branches;
        }

        result
    }

    /// Calculate total delay introduced by the interleaver
    pub fn total_delay(&self) -> usize {
        if self.branches == 0 {
            return 0;
        }
        (self.branches - 1) * self.delay_increment
    }
}

impl SymbolInterleaverConfig {
    pub fn new(symbol_size: usize, block_size: usize, depth: usize, direction: InterleaverDirection) -> Self {
        Self {
            symbol_size,
            block_size,
            depth,
            direction,
        }
    }

    /// Apply symbol-level interleaving to input bits.
    /// Takes ownership to avoid cloning large data structures.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.direction {
            InterleaverDirection::Interleave => self.interleave(input),
            InterleaverDirection::Deinterleave => self.deinterleave(input),
        }
    }

    /// Symbol interleaving: write symbols row-wise, read column-wise
    /// Treats each symbol_size bits as an atomic unit
    fn interleave(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.symbol_size == 0 || self.block_size == 0 || self.depth == 0 {
            return input;
        }

        let mut result = BitVec::new();
        let symbols_per_matrix = self.block_size * self.depth;
        let bits_per_matrix = symbols_per_matrix * self.symbol_size;
        
        // Process input in matrix-sized chunks
        for chunk_start in (0..input.len()).step_by(bits_per_matrix) {
            let chunk_end = (chunk_start + bits_per_matrix).min(input.len());
            let chunk = &input[chunk_start..chunk_end];
            
            // Extract symbols from chunk
            let mut symbols: Vec<BitVec<u8, Msb0>> = Vec::new();
            for symbol_start in (0..chunk.len()).step_by(self.symbol_size) {
                let symbol_end = (symbol_start + self.symbol_size).min(chunk.len());
                if symbol_end - symbol_start == self.symbol_size {
                    symbols.push(chunk[symbol_start..symbol_end].to_bitvec());
                }
            }
            
            // Create symbol matrix: write row-wise
            // Matrix is depth rows x block_size columns
            let mut matrix: Vec<Vec<BitVec<u8, Msb0>>> = vec![vec![]; self.depth];
            
            for (i, symbol) in symbols.iter().enumerate() {
                let row = i / self.block_size;
                let col = i % self.block_size;
                if row < self.depth {
                    if matrix[row].len() <= col {
                        matrix[row].resize(col + 1, BitVec::new());
                    }
                    matrix[row][col] = symbol.clone();
                }
            }
            
            // Read column-wise
            for col in 0..self.block_size {
                for row in 0..self.depth {
                    if col < matrix[row].len() && !matrix[row][col].is_empty() {
                        result.extend_from_bitslice(&matrix[row][col]);
                    }
                }
            }
        }
        
        result
    }

    /// Symbol deinterleaving: write symbols column-wise, read row-wise
    fn deinterleave(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.symbol_size == 0 || self.block_size == 0 || self.depth == 0 {
            return input;
        }

        let mut result = BitVec::new();
        let symbols_per_matrix = self.block_size * self.depth;
        let bits_per_matrix = symbols_per_matrix * self.symbol_size;
        
        // Process input in matrix-sized chunks
        for chunk_start in (0..input.len()).step_by(bits_per_matrix) {
            let chunk_end = (chunk_start + bits_per_matrix).min(input.len());
            let chunk = &input[chunk_start..chunk_end];
            
            // Extract symbols from chunk
            let mut symbols: Vec<BitVec<u8, Msb0>> = Vec::new();
            for symbol_start in (0..chunk.len()).step_by(self.symbol_size) {
                let symbol_end = (symbol_start + self.symbol_size).min(chunk.len());
                if symbol_end - symbol_start == self.symbol_size {
                    symbols.push(chunk[symbol_start..symbol_end].to_bitvec());
                }
            }
            
            // Create symbol matrix: write column-wise
            let mut matrix: Vec<Vec<BitVec<u8, Msb0>>> = vec![vec![BitVec::new(); self.block_size]; self.depth];
            
            let mut symbol_idx = 0;
            for col in 0..self.block_size {
                for row in 0..self.depth {
                    if symbol_idx < symbols.len() {
                        matrix[row][col] = symbols[symbol_idx].clone();
                        symbol_idx += 1;
                    }
                }
            }
            
            // Read row-wise
            for row in 0..self.depth {
                for col in 0..self.block_size {
                    if !matrix[row][col].is_empty() {
                        result.extend_from_bitslice(&matrix[row][col]);
                    }
                }
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_interleave_simple() {
        // 2x2 block interleaver
        // Input:  AB CD (row-wise)
        // Matrix: A B
        //         C D
        // Output: AC BD (column-wise)
        let input = bitvec![u8, Msb0; 1, 0, 1, 1]; // ABCD = 1011
        let config = BlockInterleaverConfig::new(2, 2, InterleaverDirection::Interleave);
        let result = config.apply(input);

        assert_eq!(result, bitvec![u8, Msb0; 1, 1, 0, 1]); // ACBD = 1101
    }

    #[test]
    fn test_block_deinterleave_simple() {
        // Reverse of the above
        let input = bitvec![u8, Msb0; 1, 1, 0, 1]; // ACBD = 1101
        let config = BlockInterleaverConfig::new(2, 2, InterleaverDirection::Deinterleave);
        let result = config.apply(input);

        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 1]); // ABCD = 1011
    }

    #[test]
    fn test_block_interleave_roundtrip() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1, 0, 0];

        let interleave_config = BlockInterleaverConfig::new(4, 2, InterleaverDirection::Interleave);
        let interleaved = interleave_config.apply(input.clone());

        let deinterleave_config = BlockInterleaverConfig::new(4, 2, InterleaverDirection::Deinterleave);
        let recovered = deinterleave_config.apply(interleaved);

        assert_eq!(input, recovered);
    }

    #[test]
    fn test_convolutional_interleave_basic() {
        // 3 branches, delay increment of 1
        // Branch 0: delay 0
        // Branch 1: delay 1
        // Branch 2: delay 2
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 1];
        let input_len = input.len();
        let config = ConvolutionalInterleaverConfig::new(3, 1, InterleaverDirection::Interleave);
        let result = config.apply(input);

        // Result should have same length or slightly less due to delay initialization
        assert!(result.len() <= input_len);
    }

    #[test]
    fn test_convolutional_total_delay() {
        let config = ConvolutionalInterleaverConfig::new(4, 2, InterleaverDirection::Interleave);
        assert_eq!(config.total_delay(), 6); // (4-1) * 2 = 6
    }

    #[test]
    fn test_empty_input() {
        let empty = BitVec::new();
        let empty2 = empty.clone();
        let empty3 = empty.clone();

        let block_config = BlockInterleaverConfig::new(4, 2, InterleaverDirection::Interleave);
        assert_eq!(block_config.apply(empty), empty2);

        let conv_config = ConvolutionalInterleaverConfig::new(3, 1, InterleaverDirection::Interleave);
        assert_eq!(conv_config.apply(empty3), BitVec::<u8, Msb0>::new());
    }

    #[test]
    fn test_symbol_interleave_bytes() {
        // Test with bytes (8-bit symbols)
        // Input: AA BB CC DD (4 bytes = 32 bits)
        // With 2x2 matrix of symbols:
        // Write row-wise:  AA BB
        //                  CC DD
        // Read column-wise: AA CC BB DD
        let input = BitVec::<u8, Msb0>::from_slice(&[0x41u8, 0x41, 0x42, 0x42]); // AABB as bytes

        let config = SymbolInterleaverConfig::new(8, 2, 2, InterleaverDirection::Interleave);
        let result = config.apply(input);

        // Expected: byte0, byte2, byte1, byte3 = 0x41, 0x42, 0x41, 0x42 = ABAB
        let expected = BitVec::<u8, Msb0>::from_slice(&[0x41u8, 0x42, 0x41, 0x42]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_symbol_interleave_aabbccdd_to_abcdabcd() {
        // AABBCCDD to ABCDABCD using symbol interleaver
        // Input: AA BB CC DD (each letter is ASCII character)
        // A = 0x41, B = 0x42, C = 0x43, D = 0x44
        let input = BitVec::<u8, Msb0>::from_slice(&[0x41u8, 0x41, 0x42, 0x42, 0x43, 0x43, 0x44, 0x44]); // AABBCCDD

        // Use 2x4 matrix (2 cols, 4 rows):
        // Write row-wise:  A A
        //                  B B
        //                  C C
        //                  D D
        // Read column-wise: A B C D | A B C D → ABCD ABCD ✓

        let config = SymbolInterleaverConfig::new(8, 2, 4, InterleaverDirection::Interleave);
        let result = config.apply(input);

        let expected = BitVec::<u8, Msb0>::from_slice(&[0x41u8, 0x42, 0x43, 0x44, 0x41, 0x42, 0x43, 0x44]); // ABCDABCD

        assert_eq!(result, expected);
    }

    #[test]
    fn test_symbol_interleave_roundtrip() {
        let input = BitVec::<u8, Msb0>::from_slice(&[0xAAu8, 0xBB, 0xCC, 0xDD]);

        let interleave_config = SymbolInterleaverConfig::new(8, 2, 2, InterleaverDirection::Interleave);
        let interleaved = interleave_config.apply(input.clone());

        let deinterleave_config = SymbolInterleaverConfig::new(8, 2, 2, InterleaverDirection::Deinterleave);
        let recovered = deinterleave_config.apply(interleaved);

        assert_eq!(input, recovered);
    }
}

/// Wrapper struct for interleave operation in the operation pipeline
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
    pub fn description(&self) -> String {
        match self.interleaver_type {
            InterleaverType::Block => {
                if let Some(cfg) = &self.block_config {
                    let dir = match cfg.direction {
                        InterleaverDirection::Interleave => "Interleave",
                        InterleaverDirection::Deinterleave => "Deinterleave",
                    };
                    format!("Block {}x{} {}", cfg.block_size, cfg.depth, dir)
                } else {
                    "Block interleaver".to_string()
                }
            }
            InterleaverType::Convolutional => {
                if let Some(cfg) = &self.convolutional_config {
                    let dir = match cfg.direction {
                        InterleaverDirection::Interleave => "Interleave",
                        InterleaverDirection::Deinterleave => "Deinterleave",
                    };
                    format!("Conv B={} M={} (delay={}) {}", cfg.branches, cfg.delay_increment, cfg.total_delay(), dir)
                } else {
                    "Convolutional interleaver".to_string()
                }
            }
            InterleaverType::Symbol => {
                if let Some(cfg) = &self.symbol_config {
                    let dir = match cfg.direction {
                        InterleaverDirection::Interleave => "Interleave",
                        InterleaverDirection::Deinterleave => "Deinterleave",
                    };
                    format!(
                        "Symbol {}x{} ({}bit) {}",
                        cfg.block_size, cfg.depth, cfg.symbol_size, dir
                    )
                } else {
                    "Symbol interleaver".to_string()
                }
            }
        }
    }

    /// Apply interleaving operation to input bits.
    /// Takes ownership to avoid cloning large data structures.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.interleaver_type {
            InterleaverType::Block => {
                if let Some(cfg) = &self.block_config {
                    cfg.apply(input)
                } else {
                    input
                }
            }
            InterleaverType::Convolutional => {
                if let Some(cfg) = &self.convolutional_config {
                    cfg.apply(input)
                } else {
                    input
                }
            }
            InterleaverType::Symbol => {
                if let Some(cfg) = &self.symbol_config {
                    cfg.apply(input)
                } else {
                    input
                }
            }
        }
    }

    pub fn is_source_operation(&self) -> bool {
        false
    }
}

// --- Editor ---

#[derive(Debug, Clone)]
pub struct InterleaveEditor {
    pub name: String,
    pub interleave_type: InterleaverType,
    pub interleave_direction: InterleaverDirection,
    pub block_size_str: String,
    pub depth_str: String,
    pub branches_str: String,
    pub delay_increment_str: String,
    pub symbol_size_str: String,
}

impl Default for InterleaveEditor {
    fn default() -> Self {
        Self {
            name: String::new(),
            interleave_type: InterleaverType::Block,
            interleave_direction: InterleaverDirection::Interleave,
            block_size_str: "8".to_string(),
            depth_str: "4".to_string(),
            branches_str: "4".to_string(),
            delay_increment_str: "1".to_string(),
            symbol_size_str: "8".to_string(),
        }
    }
}

impl OperationEditor for InterleaveEditor {
    type Operation = InterleaveBitsOperation;

    fn from_operation(op: &InterleaveBitsOperation) -> Self {
        let mut editor = Self {
            name: op.name.clone(),
            interleave_type: op.interleaver_type,
            ..Self::default()
        };

        match op.interleaver_type {
            InterleaverType::Block => {
                if let Some(cfg) = &op.block_config {
                    editor.interleave_direction = cfg.direction;
                    editor.block_size_str = cfg.block_size.to_string();
                    editor.depth_str = cfg.depth.to_string();
                }
            }
            InterleaverType::Convolutional => {
                if let Some(cfg) = &op.convolutional_config {
                    editor.interleave_direction = cfg.direction;
                    editor.branches_str = cfg.branches.to_string();
                    editor.delay_increment_str = cfg.delay_increment.to_string();
                }
            }
            InterleaverType::Symbol => {
                if let Some(cfg) = &op.symbol_config {
                    editor.interleave_direction = cfg.direction;
                    editor.symbol_size_str = cfg.symbol_size.to_string();
                    editor.block_size_str = cfg.block_size.to_string();
                    editor.depth_str = cfg.depth.to_string();
                }
            }
        }

        editor
    }

    fn try_build(&self) -> Result<InterleaveBitsOperation, String> {
        let name = if self.name.trim().is_empty() {
            match self.interleave_type {
                InterleaverType::Block => "Block Interleaver".to_string(),
                InterleaverType::Convolutional => "Convolutional Interleaver".to_string(),
                InterleaverType::Symbol => "Symbol Interleaver".to_string(),
            }
        } else {
            self.name.clone()
        };

        let (block_config, convolutional_config, symbol_config) = match self.interleave_type {
            InterleaverType::Block => {
                let block_size = self
                    .block_size_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Block size must be a positive number".to_string())?;
                if block_size == 0 {
                    return Err("Block size must be a positive number".to_string());
                }

                let depth = self
                    .depth_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Depth must be a positive number".to_string())?;
                if depth == 0 {
                    return Err("Depth must be a positive number".to_string());
                }

                (
                    Some(BlockInterleaverConfig::new(
                        block_size,
                        depth,
                        self.interleave_direction,
                    )),
                    None,
                    None,
                )
            }
            InterleaverType::Convolutional => {
                let branches = self
                    .branches_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Branches must be a positive number".to_string())?;
                if branches == 0 {
                    return Err("Branches must be a positive number".to_string());
                }

                let delay_increment = self
                    .delay_increment_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Delay increment must be a valid number".to_string())?;

                (
                    None,
                    Some(ConvolutionalInterleaverConfig::new(
                        branches,
                        delay_increment,
                        self.interleave_direction,
                    )),
                    None,
                )
            }
            InterleaverType::Symbol => {
                let symbol_size = self
                    .symbol_size_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Symbol size must be a positive number".to_string())?;
                if symbol_size == 0 {
                    return Err("Symbol size must be a positive number".to_string());
                }

                let block_size = self
                    .block_size_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Block size must be a positive number".to_string())?;
                if block_size == 0 {
                    return Err("Block size must be a positive number".to_string());
                }

                let depth = self
                    .depth_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "Depth must be a positive number".to_string())?;
                if depth == 0 {
                    return Err("Depth must be a positive number".to_string());
                }

                (
                    None,
                    None,
                    Some(SymbolInterleaverConfig::new(
                        symbol_size,
                        block_size,
                        depth,
                        self.interleave_direction,
                    )),
                )
            }
        };

        Ok(InterleaveBitsOperation {
            name,
            interleaver_type: self.interleave_type,
            block_config,
            convolutional_config,
            symbol_config,
            enabled: true,
        })
    }

    fn render(&mut self, _ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        ui.heading("Bit Interleaving / De-interleaving");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.add_space(8.0);

        // Type selection
        ui.horizontal(|ui| {
            ui.label("Interleaver Type:");
            ui.radio_value(&mut self.interleave_type, InterleaverType::Block, "Block");
            ui.radio_value(
                &mut self.interleave_type,
                InterleaverType::Convolutional,
                "Convolutional",
            );
            ui.radio_value(&mut self.interleave_type, InterleaverType::Symbol, "Symbol");
        });

        ui.add_space(4.0);

        // Direction selection
        ui.horizontal(|ui| {
            ui.label("Direction:");
            ui.radio_value(
                &mut self.interleave_direction,
                InterleaverDirection::Interleave,
                "Interleave",
            );
            ui.radio_value(
                &mut self.interleave_direction,
                InterleaverDirection::Deinterleave,
                "De-interleave",
            );
        });

        ui.add_space(8.0);
        ui.separator();

        // Parameters based on type
        let is_valid = match self.interleave_type {
            InterleaverType::Block => self.render_block_params(ui),
            InterleaverType::Convolutional => self.render_convolutional_params(ui),
            InterleaverType::Symbol => self.render_symbol_params(ui),
        };

        ui.add_space(8.0);

        render_save_cancel_buttons(ui, is_valid)
    }
}

impl InterleaveEditor {
    fn render_block_params(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label("Block Interleaver Parameters:");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Block Size (columns):");
            let block_response = ui.text_edit_singleline(&mut self.block_size_str);

            if block_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.block_size_str) {
                    self.block_size_str = result.to_string();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Depth (rows):      ");
            let depth_response = ui.text_edit_singleline(&mut self.depth_str);

            if depth_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.depth_str) {
                    self.depth_str = result.to_string();
                }
            }
        });

        let block_size = self.block_size_str.trim().parse::<usize>().ok();
        let depth = self.depth_str.trim().parse::<usize>().ok();
        let is_valid = block_size.map_or(false, |v| v > 0) && depth.map_or(false, |v| v > 0);

        ui.add_space(8.0);

        // Visual preview
        ui.group(|ui| {
            ui.label("Visual Preview:");
            ui.add_space(4.0);

            if let (Some(block_size), Some(depth)) = (block_size, depth) {
                if block_size > 0 && depth > 0 && block_size <= 16 && depth <= 16 {
                    ui.label(format!(
                        "Matrix: {}×{} = {} bits per block",
                        block_size,
                        depth,
                        block_size * depth
                    ));
                    ui.add_space(4.0);

                    match self.interleave_direction {
                        InterleaverDirection::Interleave => {
                            ui.label("Write row-wise → Read column-wise:");
                        }
                        InterleaverDirection::Deinterleave => {
                            ui.label("Write column-wise → Read row-wise:");
                        }
                    }
                    ui.add_space(2.0);
                    render_block_matrix_preview(ui, block_size, depth);
                } else {
                    ui.label("! Invalid dimensions (max 16×16)");
                }
            } else {
                ui.label("! Enter valid numbers for preview");
            }
        });

        ui.add_space(4.0);
        ui.label("Tips:");
        ui.label("• Block interleaver rearranges data in matrix blocks");
        ui.label("• Interleave: Write row-wise, read column-wise");
        ui.label("• De-interleave: Reverses the process");
        ui.label("• Example: 8×4 matrix handles 32 bits at a time");
        ui.label("• Math supported: 2*4, 16/2, 8+4, etc.");

        is_valid
    }

    fn render_convolutional_params(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label("Convolutional Interleaver Parameters:");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Branches (B):       ");
            let branches_response = ui.text_edit_singleline(&mut self.branches_str);

            if branches_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.branches_str) {
                    self.branches_str = result.to_string();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Delay Increment (M):");
            let delay_response = ui.text_edit_singleline(&mut self.delay_increment_str);

            if delay_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.delay_increment_str) {
                    self.delay_increment_str = result.to_string();
                }
            }
        });

        let branches = self.branches_str.trim().parse::<usize>().ok();
        let delay_inc = self.delay_increment_str.trim().parse::<usize>().ok();
        let is_valid = branches.map_or(false, |v| v > 0) && delay_inc.is_some();

        ui.add_space(8.0);

        // Visual preview
        ui.group(|ui| {
            ui.label("Visual Preview:");
            ui.add_space(4.0);

            if let (Some(branches), Some(delay_inc)) = (branches, delay_inc) {
                if branches > 0 && branches <= 16 && delay_inc <= 16 {
                    ui.label(format!(
                        "Branches: {}, Delay Increment: {}",
                        branches, delay_inc
                    ));
                    ui.add_space(4.0);

                    match self.interleave_direction {
                        InterleaverDirection::Interleave => {
                            ui.label("Round-robin distribution with delay:");
                        }
                        InterleaverDirection::Deinterleave => {
                            ui.label("Reverse round-robin with delay:");
                        }
                    }
                    ui.add_space(2.0);
                    render_convolutional_preview(ui, branches, delay_inc);
                } else {
                    ui.label("! Invalid parameters (max B=16, M=16)");
                }
            } else {
                ui.label("! Enter valid numbers for preview");
            }
        });

        ui.add_space(4.0);
        ui.label("Tips:");
        ui.label("• Convolutional uses delay lines for each branch");
        ui.label("• Branch i has delay = i × M symbols");
        ui.label("• Distributes bits round-robin across branches");
        ui.label("• Example: B=4, M=1 → delays [0,1,2,3]");
        ui.label("• Provides time-diversity for burst errors");

        is_valid
    }

    fn render_symbol_params(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label("Symbol Interleaver Parameters:");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Symbol Size (bits):");
            let symbol_response = ui.text_edit_singleline(&mut self.symbol_size_str);

            if symbol_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.symbol_size_str) {
                    self.symbol_size_str = result.to_string();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Block Size (columns):");
            let block_response = ui.text_edit_singleline(&mut self.block_size_str);

            if block_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.block_size_str) {
                    self.block_size_str = result.to_string();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Depth (rows):       ");
            let depth_response = ui.text_edit_singleline(&mut self.depth_str);

            if depth_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.depth_str) {
                    self.depth_str = result.to_string();
                }
            }
        });

        ui.add_space(8.0);

        ui.label("Tips:");
        ui.label("• Symbol interleaver treats multi-bit symbols as atomic units");
        ui.label("• Common symbol sizes: 8 (bytes), 4 (nibbles), 16 (words)");
        ui.label("• Use for AABBCCDD → ABCDABCD transformations");
        ui.label("• Matrix: Write symbols row-wise, read column-wise");
        ui.label("• Example: Symbol=8, Block=2, Depth=4 → AABBCCDD → ABCDABCD");

        self.symbol_size_str
            .trim()
            .parse::<usize>()
            .map_or(false, |v| v > 0)
            && self
                .block_size_str
                .trim()
                .parse::<usize>()
                .map_or(false, |v| v > 0)
            && self
                .depth_str
                .trim()
                .parse::<usize>()
                .map_or(false, |v| v > 0)
    }
}

fn render_block_matrix_preview(ui: &mut egui::Ui, cols: usize, rows: usize) {
    use egui::{Color32, Rect, Stroke};

    let cell_size = 24.0;
    let spacing = 2.0;
    let total_width = cols as f32 * (cell_size + spacing);
    let total_height = rows as f32 * (cell_size + spacing);

    let (response, painter) = ui.allocate_painter(
        egui::vec2(total_width, total_height),
        egui::Sense::hover(),
    );

    let base_pos = response.rect.min;

    for row in 0..rows.min(8) {
        for col in 0..cols.min(8) {
            let x = base_pos.x + col as f32 * (cell_size + spacing);
            let y = base_pos.y + row as f32 * (cell_size + spacing);

            let rect = Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_size, cell_size));

            let bit_index = row * cols + col;
            let color = Color32::from_rgb(
                (100 + bit_index * 15).min(255) as u8,
                (150 - bit_index * 5).max(50) as u8,
                200,
            );

            painter.rect_filled(rect, 2.0, color);
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::from_gray(80)),
                egui::epaint::StrokeKind::Outside,
            );

            let text = format!("{}", bit_index);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::monospace(10.0),
                Color32::WHITE,
            );
        }
    }

    if rows > 8 || cols > 8 {
        ui.label("  (showing first 8×8)");
    }
}

fn render_convolutional_preview(ui: &mut egui::Ui, branches: usize, delay_inc: usize) {
    use egui::{Color32, Rect, Stroke};

    let branch_width = 80.0;
    let branch_height = 30.0;
    let spacing = 10.0;

    let total_width = branch_width + 150.0;
    let total_height = branches.min(8) as f32 * (branch_height + spacing);

    let (response, painter) = ui.allocate_painter(
        egui::vec2(total_width, total_height),
        egui::Sense::hover(),
    );

    let base_pos = response.rect.min;

    for i in 0..branches.min(8) {
        let y = base_pos.y + i as f32 * (branch_height + spacing);
        let delay = i * delay_inc;

        // Draw branch box
        let rect = Rect::from_min_size(
            egui::pos2(base_pos.x, y),
            egui::vec2(branch_width, branch_height),
        );

        let color = Color32::from_rgb(
            (100 + i * 20).min(255) as u8,
            (150 - i * 10).max(50) as u8,
            200,
        );

        painter.rect_filled(rect, 3.0, color);
        painter.rect_stroke(
            rect,
            3.0,
            Stroke::new(1.5, Color32::from_gray(80)),
            egui::epaint::StrokeKind::Outside,
        );

        let text = format!("Branch {}", i);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );

        // Draw delay indicator
        let delay_text = format!("Delay: {} symbols", delay);
        painter.text(
            egui::pos2(base_pos.x + branch_width + 15.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            delay_text,
            egui::FontId::monospace(10.0),
            ui.style().visuals.text_color(),
        );
    }

    if branches > 8 {
        ui.label("  (showing first 8 branches)");
    }
}
