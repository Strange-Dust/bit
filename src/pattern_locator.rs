use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternFormat {
    Hex,    // 0x prefix required
    Ascii,  // Plain ASCII text
    Bits,   // Raw bit sequence (0s and 1s)
}

impl PatternFormat {
    pub fn name(&self) -> &str {
        match self {
            PatternFormat::Hex => "Hex (0x...)",
            PatternFormat::Ascii => "ASCII",
            PatternFormat::Bits => "Bits (0/1)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub format: PatternFormat,
    pub input: String,
    pub garbles: usize,
    #[serde(skip)]
    pub bits: BitVec<u8, Msb0>,
    #[serde(skip)]
    pub matches: Vec<PatternMatch>,
}

impl Pattern {
    pub fn new(name: String, format: PatternFormat, input: String, garbles: usize) -> Result<Self, String> {
        let bits = Self::parse_input(&input, format)?;
        
        Ok(Pattern {
            name,
            format,
            input,
            garbles,
            bits,
            matches: Vec::new(),
        })
    }
    
    /// Parse input string into a bit sequence based on format
    fn parse_input(input: &str, format: PatternFormat) -> Result<BitVec<u8, Msb0>, String> {
        match format {
            PatternFormat::Hex => Self::parse_hex(input),
            PatternFormat::Ascii => Self::parse_ascii(input),
            PatternFormat::Bits => Self::parse_bits(input),
        }
    }
    
    /// Parse hex string (must start with 0x)
    fn parse_hex(input: &str) -> Result<BitVec<u8, Msb0>, String> {
        let input = input.trim();
        
        if !input.starts_with("0x") && !input.starts_with("0X") {
            return Err("Hex pattern must start with 0x".to_string());
        }
        
        let hex_str = &input[2..];
        if hex_str.is_empty() {
            return Err("Hex pattern is empty".to_string());
        }
        
        let mut bits = BitVec::<u8, Msb0>::new();
        
        for c in hex_str.chars() {
            let nibble = c.to_digit(16)
                .ok_or_else(|| format!("Invalid hex character: {}", c))? as u8;
            
            // Add 4 bits for each hex digit
            bits.push((nibble & 0b1000) != 0);
            bits.push((nibble & 0b0100) != 0);
            bits.push((nibble & 0b0010) != 0);
            bits.push((nibble & 0b0001) != 0);
        }
        
        Ok(bits)
    }
    
    /// Parse ASCII string (each character becomes 8 bits)
    fn parse_ascii(input: &str) -> Result<BitVec<u8, Msb0>, String> {
        if input.is_empty() {
            return Err("ASCII pattern is empty".to_string());
        }
        
        let mut bits = BitVec::<u8, Msb0>::new();
        
        for byte in input.as_bytes() {
            // Add 8 bits for each ASCII character
            for i in (0..8).rev() {
                bits.push((byte & (1 << i)) != 0);
            }
        }
        
        Ok(bits)
    }
    
    /// Parse raw bit string (only 0s and 1s allowed)
    fn parse_bits(input: &str) -> Result<BitVec<u8, Msb0>, String> {
        let input = input.trim();
        
        if input.is_empty() {
            return Err("Bit pattern is empty".to_string());
        }
        
        let mut bits = BitVec::<u8, Msb0>::new();
        
        for c in input.chars() {
            match c {
                '0' => bits.push(false),
                '1' => bits.push(true),
                ' ' | '_' => {}, // Allow spaces and underscores for readability
                _ => return Err(format!("Invalid bit character: {}. Use only 0 and 1", c)),
            }
        }
        
        if bits.is_empty() {
            return Err("Bit pattern is empty".to_string());
        }
        
        Ok(bits)
    }
    
    /// Update the pattern bits after input changes
    pub fn update_bits(&mut self) -> Result<(), String> {
        self.bits = Self::parse_input(&self.input, self.format)?;
        Ok(())
    }
    
    /// Search for this pattern in the given bit sequence with garble tolerance
    pub fn search(&mut self, haystack: &BitVec<u8, Msb0>) {
        self.matches.clear();
        
        if self.bits.is_empty() || haystack.is_empty() {
            return;
        }
        
        let pattern_len = self.bits.len();
        if pattern_len > haystack.len() {
            return;
        }
        
        let mut last_position: Option<usize> = None;
        
        // Slide the pattern window across the haystack
        for start in 0..=(haystack.len() - pattern_len) {
            let window = &haystack[start..start + pattern_len];
            
            // Count mismatches (Hamming distance)
            let mismatches = self.bits.iter()
                .zip(window.iter())
                .filter(|(a, b)| a != b)
                .count();
            
            // Check if within garble tolerance
            if mismatches <= self.garbles {
                let delta = last_position.map(|last| start - last);
                last_position = Some(start);
                
                // Store the actual matched bits
                let actual_bits = window.to_bitvec();
                
                self.matches.push(PatternMatch {
                    position: start,
                    actual_bits,
                    delta,
                    mismatches,
                });
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub position: usize,        // Bit position where pattern was found
    #[serde(skip)]
    pub actual_bits: BitVec<u8, Msb0>,  // The actual bits that matched
    pub delta: Option<usize>,   // Difference from previous match
    pub mismatches: usize,      // Number of bit differences (garbles used)
}

impl PatternMatch {
    /// Get a formatted string of the matched bits
    pub fn bits_string(&self) -> String {
        self.actual_bits.iter()
            .map(|b| if *b { '1' } else { '0' })
            .collect()
    }
}
