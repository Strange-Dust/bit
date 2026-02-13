// Frame width detection - automatic detection of bit framing patterns
//
// Two-phase algorithm:
// 1. XOR-autocorrelation for fast fundamental period detection
// 2. Sampled entropy scoring for UI display and validation

use bitvec::prelude::*;

// Algorithm constants
const AUTOCORR_SAMPLES: usize = 2048;
const ENTROPY_MAX_FRAMES: usize = 256;
const PEAK_THRESHOLD_DELTA: f64 = 0.05;
const HARMONIC_SCORE_RATIO: f64 = 0.85;
const CANDIDATE_ENTROPY_RATIO: f64 = 0.7;
const ENTROPY_VALIDATION_RATIO: f64 = 0.5;
const TOP_SCORE_BAND: f64 = 0.95;

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

// --- New algorithm functions ---

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// XOR-autocorrelation: for each lag L, sample bit positions and measure how often
/// bit[i] == bit[i + L]. High match rate = data repeats at period L.
fn xor_autocorrelation_sampled(
    bits: &BitVec<u8, Msb0>,
    min_lag: usize,
    max_lag: usize,
    num_samples: usize,
) -> Vec<(usize, f64)> {
    let mut results = Vec::with_capacity(max_lag - min_lag + 1);

    for lag in min_lag..=max_lag {
        let usable_len = bits.len().saturating_sub(lag);
        if usable_len < 2 {
            results.push((lag, 0.5));
            continue;
        }

        let actual_samples = num_samples.min(usable_len);
        let stride = if actual_samples <= 1 { 1 } else { usable_len / actual_samples };
        let stride = stride.max(1);

        let mut matches = 0usize;
        let mut total = 0usize;

        let mut pos = 0;
        while pos < usable_len && total < actual_samples {
            if pos + lag < bits.len() {
                if bits[pos] == bits[pos + lag] {
                    matches += 1;
                }
                total += 1;
            }
            pos += stride;
        }

        let correlation = if total > 0 {
            matches as f64 / total as f64
        } else {
            0.5
        };

        results.push((lag, correlation));
    }

    results
}

/// Find candidate periods from autocorrelation data.
/// Returns peaks sorted by lag (smallest first), with harmonics filtered out.
fn find_candidate_periods(correlations: &[(usize, f64)]) -> Vec<(usize, f64)> {
    if correlations.is_empty() {
        return vec![];
    }

    // Compute median correlation for dynamic threshold
    let mut sorted_scores: Vec<f64> = correlations.iter().map(|(_, c)| *c).collect();
    sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = sorted_scores[sorted_scores.len() / 2];
    let threshold = median + PEAK_THRESHOLD_DELTA;

    // Find local maxima above threshold
    let mut peaks: Vec<(usize, f64)> = Vec::new();
    for i in 0..correlations.len() {
        let (lag, score) = correlations[i];
        if score <= threshold {
            continue;
        }

        // Check local maximum: score >= both neighbors
        let left_ok = i == 0 || score >= correlations[i - 1].1;
        let right_ok = i == correlations.len() - 1 || score >= correlations[i + 1].1;

        if left_ok && right_ok {
            peaks.push((lag, score));
        }
    }

    // Sort by lag (smallest first) - they should already be, but ensure it
    peaks.sort_by_key(|(lag, _)| *lag);

    // Filter harmonics: remove peaks whose lag is a multiple of an earlier peak
    let mut fundamental_peaks: Vec<(usize, f64)> = Vec::new();
    for &(lag, score) in &peaks {
        let is_harmonic = fundamental_peaks.iter().any(|&(fund_lag, fund_score)| {
            lag != fund_lag
                && lag % fund_lag == 0
                && fund_score >= score * HARMONIC_SCORE_RATIO
        });

        if !is_harmonic {
            fundamental_peaks.push((lag, score));
        }
    }

    fundamental_peaks
}

