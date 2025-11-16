# Frame Width Detection Algorithm

## Overview

Automatically detect the most probable frame width by analyzing bit patterns and structural repetition in binary data. This is particularly useful for identifying byte boundaries (width=8), word boundaries (width=16/32), or custom framing in protocols.

## Core Concept

Many data formats have structural patterns that repeat at regular intervals:
- **ASCII text**: Bit 7 is often 0 (values < 128), creating a pattern every 8 bits
- **UTF-8**: High bits follow specific patterns for multi-byte sequences
- **Network protocols**: Headers repeat at fixed frame widths
- **Structured data**: Alignment padding creates detectable patterns

## Algorithm Design

### Step 1: Bit Position Analysis

For each candidate width `w` (typically 4 to 256), analyze each bit position `p` within the frame:

```
For width w:
  For bit_position p in 0..w:
    Count occurrences of 0 and 1 at positions p, p+w, p+2w, p+3w, ...
    Calculate consistency score
```

**Consistency Score**: Measures how uniform a bit position is across all frames
- If a position is always 0 or always 1: high score
- If a position is random (50/50): low score

Formula: `score = |count_0 - count_1| / total_samples`
- Score of 1.0 = perfectly consistent (all same value)
- Score of 0.0 = perfectly random (50/50 split)

### Step 2: Width Scoring

For each candidate width, aggregate the consistency scores:

```rust
fn score_width(bits: &BitVec, width: usize) -> f64 {
    let mut total_score = 0.0;
    let num_frames = bits.len() / width;
    
    for bit_pos in 0..width {
        let mut count_0 = 0;
        let mut count_1 = 0;
        
        // Sample this bit position across all frames
        for frame_idx in 0..num_frames {
            let bit_idx = frame_idx * width + bit_pos;
            if bits[bit_idx] {
                count_1 += 1;
            } else {
                count_0 += 1;
            }
        }
        
        // Consistency: how far from 50/50 split
        let consistency = (count_0 as f64 - count_1 as f64).abs() 
                         / (count_0 + count_1) as f64;
        total_score += consistency;
    }
    
    // Average consistency across all bit positions
    total_score / width as f64
}
```

### Step 3: Find Maximum

The width with the highest average consistency score is the most probable frame width.

```rust
fn find_best_width(bits: &BitVec, min_width: usize, max_width: usize) -> usize {
    let mut best_width = min_width;
    let mut best_score = 0.0;
    
    for width in min_width..=max_width {
        let score = score_width(bits, width);
        if score > best_score {
            best_score = score;
            best_width = width;
        }
    }
    
    best_width
}
```

## Enhancements

### Delta-Based Analysis

Instead of just checking bit consistency, analyze bit transitions (XOR with previous bit):

```rust
fn score_width_with_delta(bits: &BitVec, width: usize, delta: usize) -> f64 {
    let mut total_score = 0.0;
    
    for bit_pos in 0..width {
        let mut pattern_consistency = 0.0;
        let num_samples = bits.len() / width - delta;
        
        for frame_idx in 0..num_samples {
            let bit_idx = frame_idx * width + bit_pos;
            let delta_idx = (frame_idx + delta) * width + bit_pos;
            
            // Check if bits at same position but delta frames apart match
            if bits[bit_idx] == bits[delta_idx] {
                pattern_consistency += 1.0;
            }
        }
        
        total_score += pattern_consistency / num_samples as f64;
    }
    
    total_score / width as f64
}
```

This helps detect periodic patterns where structure repeats every N frames.

### Multi-Scale Analysis

Check both the base width and its multiples:

- Width 8 might score well for byte-aligned data
- Width 16 might score even better if data is 16-bit aligned
- Width 24 might indicate RGB pixel data

Return both the fundamental width and detected multiples.

## Implementation Structure

### New Operation: FindFrameWidth

```rust
// In src/processing/operations.rs
pub enum BitOperation {
    // ... existing operations
    FindFrameWidth {
        name: String,
        min_width: usize,    // Minimum width to test (e.g., 4)
        max_width: usize,    // Maximum width to test (e.g., 256)
        delta: usize,        // Look-ahead distance for pattern matching
        enabled: bool,
    },
}
```

### Analysis Result Structure

```rust
pub struct FrameWidthAnalysis {
    pub width_scores: Vec<(usize, f64)>,  // (width, score) pairs
    pub best_width: usize,
    pub best_score: f64,
    pub bit_position_patterns: Vec<Vec<f64>>,  // Per-width, per-position scores
}
```

## Visualization with egui_plot

### 1. Line Chart: Scores vs Width

```rust
use egui_plot::{Line, Plot, PlotPoints};

fn plot_width_scores(ui: &mut egui::Ui, analysis: &FrameWidthAnalysis) {
    Plot::new("width_scores_plot")
        .view_aspect(2.0)
        .label_formatter(|name, value| {
            format!("Width: {}\nScore: {:.4}", value.x, value.y)
        })
        .show(ui, |plot_ui| {
            // Convert scores to plot points
            let points: PlotPoints = analysis.width_scores
                .iter()
                .map(|(width, score)| [*width as f64, *score])
                .collect();
            
            let line = Line::new(points)
                .color(egui::Color32::from_rgb(100, 150, 255))
                .width(2.0)
                .name("Consistency Score");
            
            plot_ui.line(line);
            
            // Highlight the best width
            let best_point = PlotPoints::new(vec![
                [analysis.best_width as f64, analysis.best_score]
            ]);
            
            let best_marker = egui_plot::Points::new(best_point)
                .color(egui::Color32::from_rgb(255, 100, 100))
                .radius(6.0)
                .name("Best Width");
            
            plot_ui.points(best_marker);
        });
}
```

