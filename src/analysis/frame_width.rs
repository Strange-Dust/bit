// Frame width detection - automatic detection of bit framing patterns

use bitvec::prelude::*;

/// Result of frame width analysis
#[derive(Debug, Clone)]
pub struct FrameWidthAnalysis {
    /// List of (width, score) pairs for all tested widths
    pub width_scores: Vec<(usize, f64)>,
    /// The detected best width
    pub best_width: usize,
    /// Score of the best width
    pub best_score: f64,
    /// Per-width, per-bit-position consistency scores
    /// Outer vec: one entry per tested width
    /// Inner vec: consistency score for each bit position in that width
    pub bit_position_patterns: Vec<Vec<f64>>,
}

/// Score a single width by measuring bit position consistency using Shannon entropy
/// Lower entropy = more structure = better frame width
/// Returns (score, bit_position_entropies) where score is INVERTED (1.0 - avg_entropy)
/// so higher score still means better, for consistency with UI
pub fn score_width(bits: &BitVec<u8, Msb0>, width: usize) -> (f64, Vec<f64>) {
    if width == 0 || bits.len() < width * 2 {
        return (0.0, vec![]);
    }
    
    let num_frames = bits.len() / width;
    if num_frames < 3 {
        // Need at least 3 frames for meaningful analysis
        return (0.0, vec![]);
    }
    
    let mut bit_position_entropies = Vec::with_capacity(width);
    let mut total_entropy = 0.0;
    
    // For each bit position (column) in the frame
    for bit_pos in 0..width {
        let mut count_0 = 0;
        let mut count_1 = 0;
        
        // Sample this bit position across all frames (down the column)
        for frame_idx in 0..num_frames {
            let bit_idx = frame_idx * width + bit_pos;
            if bit_idx < bits.len() {
                if bits[bit_idx] {
                    count_1 += 1;
                } else {
                    count_0 += 1;
                }
            }
        }
        
        // Calculate Shannon entropy for this bit position
        // H = -p(0)*log2(p(0)) - p(1)*log2(p(1))
        // Entropy = 0 when all bits are same (perfect structure)
        // Entropy = 1 when 50/50 split (maximum randomness)
        let total = (count_0 + count_1) as f64;
        let entropy = if total > 0.0 {
            let p0 = count_0 as f64 / total;
            let p1 = count_1 as f64 / total;
            
            let mut h = 0.0;
            if p0 > 0.0 {
                h -= p0 * p0.log2();
            }
            if p1 > 0.0 {
                h -= p1 * p1.log2();
            }
            h
        } else {
            0.0
        };
        
        bit_position_entropies.push(entropy);
        total_entropy += entropy;
    }
    
    // Average entropy across all bit positions
    let avg_entropy = total_entropy / width as f64;
    
    // Calculate how many bit positions have LOW entropy (< 0.3)
    // This indicates structured/consistent bits (like ASCII MSB always being 0)
    let low_entropy_count = bit_position_entropies.iter()
        .filter(|&&e| e < 0.3)
        .count();
    
    // Ratio of low-entropy positions
    let low_entropy_ratio = low_entropy_count as f64 / width as f64;
    
    // Sample size penalty: fewer frames = less reliable statistics
    // Need at least 30 frames for good confidence, scale down if less
    let sample_confidence = if num_frames >= 30 {
        1.0
    } else {
        num_frames as f64 / 30.0
    };
    
    // Width efficiency: prefer smaller widths that capture the structure
    // Penalize unnecessarily large widths by favoring better information density
    // Use a gentle logarithmic penalty for large widths
    let width_penalty = (width as f64 / 8.0).log2().max(0.0) * 0.05; // 5% penalty per doubling above 8
    
    // Combined score:
    // - Base score from inverted average entropy
    // - Bonus for having some low-entropy columns (structured bits)
    // - Penalty for insufficient samples (unreliable statistics)
    // - Small penalty for widths much larger than 8 bits
    let base_score = 1.0 - avg_entropy;
    let structure_bonus = low_entropy_ratio * 0.5; // Up to 50% bonus
    let score = (base_score + structure_bonus) * sample_confidence * (1.0 - width_penalty);
    
    // Store entropies as "consistency scores" for backward compatibility
    // Convert entropy to consistency: 1.0 - entropy
    let consistency_scores: Vec<f64> = bit_position_entropies
        .iter()
        .map(|&e| 1.0 - e)
        .collect();
    
    (score, consistency_scores)
}

