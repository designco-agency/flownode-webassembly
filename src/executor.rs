//! Node graph execution engine
//!
//! Executes the node graph in topological order, processing images through connected nodes.

use std::collections::HashMap;
use uuid::Uuid;

use crate::graph::NodeGraph;
use crate::nodes::{NodeType, NodeProperties};
use crate::image_data::ImageData;

/// Result of executing a node
#[derive(Clone)]
pub enum NodeOutput {
    /// Image/content output
    Image(ImageData),
    /// Text output
    Text(String),
    /// No output
    None,
}

/// Execution context for running the node graph
pub struct Executor {
    /// Cached outputs from nodes
    outputs: HashMap<Uuid, NodeOutput>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }
    
    /// Execute the entire graph and return the final output
    /// For now, returns the result of any Adjust or Effects node
    pub fn execute(&mut self, graph: &NodeGraph, input_images: &HashMap<u64, ImageData>) -> Result<Option<ImageData>, String> {
        // Clear previous outputs
        self.outputs.clear();
        
        // Get topological order
        let order = self.topological_sort(graph)?;
        
        // Execute each node in order
        for node_id in order {
            self.execute_node(graph, node_id, input_images)?;
        }
        
        // Find any image processing node and return its output
        // Priority: Adjust > Effects > Image
        let mut result: Option<ImageData> = None;
        
        for (id, node) in graph.nodes_iter() {
            match node.node_type {
                NodeType::Adjust | NodeType::Effects => {
                    if let Some(NodeOutput::Image(img)) = self.outputs.get(id) {
                        result = Some(img.clone());
                    }
                }
                NodeType::Image => {
                    if result.is_none() {
                        if let Some(NodeOutput::Image(img)) = self.outputs.get(id) {
                            result = Some(img.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(result)
    }
    
    /// Execute a single node
    fn execute_node(
        &mut self,
        graph: &NodeGraph,
        node_id: Uuid,
        input_images: &HashMap<u64, ImageData>,
    ) -> Result<(), String> {
        let node = graph.nodes_iter()
            .find(|(id, _)| **id == node_id)
            .map(|(_, n)| n)
            .ok_or("Node not found")?;
        
        let output = match &node.properties {
            // === Image Node ===
            NodeProperties::Image { texture_id, .. } => {
                if let Some(tex_id) = texture_id {
                    if let Some(img) = input_images.get(tex_id) {
                        NodeOutput::Image(img.clone())
                    } else {
                        NodeOutput::None
                    }
                } else {
                    NodeOutput::None
                }
            }
            
            // === Adjust Node (full color grading) ===
            NodeProperties::Adjust { 
                brightness, contrast, saturation, exposure,
                highlights, shadows, temperature, tint,
                vibrance, gamma, ..
            } => {
                let input = self.get_input_image(graph, node_id)?;
                if let Some(img) = input {
                    let result = self.apply_adjustments(
                        &img,
                        *brightness, *contrast, *saturation, *exposure,
                        *highlights, *shadows, *temperature, *tint,
                        *vibrance, *gamma
                    );
                    NodeOutput::Image(result)
                } else {
                    NodeOutput::None
                }
            }
            
            // === Effects Node ===
            NodeProperties::Effects {
                gaussian_blur, directional_blur, directional_blur_angle,
                progressive_blur, progressive_blur_direction, progressive_blur_falloff,
                glass_blinds, glass_blinds_frequency, glass_blinds_angle, glass_blinds_phase,
                grain, grain_size, grain_monochrome, grain_seed,
                sharpen, vignette, vignette_roundness, vignette_smoothness
            } => {
                let input = self.get_input_image(graph, node_id)?;
                if let Some(img) = input {
                    let mut result = img;
                    
                    // Apply effects in order
                    if *gaussian_blur > 0.0 {
                        result = self.apply_blur(&result, (*gaussian_blur * 0.5) as u32);
                    }
                    if *directional_blur > 0.0 {
                        result = self.apply_directional_blur(&result, *directional_blur / 100.0, *directional_blur_angle);
                    }
                    if *progressive_blur > 0.0 {
                        result = self.apply_progressive_blur(&result, *progressive_blur / 100.0, progressive_blur_direction, *progressive_blur_falloff / 100.0);
                    }
                    if *glass_blinds > 0.0 {
                        result = self.apply_glass_blinds(&result, *glass_blinds / 100.0, *glass_blinds_frequency, *glass_blinds_angle, *glass_blinds_phase / 100.0);
                    }
                    if *sharpen > 0.0 {
                        result = self.apply_sharpen(&result, *sharpen / 100.0);
                    }
                    if *grain > 0.0 {
                        result = self.apply_grain_advanced(&result, *grain / 100.0, *grain_size, *grain_monochrome, *grain_seed);
                    }
                    if *vignette > 0.0 {
                        result = self.apply_vignette(&result, *vignette / 100.0, *vignette_roundness / 100.0, *vignette_smoothness / 100.0);
                    }
                    
                    NodeOutput::Image(result)
                } else {
                    NodeOutput::None
                }
            }
            
            // === Text Nodes ===
            NodeProperties::Text { text } => {
                NodeOutput::Text(text.clone())
            }
            
            NodeProperties::Concat { separator } => {
                // Get text inputs and join them
                let text1 = self.get_input_text(graph, node_id, 0);
                let text2 = self.get_input_text(graph, node_id, 1);
                let result = format!("{}{}{}", text1.unwrap_or_default(), separator, text2.unwrap_or_default());
                NodeOutput::Text(result)
            }
            
            // Pass through for content nodes
            NodeProperties::Content { .. } | NodeProperties::Bucket { .. } | NodeProperties::Compare {} => {
                let input = self.get_input_image(graph, node_id)?;
                if let Some(img) = input {
                    NodeOutput::Image(img)
                } else {
                    NodeOutput::None
                }
            }
            
            // Other node types - not yet implemented
            _ => NodeOutput::None,
        };
        
        self.outputs.insert(node_id, output);
        Ok(())
    }
    
    /// Get the image input for a node (from first connected input)
    fn get_input_image(&self, graph: &NodeGraph, node_id: Uuid) -> Result<Option<ImageData>, String> {
        for conn in graph.connections_iter() {
            if conn.to_node == node_id && conn.to_slot == 0 {
                if let Some(NodeOutput::Image(img)) = self.outputs.get(&conn.from_node) {
                    return Ok(Some(img.clone()));
                }
            }
        }
        Ok(None)
    }
    
    /// Get text input at a specific slot
    fn get_input_text(&self, graph: &NodeGraph, node_id: Uuid, slot: usize) -> Option<String> {
        for conn in graph.connections_iter() {
            if conn.to_node == node_id && conn.to_slot == slot {
                if let Some(NodeOutput::Text(text)) = self.outputs.get(&conn.from_node) {
                    return Some(text.clone());
                }
            }
        }
        None
    }
    
    /// Topological sort of the graph
    fn topological_sort(&self, graph: &NodeGraph) -> Result<Vec<Uuid>, String> {
        let mut result = Vec::new();
        let mut visited = HashMap::new();
        
        let node_ids: Vec<Uuid> = graph.nodes_iter().map(|(id, _)| *id).collect();
        
        for node_id in node_ids {
            self.visit_node(graph, node_id, &mut visited, &mut result)?;
        }
        
        Ok(result)
    }
    
    fn visit_node(
        &self,
        graph: &NodeGraph,
        node_id: Uuid,
        visited: &mut HashMap<Uuid, bool>,
        result: &mut Vec<Uuid>,
    ) -> Result<(), String> {
        if let Some(&in_progress) = visited.get(&node_id) {
            if in_progress {
                return Err("Cycle detected in graph".to_string());
            }
            return Ok(());
        }
        
        visited.insert(node_id, true);
        
        // Visit all nodes that this node depends on
        for conn in graph.connections_iter() {
            if conn.to_node == node_id {
                self.visit_node(graph, conn.from_node, visited, result)?;
            }
        }
        
        visited.insert(node_id, false);
        result.push(node_id);
        
        Ok(())
    }
    
    // === Image Processing Functions ===
    
    /// Apply all adjust node parameters
    fn apply_adjustments(
        &self,
        img: &ImageData,
        brightness: f32,
        contrast: f32,
        saturation: f32,
        exposure: f32,
        highlights: f32,
        shadows: f32,
        temperature: f32,
        tint: f32,
        vibrance: f32,
        gamma: f32,
    ) -> ImageData {
        let mut output = img.pixels.as_ref().clone();
        
        // Convert -100..100 ranges to usable values
        let brightness_factor = brightness / 100.0;
        let contrast_factor = 1.0 + (contrast / 100.0);
        let saturation_factor = 1.0 + (saturation / 100.0);
        let exposure_factor = (exposure / 50.0).exp2(); // 2^(exposure/50) for natural feel
        let gamma_value = 1.0 / (1.0 + gamma / 100.0).max(0.1);
        let temp_shift = temperature / 100.0;
        let tint_shift = tint / 100.0;
        let vibrance_factor = vibrance / 100.0;
        
        for chunk in output.chunks_exact_mut(4) {
            let mut r = chunk[0] as f32 / 255.0;
            let mut g = chunk[1] as f32 / 255.0;
            let mut b = chunk[2] as f32 / 255.0;
            
            // Exposure
            r *= exposure_factor;
            g *= exposure_factor;
            b *= exposure_factor;
            
            // Temperature (shift R/B balance)
            r += temp_shift * 0.1;
            b -= temp_shift * 0.1;
            
            // Tint (shift G/M balance)
            g += tint_shift * 0.05;
            
            // Brightness
            r += brightness_factor;
            g += brightness_factor;
            b += brightness_factor;
            
            // Contrast
            r = (r - 0.5) * contrast_factor + 0.5;
            g = (g - 0.5) * contrast_factor + 0.5;
            b = (b - 0.5) * contrast_factor + 0.5;
            
            // Saturation
            let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
            r = luminance + (r - luminance) * saturation_factor;
            g = luminance + (g - luminance) * saturation_factor;
            b = luminance + (b - luminance) * saturation_factor;
            
            // Vibrance (saturation that affects less saturated colors more)
            let max_rgb = r.max(g).max(b);
            let min_rgb = r.min(g).min(b);
            let current_sat = if max_rgb > 0.0 { (max_rgb - min_rgb) / max_rgb } else { 0.0 };
            let vib_mult = 1.0 + vibrance_factor * (1.0 - current_sat);
            r = luminance + (r - luminance) * vib_mult;
            g = luminance + (g - luminance) * vib_mult;
            b = luminance + (b - luminance) * vib_mult;
            
            // Highlights/Shadows (simplified)
            if luminance > 0.5 {
                let highlight_factor = (luminance - 0.5) * 2.0 * (highlights / 200.0);
                r += highlight_factor;
                g += highlight_factor;
                b += highlight_factor;
            } else {
                let shadow_factor = (0.5 - luminance) * 2.0 * (shadows / 200.0);
                r += shadow_factor;
                g += shadow_factor;
                b += shadow_factor;
            }
            
            // Gamma
            r = r.max(0.0).powf(gamma_value);
            g = g.max(0.0).powf(gamma_value);
            b = b.max(0.0).powf(gamma_value);
            
            // Clamp and store
            chunk[0] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            chunk[1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            chunk[2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply gaussian blur (simplified box blur)
    fn apply_blur(&self, img: &ImageData, radius: u32) -> ImageData {
        if radius == 0 {
            return img.clone();
        }
        
        let radius = radius.min(50) as i32;
        let width = img.width as i32;
        let height = img.height as i32;
        let mut output = img.pixels.as_ref().clone();
        
        // Horizontal pass
        let mut temp = output.clone();
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut count = 0u32;
                
                for dx in -radius..=radius {
                    let sx = (x + dx).clamp(0, width - 1);
                    let idx = ((y * width + sx) * 4) as usize;
                    r_sum += img.pixels[idx] as u32;
                    g_sum += img.pixels[idx + 1] as u32;
                    b_sum += img.pixels[idx + 2] as u32;
                    count += 1;
                }
                
                let idx = ((y * width + x) * 4) as usize;
                temp[idx] = (r_sum / count) as u8;
                temp[idx + 1] = (g_sum / count) as u8;
                temp[idx + 2] = (b_sum / count) as u8;
            }
        }
        
        // Vertical pass
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut count = 0u32;
                
                for dy in -radius..=radius {
                    let sy = (y + dy).clamp(0, height - 1);
                    let idx = ((sy * width + x) * 4) as usize;
                    r_sum += temp[idx] as u32;
                    g_sum += temp[idx + 1] as u32;
                    b_sum += temp[idx + 2] as u32;
                    count += 1;
                }
                
                let idx = ((y * width + x) * 4) as usize;
                output[idx] = (r_sum / count) as u8;
                output[idx + 1] = (g_sum / count) as u8;
                output[idx + 2] = (b_sum / count) as u8;
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply sharpening (unsharp mask)
    fn apply_sharpen(&self, img: &ImageData, amount: f32) -> ImageData {
        // Create a blurred version
        let blurred = self.apply_blur(img, 1);
        let mut output = img.pixels.as_ref().clone();
        
        for i in (0..output.len()).step_by(4) {
            for c in 0..3 {
                let original = img.pixels[i + c] as f32;
                let blur = blurred.pixels[i + c] as f32;
                // Unsharp mask: original + amount * (original - blur)
                let sharpened = original + amount * (original - blur);
                output[i + c] = sharpened.clamp(0.0, 255.0) as u8;
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply film grain
    fn apply_grain(&self, img: &ImageData, amount: f32, monochrome: bool) -> ImageData {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut output = img.pixels.as_ref().clone();
        let mut hasher = DefaultHasher::new();
        
        for i in (0..output.len()).step_by(4) {
            // Simple pseudo-random based on position
            i.hash(&mut hasher);
            let noise = ((hasher.finish() % 256) as f32 / 255.0 - 0.5) * 2.0 * amount * 50.0;
            
            if monochrome {
                for c in 0..3 {
                    let val = output[i + c] as f32 + noise;
                    output[i + c] = val.clamp(0.0, 255.0) as u8;
                }
            } else {
                for c in 0..3 {
                    (i + c).hash(&mut hasher);
                    let color_noise = ((hasher.finish() % 256) as f32 / 255.0 - 0.5) * 2.0 * amount * 50.0;
                    let val = output[i + c] as f32 + color_noise;
                    output[i + c] = val.clamp(0.0, 255.0) as u8;
                }
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply vignette effect
    fn apply_vignette(&self, img: &ImageData, intensity: f32, roundness: f32, smoothness: f32) -> ImageData {
        let mut output = img.pixels.as_ref().clone();
        let width = img.width as f32;
        let height = img.height as f32;
        let cx = width / 2.0;
        let cy = height / 2.0;
        let max_dist = (cx * cx + cy * cy).sqrt();
        
        // Roundness: 0 = elliptical, 1 = circular
        let aspect = width / height;
        let x_scale = 1.0 + (1.0 - roundness) * (aspect - 1.0).abs();
        
        for y in 0..img.height {
            for x in 0..img.width {
                let dx = (x as f32 - cx) / x_scale;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt() / max_dist;
                
                // Vignette falloff based on smoothness
                let falloff_start = 0.3 + smoothness * 0.4;
                let vignette = if dist < falloff_start {
                    1.0
                } else {
                    let t = (dist - falloff_start) / (1.0 - falloff_start);
                    1.0 - t.powf(2.0 - smoothness) * intensity
                };
                
                let idx = ((y * img.width + x) * 4) as usize;
                for c in 0..3 {
                    output[idx + c] = (output[idx + c] as f32 * vignette).clamp(0.0, 255.0) as u8;
                }
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply directional (motion) blur
    fn apply_directional_blur(&self, img: &ImageData, amount: f32, angle: f32) -> ImageData {
        let mut output = img.pixels.as_ref().clone();
        let width = img.width as i32;
        let height = img.height as i32;
        
        // Convert angle to radians and calculate direction
        let angle_rad = angle.to_radians();
        let dx = angle_rad.cos();
        let dy = angle_rad.sin();
        
        // Number of samples based on amount
        let samples = (amount * 20.0).max(1.0) as i32;
        
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0.0f32;
                let mut g_sum = 0.0f32;
                let mut b_sum = 0.0f32;
                let mut count = 0.0f32;
                
                for i in -samples..=samples {
                    let sx = (x as f32 + dx * i as f32) as i32;
                    let sy = (y as f32 + dy * i as f32) as i32;
                    
                    if sx >= 0 && sx < width && sy >= 0 && sy < height {
                        let idx = ((sy * width + sx) * 4) as usize;
                        r_sum += img.pixels[idx] as f32;
                        g_sum += img.pixels[idx + 1] as f32;
                        b_sum += img.pixels[idx + 2] as f32;
                        count += 1.0;
                    }
                }
                
                let idx = ((y * width + x) * 4) as usize;
                output[idx] = (r_sum / count) as u8;
                output[idx + 1] = (g_sum / count) as u8;
                output[idx + 2] = (b_sum / count) as u8;
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply progressive (gradient) blur
    fn apply_progressive_blur(&self, img: &ImageData, amount: f32, direction: &crate::nodes::BlurDirection, falloff: f32) -> ImageData {
        use crate::nodes::BlurDirection;
        
        let width = img.width;
        let height = img.height;
        let mut output = img.pixels.as_ref().clone();
        
        // Pre-compute blur levels
        let max_radius = (amount * 25.0) as u32;
        
        for y in 0..height {
            for x in 0..width {
                // Calculate blur factor based on position and direction
                let factor = match direction {
                    BlurDirection::Top => 1.0 - (y as f32 / height as f32),
                    BlurDirection::Bottom => y as f32 / height as f32,
                    BlurDirection::Left => 1.0 - (x as f32 / width as f32),
                    BlurDirection::Right => x as f32 / width as f32,
                };
                
                // Apply falloff curve
                let blur_factor = (factor / falloff.max(0.01)).min(1.0).powf(2.0);
                let radius = (blur_factor * max_radius as f32) as i32;
                
                if radius > 0 {
                    let mut r_sum = 0u32;
                    let mut g_sum = 0u32;
                    let mut b_sum = 0u32;
                    let mut count = 0u32;
                    
                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            let sx = (x as i32 + dx).clamp(0, width as i32 - 1) as u32;
                            let sy = (y as i32 + dy).clamp(0, height as i32 - 1) as u32;
                            let idx = ((sy * width + sx) * 4) as usize;
                            r_sum += img.pixels[idx] as u32;
                            g_sum += img.pixels[idx + 1] as u32;
                            b_sum += img.pixels[idx + 2] as u32;
                            count += 1;
                        }
                    }
                    
                    let idx = ((y * width + x) * 4) as usize;
                    output[idx] = (r_sum / count) as u8;
                    output[idx + 1] = (g_sum / count) as u8;
                    output[idx + 2] = (b_sum / count) as u8;
                }
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply glass blinds effect (wave distortion)
    fn apply_glass_blinds(&self, img: &ImageData, intensity: f32, frequency: f32, angle: f32, phase: f32) -> ImageData {
        let mut output = img.pixels.as_ref().clone();
        let width = img.width as f32;
        let height = img.height as f32;
        
        let angle_rad = angle.to_radians();
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        
        for y in 0..img.height {
            for x in 0..img.width {
                // Rotate coordinates
                let rx = x as f32 * cos_a + y as f32 * sin_a;
                
                // Calculate wave displacement
                let wave = ((rx * frequency / 100.0 + phase * std::f32::consts::PI * 2.0).sin() * intensity * 20.0) as i32;
                
                // Calculate source coordinates
                let sx = (x as i32 - (wave as f32 * sin_a) as i32).clamp(0, img.width as i32 - 1) as u32;
                let sy = (y as i32 + (wave as f32 * cos_a) as i32).clamp(0, img.height as i32 - 1) as u32;
                
                let src_idx = ((sy * img.width + sx) * 4) as usize;
                let dst_idx = ((y * img.width + x) * 4) as usize;
                
                output[dst_idx] = img.pixels[src_idx];
                output[dst_idx + 1] = img.pixels[src_idx + 1];
                output[dst_idx + 2] = img.pixels[src_idx + 2];
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
    
    /// Apply film grain with size control
    fn apply_grain_advanced(&self, img: &ImageData, amount: f32, size: f32, monochrome: bool, seed: u32) -> ImageData {
        let mut output = img.pixels.as_ref().clone();
        let width = img.width;
        let height = img.height;
        
        // Use seed for reproducible noise
        let mut rng_state = seed.wrapping_add(12345);
        
        // Size determines the grain "block" size
        let block_size = size.max(1.0) as u32;
        
        for by in (0..height).step_by(block_size as usize) {
            for bx in (0..width).step_by(block_size as usize) {
                // Generate noise for this block
                rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                let noise_val = ((rng_state >> 16) & 0xFF) as f32 / 255.0 - 0.5;
                let noise = noise_val * 2.0 * amount * 50.0;
                
                // Color noise if not monochrome
                let (nr, ng, nb) = if monochrome {
                    (noise, noise, noise)
                } else {
                    rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                    let nr = ((rng_state >> 16) & 0xFF) as f32 / 255.0 - 0.5;
                    rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                    let ng = ((rng_state >> 16) & 0xFF) as f32 / 255.0 - 0.5;
                    rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                    let nb = ((rng_state >> 16) & 0xFF) as f32 / 255.0 - 0.5;
                    (nr * 2.0 * amount * 50.0, ng * 2.0 * amount * 50.0, nb * 2.0 * amount * 50.0)
                };
                
                // Apply to all pixels in block
                for dy in 0..block_size {
                    for dx in 0..block_size {
                        let x = bx + dx;
                        let y = by + dy;
                        if x < width && y < height {
                            let idx = ((y * width + x) * 4) as usize;
                            output[idx] = (output[idx] as f32 + nr).clamp(0.0, 255.0) as u8;
                            output[idx + 1] = (output[idx + 1] as f32 + ng).clamp(0.0, 255.0) as u8;
                            output[idx + 2] = (output[idx + 2] as f32 + nb).clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
        }
        
        ImageData::new(output, img.width, img.height)
    }
}
