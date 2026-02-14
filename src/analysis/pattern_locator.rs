use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternFormat {
    Bits,
    Octal,
    Hex,
    Ascii,
}

impl PatternFormat {
    pub fn label(&self) -> &str {
        match self {
            PatternFormat::Bits => "Bin",
            PatternFormat::Octal => "Oct",
            PatternFormat::Hex => "Hex",
            PatternFormat::Ascii => "ASCII",
        }
    }

    pub fn all() -> &'static [PatternFormat] {
        &[
            PatternFormat::Bits,
            PatternFormat::Octal,
            PatternFormat::Hex,
            PatternFormat::Ascii,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: usize,
    pub name: String,
    pub format: PatternFormat,
    pub input: String,
    pub enabled: bool,
    pub search_inverted: bool,
    pub highlight: bool,
    #[serde(skip)]
    pub bits: BitVec<u8, Msb0>,
    #[serde(skip)]
    pub normal_matches: Vec<PatternMatch>,
    #[serde(skip)]
    pub inverted_matches: Vec<PatternMatch>,
    #[serde(skip)]
    pub validation_error: Option<String>,
}

impl Pattern {
    pub fn new_empty(id: usize) -> Self {
        Pattern {
            id,
            name: format!("Pattern {}", id),
            format: PatternFormat::Bits,
            input: String::new(),
            enabled: true,
            search_inverted: false,
            highlight: true,
            bits: BitVec::new(),
            normal_matches: Vec::new(),
            inverted_matches: Vec::new(),
            validation_error: None,
        }
    }

    /// Legacy constructor for backward compatibility with tests.
    /// The garbles parameter is accepted but ignored.
    pub fn new(name: String, format: PatternFormat, input: String, _garbles: usize) -> Result<Self, String> {
        let bits = Self::parse_input(&input, format)?;

        Ok(Pattern {
            id: 0,
            name,
            format,
            input,
            enabled: true,
            search_inverted: false,
            highlight: true,
            bits,
            normal_matches: Vec::new(),
            inverted_matches: Vec::new(),
            validation_error: None,
        })
    }

    /// Validate and parse the input, setting bits or validation_error.
    pub fn validate(&mut self) {
        if self.input.is_empty() {
            self.validation_error = None;
            self.bits.clear();
            return;
        }
        match Self::parse_input(&self.input, self.format) {
            Ok(bits) => {
                self.bits = bits;
                self.validation_error = None;
            }
            Err(e) => {
                self.validation_error = Some(e);
            }
        }
    }

    fn parse_input(input: &str, format: PatternFormat) -> Result<BitVec<u8, Msb0>, String> {
        match format {
            PatternFormat::Hex => Self::parse_hex(input),
            PatternFormat::Octal => Self::parse_octal(input),
            PatternFormat::Ascii => Self::parse_ascii(input),
            PatternFormat::Bits => Self::parse_bits(input),
        }
    }

    fn parse_hex(input: &str) -> Result<BitVec<u8, Msb0>, String> {
        let input = input.trim();

        // Strip 0x prefix if present (no longer required)
        let hex_str = if input.starts_with("0x") || input.starts_with("0X") {
            &input[2..]
        } else {
            input
        };

        if hex_str.is_empty() {
            return Err("Hex pattern is empty".to_string());
        }

        let mut bits = BitVec::<u8, Msb0>::new();

        for c in hex_str.chars() {
            if c == ' ' || c == '_' {
                continue;
            }
            let nibble = c
                .to_digit(16)
                .ok_or_else(|| format!("Invalid hex character: {}", c))? as u8;

            bits.push((nibble & 0b1000) != 0);
            bits.push((nibble & 0b0100) != 0);
            bits.push((nibble & 0b0010) != 0);
            bits.push((nibble & 0b0001) != 0);
        }

        if bits.is_empty() {
            return Err("Hex pattern is empty".to_string());
        }

        Ok(bits)
    }