/// Score a width using delta-based pattern matching
/// Delta = number of frames to look ahead for repeating patterns
pub fn score_width_with_delta(bits: &BitVec<u8, Msb0>, width: usize, delta: usize) -> (f64, Vec<f64>) {
    if width == 0 || bits.len() < width * (delta + 2) {
        return (0.0, vec![]);
    }
    
    let num_samples = bits.len() / width - delta;
    let mut bit_position_scores = Vec::with_capacity(width);
    let mut total_score = 0.0;
    
    for bit_pos in 0..width {
        let mut pattern_consistency = 0.0;
        let mut valid_samples = 0;
        
        for frame_idx in 0..num_samples {
            let bit_idx = frame_idx * width + bit_pos;
            let delta_idx = (frame_idx + delta) * width + bit_pos;
            
            if delta_idx < bits.len() {
                // Check if bits at same position but delta frames apart match
                if bits[bit_idx] == bits[delta_idx] {
                    pattern_consistency += 1.0;
                }
                valid_samples += 1;
            }
        }
        
        let consistency = if valid_samples > 0 {
            pattern_consistency / valid_samples as f64
        } else {
            0.0
        };
        
        bit_position_scores.push(consistency);
        total_score += consistency;
    }
    
    let avg_score = total_score / width as f64;
    (avg_score, bit_position_scores)
}

