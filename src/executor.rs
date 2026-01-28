//! Node graph execution engine
//!
//! Executes the node graph in topological order, processing images through connected nodes.

use std::collections::HashMap;
use uuid::Uuid;

use crate::graph::{NodeGraph, Connection};
use crate::nodes::{NodeType, NodeProperties};
use crate::image_data::ImageData;
use crate::gpu::{cpu, BrightnessContrastParams, HueSaturationParams};

/// Result of executing a node
#[derive(Clone)]
pub enum NodeOutput {
    /// Image output
    Image(ImageData),
    /// Color output [R, G, B, A] in 0-1 range
    Color([f32; 4]),
    /// Number output
    Number(f32),
    /// No output (e.g., output node)
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
    
    /// Execute the entire graph and return the output node's result
    pub fn execute(&mut self, graph: &NodeGraph, input_images: &HashMap<u64, ImageData>) -> Result<Option<ImageData>, String> {
        // Clear previous outputs
        self.outputs.clear();
        
        // Get topological order
        let order = self.topological_sort(graph)?;
        
        // Execute each node in order
        for node_id in order {
            self.execute_node(graph, node_id, input_images)?;
        }
        
        // Find the output node and return its input
        for (id, node) in graph.nodes_iter() {
            if matches!(node.node_type, NodeType::Output) {
                // Get the input to the output node
                if let Some(conn) = graph.connections_iter().find(|c| c.to_node == *id) {
                    if let Some(NodeOutput::Image(img)) = self.outputs.get(&conn.from_node) {
                        return Ok(Some(img.clone()));
                    }
                }
            }
        }
        
        Ok(None)
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
            NodeProperties::ImageInput { texture_id, .. } => {
                // Get image from input_images map
                if let Some(id) = texture_id {
                    if let Some(img) = input_images.get(id) {
                        NodeOutput::Image(img.clone())
                    } else {
                        NodeOutput::None
                    }
                } else {
                    NodeOutput::None
                }
            }
            
            NodeProperties::Color { color } => {
                NodeOutput::Color(*color)
            }
            
            NodeProperties::Number { value, .. } => {
                NodeOutput::Number(*value)
            }
            
            NodeProperties::BrightnessContrast { brightness, contrast } => {
                // Get input image
                let input = self.get_input_image(graph, node_id)?;
                if let Some(img) = input {
                    let params = BrightnessContrastParams {
                        brightness: *brightness,
                        contrast: *contrast,
                    };
                    let result = cpu::brightness_contrast(&img, params);
                    NodeOutput::Image(result)
                } else {
                    NodeOutput::None
                }
            }
            
            NodeProperties::HueSaturation { hue, saturation, lightness } => {
                // Get input image
                let input = self.get_input_image(graph, node_id)?;
                if let Some(img) = input {
                    let params = HueSaturationParams {
                        hue_shift: *hue,
                        saturation: *saturation,
                        lightness: *lightness,
                    };
                    // Use GPU context's CPU implementation for now
                    let mut output = img.pixels.as_ref().clone();
                    
                    for chunk in output.chunks_exact_mut(4) {
                        let r = chunk[0] as f32 / 255.0;
                        let g = chunk[1] as f32 / 255.0;
                        let b = chunk[2] as f32 / 255.0;
                        
                        // Simple saturation adjustment
                        let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                        let r = gray + (r - gray) * (1.0 + *saturation);
                        let g = gray + (g - gray) * (1.0 + *saturation);
                        let b = gray + (b - gray) * (1.0 + *saturation);
                        
                        // Lightness adjustment
                        let r = (r + *lightness).clamp(0.0, 1.0);
                        let g = (g + *lightness).clamp(0.0, 1.0);
                        let b = (b + *lightness).clamp(0.0, 1.0);
                        
                        chunk[0] = (r * 255.0) as u8;
                        chunk[1] = (g * 255.0) as u8;
                        chunk[2] = (b * 255.0) as u8;
                    }
                    
                    NodeOutput::Image(ImageData::new(output, img.width, img.height))
                } else {
                    NodeOutput::None
                }
            }
            
            NodeProperties::Blur { radius, .. } => {
                let input = self.get_input_image(graph, node_id)?;
                if let Some(img) = input {
                    // Simple box blur for now
                    let result = self.apply_blur(&img, *radius as u32);
                    NodeOutput::Image(result)
                } else {
                    NodeOutput::None
                }
            }
            
            NodeProperties::Output {} => {
                // Output node just passes through
                let input = self.get_input_image(graph, node_id)?;
                if let Some(img) = input {
                    NodeOutput::Image(img)
                } else {
                    NodeOutput::None
                }
            }
            
            // Other node types pass through or do nothing for now
            _ => NodeOutput::None,
        };
        
        self.outputs.insert(node_id, output);
        Ok(())
    }
    
    /// Get the image input for a node (from first connected input)
    fn get_input_image(&self, graph: &NodeGraph, node_id: Uuid) -> Result<Option<ImageData>, String> {
        // Find connection to this node's first input
        for conn in graph.connections_iter() {
            if conn.to_node == node_id && conn.to_slot == 0 {
                if let Some(NodeOutput::Image(img)) = self.outputs.get(&conn.from_node) {
                    return Ok(Some(img.clone()));
                }
            }
        }
        Ok(None)
    }
    
    /// Simple box blur implementation
    fn apply_blur(&self, input: &ImageData, radius: u32) -> ImageData {
        if radius == 0 {
            return input.clone();
        }
        
        let radius = radius.min(20); // Limit for performance
        let width = input.width;
        let height = input.height;
        let mut output = vec![0u8; input.pixels.len()];
        
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0u32;
                
                let x_start = x.saturating_sub(radius);
                let x_end = (x + radius + 1).min(width);
                let y_start = y.saturating_sub(radius);
                let y_end = (y + radius + 1).min(height);
                
                for sy in y_start..y_end {
                    for sx in x_start..x_end {
                        let pixel = input.get_pixel(sx, sy);
                        r_sum += pixel[0] as u32;
                        g_sum += pixel[1] as u32;
                        b_sum += pixel[2] as u32;
                        a_sum += pixel[3] as u32;
                        count += 1;
                    }
                }
                
                let idx = ((y * width + x) * 4) as usize;
                output[idx] = (r_sum / count) as u8;
                output[idx + 1] = (g_sum / count) as u8;
                output[idx + 2] = (b_sum / count) as u8;
                output[idx + 3] = (a_sum / count) as u8;
            }
        }
        
        ImageData::new(output, width, height)
    }
    
    /// Topological sort of nodes (Kahn's algorithm)
    fn topological_sort(&self, graph: &NodeGraph) -> Result<Vec<Uuid>, String> {
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        // Initialize
        for (id, _) in graph.nodes_iter() {
            in_degree.insert(*id, 0);
            adjacency.insert(*id, Vec::new());
        }
        
        // Build graph
        for conn in graph.connections_iter() {
            adjacency.get_mut(&conn.from_node)
                .map(|v| v.push(conn.to_node));
            *in_degree.get_mut(&conn.to_node).unwrap_or(&mut 0) += 1;
        }
        
        // Find nodes with no incoming edges
        let mut queue: Vec<Uuid> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| *id)
            .collect();
        
        let mut result = Vec::new();
        
        while let Some(node) = queue.pop() {
            result.push(node);
            
            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(*neighbor);
                        }
                    }
                }
            }
        }
        
        if result.len() != in_degree.len() {
            return Err("Cycle detected in node graph".to_string());
        }
        
        Ok(result)
    }
}