### 2. Heatmap: Bit Position Consistency

```rust
fn plot_bit_position_heatmap(ui: &mut egui::Ui, analysis: &FrameWidthAnalysis) {
    // Show consistency score for each bit position at the best width
    let best_width_idx = analysis.width_scores
        .iter()
        .position(|(w, _)| *w == analysis.best_width)
        .unwrap();
    
    let bit_patterns = &analysis.bit_position_patterns[best_width_idx];
    
    Plot::new("bit_position_heatmap")
        .view_aspect(3.0)
        .show(ui, |plot_ui| {
            // Create bars for each bit position
            let bars: Vec<_> = bit_patterns
                .iter()
                .enumerate()
                .map(|(pos, &score)| {
                    egui_plot::Bar::new(pos as f64, score)
                        .width(0.8)
                })
                .collect();
            
            let bar_chart = egui_plot::BarChart::new(bars)
                .color(egui::Color32::from_rgb(150, 200, 150))
                .name("Bit Position Consistency");
            
            plot_ui.bar_chart(bar_chart);
        });
}
```

### 3. Multiple Width Comparison

```rust
fn plot_multiple_widths(ui: &mut egui::Ui, analysis: &FrameWidthAnalysis) {
    Plot::new("multi_width_comparison")
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            // Show top 5 candidate widths
            let top_widths: Vec<_> = analysis.width_scores
                .iter()
                .cloned()
                .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
                .take(5)
                .collect();
            
            for (i, (width, _)) in top_widths.iter().enumerate() {
                let width_idx = analysis.width_scores
                    .iter()
                    .position(|(w, _)| w == width)
                    .unwrap();
                
                let bit_patterns = &analysis.bit_position_patterns[width_idx];
                let points: PlotPoints = bit_patterns
                    .iter()
                    .enumerate()
                    .map(|(pos, &score)| [pos as f64, score])
                    .collect();
                
                let color = match i {
                    0 => egui::Color32::from_rgb(255, 100, 100),
                    1 => egui::Color32::from_rgb(100, 255, 100),
                    2 => egui::Color32::from_rgb(100, 100, 255),
                    3 => egui::Color32::from_rgb(255, 255, 100),
                    _ => egui::Color32::from_rgb(200, 100, 255),
                };
                
                let line = Line::new(points)
                    .color(color)
                    .name(format!("Width {}", width));
                
                plot_ui.line(line);
            }
        });
}
```

## UI Integration

### Window for Frame Width Finder

```rust
// In src/ui/windows.rs
pub fn show_frame_width_finder(
    ctx: &egui::Context,
    app: &mut BitApp,
) {
    egui::Window::new("🔍 Find Frame Width")
        .open(&mut app.show_frame_width_finder)
        .show(ctx, |ui| {
            ui.heading("Automatic Frame Width Detection");
            
            ui.horizontal(|ui| {
                ui.label("Min Width:");
                ui.add(egui::DragValue::new(&mut app.frame_width_min)
                    .range(1..=1024));
                
                ui.label("Max Width:");
                ui.add(egui::DragValue::new(&mut app.frame_width_max)
                    .range(1..=1024));
            });
            
            ui.horizontal(|ui| {
                ui.label("Delta (pattern look-ahead):");
                ui.add(egui::DragValue::new(&mut app.frame_width_delta)
                    .range(1..=100));
            });
            
            if ui.button("🔍 Analyze").clicked() {
                app.run_frame_width_analysis();
            }
            
            ui.separator();
            
            if let Some(analysis) = &app.frame_width_analysis {
                ui.label(format!(
                    "Best Width: {} (score: {:.4})", 
                    analysis.best_width, 
                    analysis.best_score
                ));
                
                if ui.button("Apply Width").clicked() {
                    app.viewer.frame_length = analysis.best_width;
                }
                
                ui.separator();
                ui.heading("Width Scores");
                plot_width_scores(ui, analysis);
                
                ui.separator();
                ui.heading("Bit Position Consistency");
                plot_bit_position_heatmap(ui, analysis);
            }
        });
}
```

## Expected Results

### ASCII Text (width=8)
- Bit 7: ~100% zeros (ASCII < 128)
- Bits 0-6: Random distribution
- **Score**: High for width 8, medium for multiples (16, 24, 32)

### 16-bit Little-Endian Integers
- Even byte positions might show different patterns than odd bytes
- **Score**: High for width 16

### RGB24 Images
- Repeating patterns every 24 bits (3 bytes)
- **Score**: High for width 24

## Performance Considerations

- **Sample Size**: For large files (>1MB), sample every Nth bit to speed up analysis
- **Width Range**: Limit max_width to 256 to keep analysis fast
- **Caching**: Cache analysis results to avoid recomputing

## Future Enhancements

1. **Autocorrelation**: Use FFT-based autocorrelation for faster pattern detection
2. **Entropy Analysis**: Calculate Shannon entropy at each bit position
3. **Multi-Delta**: Test multiple delta values simultaneously
4. **Confidence Intervals**: Provide statistical confidence in the detected width
5. **Pattern Templates**: Pre-defined templates for common formats (ASCII, UTF-8, network protocols)