/// Sampled entropy scoring - same logic as score_width() but only samples up to
/// max_frames evenly-spaced frames. No width_penalty applied.
fn score_width_sampled(
    bits: &BitVec<u8, Msb0>,
    width: usize,
    max_frames: usize,
) -> (f64, Vec<f64>) {
    if width == 0 || bits.len() < width * 2 {
        return (0.0, vec![]);
    }

    let total_frames = bits.len() / width;
    if total_frames < 3 {
        return (0.0, vec![]);
    }

    // Select which frames to sample
    let num_frames = total_frames.min(max_frames);
    let stride = if num_frames <= 1 { 1 } else { total_frames / num_frames };
    let stride = stride.max(1);

    let mut bit_position_entropies = Vec::with_capacity(width);
    let mut total_entropy = 0.0;

    for bit_pos in 0..width {
        let mut count_0 = 0usize;
        let mut count_1 = 0usize;

        let mut frame_idx = 0;
        let mut sampled = 0;
        while frame_idx < total_frames && sampled < num_frames {
            let bit_idx = frame_idx * width + bit_pos;
            if bit_idx < bits.len() {
                if bits[bit_idx] {
                    count_1 += 1;
                } else {
                    count_0 += 1;
                }
            }
            sampled += 1;
            frame_idx += stride;
        }

        let total = (count_0 + count_1) as f64;
        let entropy = if total > 0.0 {
            let p0 = count_0 as f64 / total;
            let p1 = count_1 as f64 / total;
            let mut h = 0.0;
            if p0 > 0.0 { h -= p0 * p0.log2(); }
            if p1 > 0.0 { h -= p1 * p1.log2(); }
            h
        } else {
            0.0
        };

        bit_position_entropies.push(entropy);
        total_entropy += entropy;
    }

    let avg_entropy = total_entropy / width as f64;

    let low_entropy_count = bit_position_entropies.iter()
        .filter(|&&e| e < 0.3)
        .count();
    let low_entropy_ratio = low_entropy_count as f64 / width as f64;

    let sample_confidence = if num_frames >= 30 {
        1.0
    } else {
        num_frames as f64 / 30.0
    };

    let base_score = 1.0 - avg_entropy;
    let structure_bonus = low_entropy_ratio * 0.5;
    let score = (base_score + structure_bonus) * sample_confidence;

    let consistency_scores: Vec<f64> = bit_position_entropies
        .iter()
        .map(|&e| 1.0 - e)
        .collect();

    (score, consistency_scores)
}

