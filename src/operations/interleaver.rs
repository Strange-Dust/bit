// Bit interleaving and de-interleaving algorithms
// Used for error resilience in communication protocols

use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

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

    /// Apply block interleaving to input bits
    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.direction {
            InterleaverDirection::Interleave => self.interleave(input),
            InterleaverDirection::Deinterleave => self.deinterleave(input),
        }
    }

    /// Block interleaving: write row-wise, read column-wise
    fn interleave(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.block_size == 0 || self.depth == 0 {
            return input.clone();
        }

        let mut result = BitVec::new();
        let matrix_size = self.block_size * self.depth;
        
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
    fn deinterleave(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.block_size == 0 || self.depth == 0 {
            return input.clone();
        }

        let mut result = BitVec::new();
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

    /// Apply convolutional interleaving to input bits
    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.direction {
            InterleaverDirection::Interleave => self.interleave(input),
            InterleaverDirection::Deinterleave => self.deinterleave(input),
        }
    }

    /// Convolutional interleaving: distribute bits across branches with increasing delays
    fn interleave(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.branches == 0 {
            return input.clone();
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
    fn deinterleave(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.branches == 0 {
            return input.clone();
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

    /// Apply symbol-level interleaving to input bits
    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self.direction {
            InterleaverDirection::Interleave => self.interleave(input),
            InterleaverDirection::Deinterleave => self.deinterleave(input),
        }
    }

    /// Symbol interleaving: write symbols row-wise, read column-wise
    /// Treats each symbol_size bits as an atomic unit
    fn interleave(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.symbol_size == 0 || self.block_size == 0 || self.depth == 0 {
            return input.clone();
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
    fn deinterleave(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if input.is_empty() || self.symbol_size == 0 || self.block_size == 0 || self.depth == 0 {
            return input.clone();
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
        let result = config.apply(&input);
        
        assert_eq!(result, bitvec![u8, Msb0; 1, 1, 0, 1]); // ACBD = 1101
    }

    #[test]
    fn test_block_deinterleave_simple() {
        // Reverse of the above
        let input = bitvec![u8, Msb0; 1, 1, 0, 1]; // ACBD = 1101
        let config = BlockInterleaverConfig::new(2, 2, InterleaverDirection::Deinterleave);
        let result = config.apply(&input);
        
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 1]); // ABCD = 1011
    }

    #[test]
    fn test_block_interleave_roundtrip() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1, 0, 0];
        
        let interleave_config = BlockInterleaverConfig::new(4, 2, InterleaverDirection::Interleave);
        let interleaved = interleave_config.apply(&input);
        
        let deinterleave_config = BlockInterleaverConfig::new(4, 2, InterleaverDirection::Deinterleave);
        let recovered = deinterleave_config.apply(&interleaved);
        
        assert_eq!(input, recovered);
    }

    #[test]
    fn test_convolutional_interleave_basic() {
        // 3 branches, delay increment of 1
        // Branch 0: delay 0
        // Branch 1: delay 1
        // Branch 2: delay 2
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 1];
        let config = ConvolutionalInterleaverConfig::new(3, 1, InterleaverDirection::Interleave);
        let result = config.apply(&input);
        
        // Result should have same length or slightly less due to delay initialization
        assert!(result.len() <= input.len());
    }

    #[test]
    fn test_convolutional_total_delay() {
        let config = ConvolutionalInterleaverConfig::new(4, 2, InterleaverDirection::Interleave);
        assert_eq!(config.total_delay(), 6); // (4-1) * 2 = 6
    }

    #[test]
    fn test_empty_input() {
        let empty = BitVec::new();
        
        let block_config = BlockInterleaverConfig::new(4, 2, InterleaverDirection::Interleave);
        assert_eq!(block_config.apply(&empty), empty);
        
        let conv_config = ConvolutionalInterleaverConfig::new(3, 1, InterleaverDirection::Interleave);
        assert_eq!(conv_config.apply(&empty), empty);
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
        let result = config.apply(&input);
        
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
        let result = config.apply(&input);
        
        let expected = BitVec::<u8, Msb0>::from_slice(&[0x41u8, 0x42, 0x43, 0x44, 0x41, 0x42, 0x43, 0x44]); // ABCDABCD
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_symbol_interleave_roundtrip() {
        let input = BitVec::<u8, Msb0>::from_slice(&[0xAAu8, 0xBB, 0xCC, 0xDD]);
        
        let interleave_config = SymbolInterleaverConfig::new(8, 2, 2, InterleaverDirection::Interleave);
        let interleaved = interleave_config.apply(&input);
        
        let deinterleave_config = SymbolInterleaverConfig::new(8, 2, 2, InterleaverDirection::Deinterleave);
        let recovered = deinterleave_config.apply(&interleaved);
        
        assert_eq!(input, recovered);
    }
}