/// Find the best frame width in the given range
pub fn find_best_width(
    bits: &BitVec<u8, Msb0>,
    min_width: usize,
    max_width: usize,
    delta: usize,
) -> FrameWidthAnalysis {
    let mut width_scores = Vec::new();
    let mut bit_position_patterns = Vec::new();
    let mut best_width = min_width;
    let mut best_score = 0.0;
    
    for width in min_width..=max_width {
        let (score, bit_patterns) = if delta > 0 {
            score_width_with_delta(bits, width, delta)
        } else {
            score_width(bits, width)
        };
        
        width_scores.push((width, score));
        bit_position_patterns.push(bit_patterns);
        
        // Update best if this score is higher, or if equal, prefer smaller width
        let epsilon = 1e-6; // Consider scores equal if within this threshold
        if score > best_score + epsilon {
            best_score = score;
            best_width = width;
        } else if (score - best_score).abs() < epsilon && width < best_width {
            // Scores are essentially equal - prefer smaller width (fundamental vs harmonic)
            best_width = width;
        }
    }
    
    // Harmonic detection: if the best width is a multiple of another high-scoring width,
    // prefer the fundamental frequency (smaller width)
    // Only apply if best score is reasonably good (> 0.3)
    if best_score > 0.3 {
        for (width, score) in &width_scores {
            // Check if current width is a divisor of best_width (i.e., best_width is a harmonic)
            if *width < best_width && best_width % width == 0 {
                // If the divisor has a score within 30% of the best, it's likely the fundamental
                let score_threshold = best_score * 0.7;
                if *score > score_threshold {
                    best_width = *width;
                    best_score = *score;
                    break; // Take the first (smallest) fundamental found
                }
            }
        }
    }
    
    FrameWidthAnalysis {
        width_scores,
        best_width,
        best_score,
        bit_position_patterns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ascii_detection() {
        // Create ASCII text "AAAA" (0x41414141)
        // In bits: 01000001 01000001 01000001 01000001
        // Bit 7 should be consistently 0
        let mut bits = BitVec::<u8, Msb0>::new();
        for _ in 0..4 {
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,0,1]);
        }
        
        let (score_8, bit_patterns) = score_width(&bits, 8);
        
        // Width 8 should have reasonable score (penalized due to only 4 frames)
        assert!(score_8 > 0.15, "Score for width 8: {}", score_8);
        
        // Bit position 0 (MSB) should be perfectly consistent (all 0s)
        assert!(bit_patterns[0] > 0.99, "Bit 0 consistency: {}", bit_patterns[0]);
    }
    
    #[test]
    fn test_random_data() {
        // Random-looking data should have high entropy (low score)
        // Using more data for better statistical properties
        let bits = bitvec![u8, Msb0; 
            1,0,1,1,0,0,1,0,1,1,1,0,0,1,0,1,
            0,0,1,1,0,1,0,0,1,0,1,0,1,1,0,1,
            1,1,0,0,1,0,1,1,0,1,0,0,1,1,1,0,
            0,1,0,1,1,0,0,1,0,0,1,1,0,1,0,1
        ];
        
        let (score_8, _) = score_width(&bits, 8);
        
        // With 64 bits (8 frames), entropy should be higher, score lower
        // But small sample sizes can still show some structure
        assert!(score_8 < 0.5, "Random data score should be < 0.5: {}", score_8);
    }
    
    #[test]
    fn test_find_best_width() {
        // ASCII pattern - using varied characters so width 8 is distinguishable
        let mut bits = BitVec::<u8, Msb0>::new();
        for _ in 0..10 {
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,0,1]); // 'A'
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,1,0]); // 'B'
        }
        
        let analysis = find_best_width(&bits, 4, 16, 0);
        
        // Should detect width 8 for ASCII
        assert_eq!(analysis.best_width, 8, "Best width should be 8 for ASCII");
    }
    
    #[test]
    fn test_varied_ascii() {
        // Test with varied ASCII text "Hello"
        // H=0x48=01001000, e=0x65=01100101, l=0x6C=01101100, o=0x6F=01101111
        let mut bits = BitVec::<u8, Msb0>::new();
        
        // "Hello" repeated a few times
        let hello_bytes = vec![0x48u8, 0x65, 0x6C, 0x6C, 0x6F];
        for _ in 0..4 {
            for &byte in &hello_bytes {
                for i in (0..8).rev() {
                    bits.push((byte >> i) & 1 != 0);
                }
            }
        }
        
        println!("\nTesting varied ASCII 'Hello' repeated 4 times ({} bits)", bits.len());
        
        let analysis = find_best_width(&bits, 1, 20, 0);
        
        // Print top 5 candidates
        let mut sorted = analysis.width_scores.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        println!("\nTop 5 candidates:");
        for (i, (width, score)) in sorted.iter().take(5).enumerate() {
            println!("  {}. Width {}: {:.6}", i + 1, width, score);
        }
        
        // Width 8 should be in top candidates
        let width_8_score = analysis.width_scores.iter()
            .find(|(w, _)| *w == 8)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        
        println!("\nWidth 8 score: {:.6}", width_8_score);
        println!("Best width detected: {} (score: {:.6})", analysis.best_width, analysis.best_score);
        
        // For ASCII, bit 0 (MSB) should be 0 for all standard ASCII
        // This gives high consistency for that bit position
        assert!(width_8_score > 0.2, "Width 8 should have reasonable score for ASCII");
    }
    
    #[test]
    fn test_large_max_width() {
        // Test that large max widths don't incorrectly win
        let mut bits = BitVec::<u8, Msb0>::new();
        
        // "AAAA" repeated - true width is 8
        for _ in 0..50 {
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,0,1]); // 'A' = 0x41
        }
        
        println!("\n=== Testing Large Max Width (400 bits of 'A', max_width=400) ===");
        
        let analysis = find_best_width(&bits, 1, 400, 0);
        
        // Print top 10 candidates
        let mut sorted = analysis.width_scores.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        println!("\nTop 10 candidates:");
        for (i, (width, score)) in sorted.iter().take(10).enumerate() {
            let num_frames = bits.len() / width;
            println!("  {}. Width {}: {:.6} ({} frames)", i + 1, width, score, num_frames);
        }
        
        println!("\nBest width detected: {} (score: {:.6})", analysis.best_width, analysis.best_score);
        println!("Expected: 8 (true ASCII width)");
        
        // Width 8 should win, not large widths like 400
        assert!(analysis.best_width <= 16, 
            "Best width should be 8 or 16, not {}", analysis.best_width);
    }
    
    #[test]
    fn test_entropy_advantage() {
        // Demonstrate entropy-based detection superiority
        println!("\n=== Entropy-Based Detection Demo ===");
        println!("Using protocol-like data with sync patterns and structure");
        
        let mut bits = BitVec::<u8, Msb0>::new();
        
        // Simulate a simple protocol with 16-bit frames:
        // 2-bit sync (11), 6-bit address, 8-bit data
        for i in 0..50 {
            // Sync bits (always 11)
            bits.push(true);
            bits.push(true);
            
            // Address (varies but structured)
            let addr = (i % 4) as u8;
            for j in (0..6).rev() {
                bits.push((addr >> j) & 1 != 0);
            }
            
            // Data (varies)
            let data = (i * 7) as u8;
            for j in (0..8).rev() {
                bits.push((data >> j) & 1 != 0);
            }
        }
        
        println!("\nGenerated {} bits ({} 16-bit frames)", bits.len(), bits.len() / 16);
        
        // Test various widths
        let test_widths = vec![8, 12, 14, 15, 16, 17, 18, 20, 32];
        println!("\nWidth | Score  | Entropy | Description");
        println!("------|--------|---------|-------------");
        
        for &width in &test_widths {
            let (score, _) = score_width(&bits, width);
            let entropy = 1.0 - score;
            let desc = if width == 16 {
                "CORRECT - sync bits align"
            } else if width % 16 == 0 {
                "Harmonic - also aligns"
            } else {
                "Wrong - scrambles structure"
            };
            
            println!("{:5} | {:.4} | {:.4}   | {}", width, score, entropy, desc);
        }
        
        println!("\n--- Analysis ---");
        println!("Correct width (16) has LOWEST entropy because:");
        println!("- Sync bits always align to same columns → entropy ≈ 0");
        println!("- Structured address bits → low entropy");  
        println!("Wrong widths scramble the structure → higher entropy (closer to 0.5)");
        
        let analysis = find_best_width(&bits, 8, 32, 0);
        println!("\nDetected width: {} (expected: 16)", analysis.best_width);
        
        assert!(analysis.best_width == 16 || analysis.best_width == 32, 
            "Should detect width 16 or its harmonic 32, got {}", analysis.best_width);
    }
    
    #[test]
    fn test_delta_effect() {
        // Test with repeating pattern "AAAA"
        let mut bits = BitVec::<u8, Msb0>::new();
        for _ in 0..10 {
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,0,1]); // 'A' = 0x41
        }
        
        println!("\n=== Testing Delta Effect with 'AAAA...' (10 repetitions, 80 bits) ===");
        
        // Test with delta=0 (basic consistency)
        let analysis_no_delta = find_best_width(&bits, 1, 20, 0);
        println!("\nDelta = 0 (Basic Consistency Check):");
        let mut sorted = analysis_no_delta.width_scores.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (i, (width, score)) in sorted.iter().take(5).enumerate() {
            println!("  {}. Width {}: {:.6}", i + 1, width, score);
        }
        
        // Test with delta=10
        let analysis_delta_10 = find_best_width(&bits, 1, 20, 10);
        println!("\nDelta = 10 (Compare frames 10 steps apart):");
        let mut sorted = analysis_delta_10.width_scores.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (i, (width, score)) in sorted.iter().take(5).enumerate() {
            println!("  {}. Width {}: {:.6}", i + 1, width, score);
        }
        
        println!("\n--- Explanation ---");
        println!("Delta measures repetition period, not frame width!");
        println!("With 10 identical 'A' chars, frames repeat every 1 position.");
        println!("Width 4 with delta=10 means: bit at position X equals bit at position X+40");
        println!("Since all chars are 'A', ANY width will show perfect repetition at ANY delta.");
        println!("This is why delta is NOT useful for uniform data - use delta=0 instead.");
    }
    
    #[test]
    fn test_size_penalty_effect() {
        // Create a longer ASCII pattern to test larger widths
        let mut bits = BitVec::<u8, Msb0>::new();
        
        // Create 200 bytes of ASCII "ABCDEFGH" repeated
        let pattern = "ABCDEFGH".as_bytes();
        for _ in 0..25 {
            for &byte in pattern {
                for i in (0..8).rev() {
                    bits.push((byte >> i) & 1 != 0);
                }
            }
        }
        
        println!("\n=== Testing Width Detection with {} bits ({} bytes) ===", bits.len(), bits.len() / 8);
        
        // Test specific widths to see raw scores
        let test_widths = vec![8, 16, 32, 64, 96, 128, 160, 192];
        
        println!("\nWidth | Score    | Samples");
        println!("------|----------|--------");
        
        for &width in &test_widths {
            let (score, _) = score_width(&bits, width);
            let num_samples = bits.len() / width;
            
            println!("{:5} | {:8.6} | {:7}", width, score, num_samples);
        }
        
        println!("\n--- Analysis ---");
        println!("No artificial penalties - scores reflect actual bit pattern consistency.");
        println!("Scores may vary due to sample size and actual data patterns.");
        println!("Width 8 should score highest for ASCII data (MSB always 0).");
        println!("Multiples of 8 (16, 24, 32...) will score similarly due to repetition.");
        println!("Algorithm prefers smallest width when scores are equal (within 0.0001%).");
    }
}