/// Sampled delta scoring - same as score_width_with_delta() but sampling frames.
fn score_width_with_delta_sampled(
    bits: &BitVec<u8, Msb0>,
    width: usize,
    delta: usize,
    max_frames: usize,
) -> (f64, Vec<f64>) {
    if width == 0 || bits.len() < width * (delta + 2) {
        return (0.0, vec![]);
    }

    let total_samples = bits.len() / width - delta;
    if total_samples < 3 {
        return (0.0, vec![]);
    }

    let num_samples = total_samples.min(max_frames);
    let stride = if num_samples <= 1 { 1 } else { total_samples / num_samples };
    let stride = stride.max(1);

    let mut bit_position_scores = Vec::with_capacity(width);
    let mut total_score = 0.0;

    for bit_pos in 0..width {
        let mut pattern_consistency = 0.0;
        let mut valid_samples = 0;

        let mut frame_idx = 0;
        let mut sampled = 0;
        while frame_idx < total_samples && sampled < num_samples {
            let bit_idx = frame_idx * width + bit_pos;
            let delta_idx = (frame_idx + delta) * width + bit_pos;

            if delta_idx < bits.len() {
                if bits[bit_idx] == bits[delta_idx] {
                    pattern_consistency += 1.0;
                }
                valid_samples += 1;
            }
            sampled += 1;
            frame_idx += stride;
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

/// Entropy-only fallback: find best width using GCD of top-scoring widths.
fn fallback_entropy_best(width_scores: &[(usize, f64)]) -> usize {
    if width_scores.is_empty() {
        return 1;
    }

    let max_score = width_scores.iter()
        .map(|(_, s)| *s)
        .fold(0.0f64, f64::max);

    if max_score <= 0.0 {
        return width_scores[0].0;
    }

    let threshold = max_score * TOP_SCORE_BAND;
    let top_widths: Vec<usize> = width_scores.iter()
        .filter(|(_, s)| *s >= threshold)
        .map(|(w, _)| *w)
        .collect();

    if top_widths.is_empty() {
        return width_scores[0].0;
    }

    // Compute GCD of all top widths
    let mut g = top_widths[0];
    for &w in &top_widths[1..] {
        g = gcd(g, w);
    }

    // Check if the GCD width has a decent score
    if let Some(&(_, gcd_score)) = width_scores.iter().find(|(w, _)| *w == g) && gcd_score >= max_score * ENTROPY_VALIDATION_RATIO {
        return g;
    }

    // Otherwise return smallest top width
    *top_widths.iter().min().unwrap_or(&1)
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

    // Phase 1: XOR-autocorrelation (fast candidate screening)
    let correlations = xor_autocorrelation_sampled(bits, min_width, max_width, AUTOCORR_SAMPLES);

    // Phase 2: Extract candidate periods with harmonic filtering
    let candidates = find_candidate_periods(&correlations);

    // Phase 3: Sampled entropy scoring for all widths (populates UI data)
    for width in min_width..=max_width {
        let (score, bit_patterns) = if delta > 0 {
            score_width_with_delta_sampled(bits, width, delta, ENTROPY_MAX_FRAMES)
        } else {
            score_width_sampled(bits, width, ENTROPY_MAX_FRAMES)
        };

        width_scores.push((width, score));
        bit_position_patterns.push(bit_patterns);
    }

    // Phase 4: Best width selection
    let best_width;
    let best_score;

    // Find best entropy score for validation
    let max_entropy_score = width_scores.iter()
        .map(|(_, s)| *s)
        .fold(0.0f64, f64::max);

    if !candidates.is_empty() && max_entropy_score > 0.0 {
        // Find best entropy score among autocorrelation candidates
        let best_candidate_entropy = candidates.iter()
            .filter_map(|&(lag, _)| {
                if lag >= min_width && lag <= max_width {
                    width_scores.iter().find(|(w, _)| *w == lag).map(|(_, s)| *s)
                } else {
                    None
                }
            })
            .fold(0.0f64, f64::max);

        // Candidate must have good entropy relative to both:
        // - Other candidates (filters coincidental sub-period matches)
        // - Global max (filters candidates with poor structural alignment)
        let threshold = (best_candidate_entropy * CANDIDATE_ENTROPY_RATIO)
            .max(max_entropy_score * ENTROPY_VALIDATION_RATIO);

        // Try candidates smallest-first (fundamental priority)
        let mut found = false;
        let mut selected_width = candidates[0].0;
        let mut selected_score = 0.0;

        for &(lag, _) in &candidates {
            if lag < min_width || lag > max_width {
                continue;
            }
            if let Some(&(_, entropy_score)) = width_scores.iter().find(|(w, _)| *w == lag) && entropy_score >= threshold {
                selected_width = lag;
                selected_score = entropy_score;
                found = true;
                break;
            }
        }

        if found {
            best_width = selected_width;
            best_score = selected_score;
        } else {
            // Autocorrelation candidates didn't validate — use entropy fallback
            let fb = fallback_entropy_best(&width_scores);
            best_width = fb;
            best_score = width_scores.iter()
                .find(|(w, _)| *w == fb)
                .map(|(_, s)| *s)
                .unwrap_or(0.0);
        }
    } else {
        // No autocorrelation peaks — pure entropy fallback
        let fb = fallback_entropy_best(&width_scores);
        best_width = fb;
        best_score = width_scores.iter()
            .find(|(w, _)| *w == fb)
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
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

        // New algorithm should detect exactly width 8
        assert_eq!(analysis.best_width, 8,
            "Best width should be 8, not {}", analysis.best_width);
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
        println!("- Sync bits always align to same columns -> entropy ~ 0");
        println!("- Structured address bits -> low entropy");
        println!("Wrong widths scramble the structure -> higher entropy (closer to 0.5)");

        let analysis = find_best_width(&bits, 8, 32, 0);
        println!("\nDetected width: {} (expected: 16)", analysis.best_width);

        // New algorithm should detect width 16 as the fundamental
        assert_eq!(analysis.best_width, 16,
            "Should detect width 16, got {}", analysis.best_width);
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

    #[test]
    fn test_autocorrelation_basic() {
        // Create data with a clear 8-bit period
        let mut bits = BitVec::<u8, Msb0>::new();
        for _ in 0..100 {
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,0,1]); // 'A'
        }

        let correlations = xor_autocorrelation_sampled(&bits, 1, 20, AUTOCORR_SAMPLES);

        // Lag 8 should have the highest correlation (perfect match)
        let corr_8 = correlations.iter().find(|(lag, _)| *lag == 8).unwrap().1;
        println!("Autocorrelation at lag 8: {:.4}", corr_8);
        assert!(corr_8 > 0.9, "Lag 8 correlation should be very high: {}", corr_8);

        // Lag 7 and 9 should be lower (misaligned)
        let corr_7 = correlations.iter().find(|(lag, _)| *lag == 7).unwrap().1;
        let corr_9 = correlations.iter().find(|(lag, _)| *lag == 9).unwrap().1;
        println!("Autocorrelation at lag 7: {:.4}, lag 9: {:.4}", corr_7, corr_9);
        assert!(corr_8 > corr_7, "Lag 8 should beat lag 7");
        assert!(corr_8 > corr_9, "Lag 8 should beat lag 9");
    }

    #[test]
    fn test_fundamental_over_harmonics() {
        // 8-bit ASCII data — algorithm should detect width 8, not 16/24/32/64
        let mut bits = BitVec::<u8, Msb0>::new();
        let ascii_text = "The quick brown fox jumps over the lazy dog. ";
        for _ in 0..10 {
            for &byte in ascii_text.as_bytes() {
                for i in (0..8).rev() {
                    bits.push((byte >> i) & 1 != 0);
                }
            }
        }

        println!("Testing fundamental detection with {} bits of ASCII text", bits.len());

        let analysis = find_best_width(&bits, 1, 64, 0);

        println!("Detected width: {} (expected: 8)", analysis.best_width);

        // Must detect width 8, not any harmonic
        assert_eq!(analysis.best_width, 8,
            "Should detect fundamental width 8, not harmonic {}", analysis.best_width);
    }

    #[test]
    fn test_protocol_sync_detection() {
        // Protocol with 32-bit frames: 8-bit sync (0xAA), 24-bit payload
        let mut bits = BitVec::<u8, Msb0>::new();

        for i in 0..100 {
            // Sync byte: 0xAA = 10101010
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 1,0,1,0,1,0,1,0]);
            // 3 bytes of varying payload
            let b1 = (i * 3) as u8;
            let b2 = (i * 7 + 1) as u8;
            let b3 = (i * 13 + 2) as u8;
            for byte in [b1, b2, b3] {
                for j in (0..8).rev() {
                    bits.push((byte >> j) & 1 != 0);
                }
            }
        }

        println!("Testing protocol sync detection with {} bits", bits.len());

        let analysis = find_best_width(&bits, 8, 64, 0);

        println!("Detected width: {} (expected: 32)", analysis.best_width);

        assert_eq!(analysis.best_width, 32,
            "Should detect 32-bit frame width, got {}", analysis.best_width);
    }

    #[test]
    fn test_sampled_vs_full_consistency() {
        // On small data, sampled scoring should agree with full scoring
        let mut bits = BitVec::<u8, Msb0>::new();
        for _ in 0..20 {
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,0,1]); // 'A'
            bits.extend_from_bitslice(&bitvec![u8, Msb0; 0,1,0,0,0,0,1,0]); // 'B'
        }

        // For small data (40 frames at width 8), sampled and full should be very close
        let (full_score, _) = score_width(&bits, 8);
        let (sampled_score, _) = score_width_sampled(&bits, 8, ENTROPY_MAX_FRAMES);

        println!("Full score: {:.6}, Sampled score: {:.6}", full_score, sampled_score);

        // Sampled doesn't apply width_penalty, so it may be slightly higher.
        // Just check they're in the same ballpark (within 20% relative)
        let ratio = sampled_score / full_score;
        assert!(ratio > 0.8 && ratio < 1.25,
            "Sampled and full scores should be close: full={:.4}, sampled={:.4}, ratio={:.4}",
            full_score, sampled_score, ratio);
    }
}