    fn parse_octal(input: &str) -> Result<BitVec<u8, Msb0>, String> {
        let input = input.trim();

        if input.is_empty() {
            return Err("Octal pattern is empty".to_string());
        }

        let mut bits = BitVec::<u8, Msb0>::new();

        for c in input.chars() {
            if c == ' ' || c == '_' {
                continue;
            }
            let digit = c
                .to_digit(8)
                .ok_or_else(|| format!("Invalid octal character: {}", c))? as u8;

            // Each octal digit = 3 bits
            bits.push((digit & 0b100) != 0);
            bits.push((digit & 0b010) != 0);
            bits.push((digit & 0b001) != 0);
        }

        if bits.is_empty() {
            return Err("Octal pattern is empty".to_string());
        }

        Ok(bits)
    }

    fn parse_ascii(input: &str) -> Result<BitVec<u8, Msb0>, String> {
        if input.is_empty() {
            return Err("ASCII pattern is empty".to_string());
        }

        let mut bits = BitVec::<u8, Msb0>::new();

        for byte in input.as_bytes() {
            for i in (0..8).rev() {
                bits.push((byte & (1 << i)) != 0);
            }
        }

        Ok(bits)
    }

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
                ' ' | '_' => {}
                _ => return Err(format!("Invalid bit character: {}. Use only 0 and 1", c)),
            }
        }

        if bits.is_empty() {
            return Err("Bit pattern is empty".to_string());
        }

        Ok(bits)
    }

    #[allow(dead_code)]
    pub fn update_bits(&mut self) -> Result<(), String> {
        self.bits = Self::parse_input(&self.input, self.format)?;
        Ok(())
    }

    /// Search for this pattern in the given bit sequence.
    /// Populates normal_matches and (if search_inverted) inverted_matches.
    pub fn search(&mut self, haystack: &BitVec<u8, Msb0>) {
        self.normal_matches.clear();
        self.inverted_matches.clear();

        if self.bits.is_empty() || haystack.is_empty() {
            return;
        }

        let pattern_len = self.bits.len();
        if pattern_len > haystack.len() {
            return;
        }

        // Build inverted pattern if needed
        let inverted_bits = if self.search_inverted {
            let mut inv = self.bits.clone();
            for mut bit in inv.iter_mut() {
                let val = *bit;
                bit.set(!val);
            }
            Some(inv)
        } else {
            None
        };

        let mut last_normal_pos: Option<usize> = None;
        let mut last_inverted_pos: Option<usize> = None;

        for start in 0..=(haystack.len() - pattern_len) {
            let window = &haystack[start..start + pattern_len];

            // Check normal match (exact)
            let is_normal_match = self.bits.iter().zip(window.iter()).all(|(a, b)| *a == *b);
            if is_normal_match {
                let delta = last_normal_pos.map(|last| start - last);
                last_normal_pos = Some(start);
                self.normal_matches.push(PatternMatch {
                    position: start,
                    delta,
                });
            }

            // Check inverted match
            if let Some(ref inv) = inverted_bits {
                let is_inv_match = inv.iter().zip(window.iter()).all(|(a, b)| *a == *b);
                if is_inv_match {
                    let delta = last_inverted_pos.map(|last| start - last);
                    last_inverted_pos = Some(start);
                    self.inverted_matches.push(PatternMatch {
                        position: start,
                        delta,
                    });
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub position: usize,
    pub delta: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MergedMatch {
    pub position: usize,
    pub pattern_id: usize,
    pub is_inverted: bool,
    pub delta_next_same_id: Option<usize>,
}

/// Merge all matches from all enabled patterns, sorted by position.
/// Computes delta_next_same_id via reverse pass.
pub fn merge_matches(patterns: &[Pattern]) -> Vec<MergedMatch> {
    let mut merged: Vec<MergedMatch> = Vec::new();

    for pattern in patterns {
        if !pattern.enabled {
            continue;
        }

        for m in &pattern.normal_matches {
            merged.push(MergedMatch {
                position: m.position,
                pattern_id: pattern.id,
                is_inverted: false,
                delta_next_same_id: None,
            });
        }

        for m in &pattern.inverted_matches {
            merged.push(MergedMatch {
                position: m.position,
                pattern_id: pattern.id,
                is_inverted: true,
                delta_next_same_id: None,
            });
        }
    }

    // Sort by position
    merged.sort_by_key(|m| m.position);

    // Reverse pass to compute delta_next_same_id
    let mut last_seen: HashMap<usize, usize> = HashMap::new();
    for m in merged.iter_mut().rev() {
        if let Some(&next_pos) = last_seen.get(&m.pattern_id) {
            m.delta_next_same_id = Some(next_pos - m.position);
        }
        last_seen.insert(m.pattern_id, m.position);
    }

    merged
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchFilter {
    All,
    NormalOnly,
    InvertedOnly,
}

pub struct PatternLocatorState {
    pub show: bool,
    pub patterns: Vec<Pattern>,
    pub next_id: usize,
    pub merged_matches: Vec<MergedMatch>,
    pub has_searched: bool,
    pub match_page: usize,
    pub filter_pattern_id: Option<usize>,
    pub filter_mode: MatchFilter,
    pub selected_match_index: Option<usize>,
}

impl Default for PatternLocatorState {
    fn default() -> Self {
        let patterns: Vec<Pattern> = (1..=4).map(Pattern::new_empty).collect();
        Self {
            show: false,
            patterns,
            next_id: 5,
            merged_matches: Vec::new(),
            has_searched: false,
            match_page: 0,
            filter_pattern_id: None,
            filter_mode: MatchFilter::All,
            selected_match_index: None,
        }
    }
}

impl PatternLocatorState {
    pub fn add_pattern(&mut self) {
        let pattern = Pattern::new_empty(self.next_id);
        self.next_id += 1;
        self.patterns.push(pattern);
    }

    pub fn remove_pattern(&mut self, idx: usize) {
        if idx < self.patterns.len() {
            self.patterns.remove(idx);
        }
    }

    pub fn duplicate_pattern(&mut self, idx: usize) {
        if idx < self.patterns.len() {
            let mut dup = self.patterns[idx].clone();
            dup.id = self.next_id;
            dup.name = format!("{} (copy)", dup.name);
            dup.normal_matches.clear();
            dup.inverted_matches.clear();
            self.next_id += 1;
            self.patterns.insert(idx + 1, dup);
        }
    }

    pub fn validate_all(&mut self) {
        for pattern in &mut self.patterns {
            pattern.validate();
        }
    }

    pub fn search_all(&mut self, haystack: &BitVec<u8, Msb0>) {
        for pattern in &mut self.patterns {
            if pattern.enabled && pattern.validation_error.is_none() && !pattern.bits.is_empty() {
                pattern.search(haystack);
            }
        }
        self.merged_matches = merge_matches(&self.patterns);
        self.has_searched = true;
        self.match_page = 0;
        self.selected_match_index = None;
    }

    pub fn filtered_matches(&self) -> Vec<&MergedMatch> {
        self.merged_matches
            .iter()
            .filter(|m| {
                if let Some(pid) = self.filter_pattern_id {
                    if m.pattern_id != pid {
                        return false;
                    }
                }
                match self.filter_mode {
                    MatchFilter::All => true,
                    MatchFilter::NormalOnly => !m.is_inverted,
                    MatchFilter::InvertedOnly => m.is_inverted,
                }
            })
            .collect()
    }

    pub fn set_all_enabled(&mut self, enabled: bool) {
        for pattern in &mut self.patterns {
            pattern.enabled = enabled;
        }
    }

    pub fn toggle_all_inverted(&mut self) {
        for pattern in &mut self.patterns {
            pattern.search_inverted = !pattern.search_inverted;
        }
    }

    pub fn toggle_all_highlight(&mut self) {
        for pattern in &mut self.patterns {
            pattern.highlight = !pattern.highlight;
        }
    }
}
