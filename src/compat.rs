//! Compatibility layer for FlowNode.io React Flow format
//! 
//! This module handles conversion between WASM internal format
//! and the React Flow JSON format used by FlowNode.io

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use eframe::egui::Vec2;

use crate::nodes::{Node, NodeType, NodeProperties, BlendMode, BlurDirection, ColorWheel};
use crate::graph::{NodeGraph, Connection};

/// React Flow compatible node format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub position: Position,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// React Flow compatible edge format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "sourceHandle")]
    pub source_handle: Option<String>,
    #[serde(rename = "targetHandle")]
    pub target_handle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

/// Complete React Flow workflow format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowWorkflow {
    pub nodes: Vec<ReactFlowNode>,
    pub edges: Vec<ReactFlowEdge>,
    pub viewport: Viewport,
}

impl ReactFlowWorkflow {
    /// Convert from internal NodeGraph format to React Flow format
    pub fn from_graph(graph: &NodeGraph, pan: Vec2, zoom: f32) -> Self {
        let nodes: Vec<ReactFlowNode> = graph.nodes_iter()
            .map(|(_, node)| {
                ReactFlowNode {
                    id: node.id.to_string(),
                    node_type: node_type_to_string(&node.node_type),
                    position: Position {
                        x: node.position.x,
                        y: node.position.y,
                    },
                    data: node_to_data(node),
                }
            })
            .collect();
        
        let edges: Vec<ReactFlowEdge> = graph.connections_iter()
            .map(|conn| {
                ReactFlowEdge {
                    id: format!("e-{}-{}-{}-{}", 
                        conn.from_node, conn.from_slot, 
                        conn.to_node, conn.to_slot),
                    source: conn.from_node.to_string(),
                    target: conn.to_node.to_string(),
                    source_handle: Some(format!("output-{}", conn.from_slot)),
                    target_handle: Some(format!("input-{}", conn.to_slot)),
                }
            })
            .collect();
        
        Self {
            nodes,
            edges,
            viewport: Viewport {
                x: pan.x,
                y: pan.y,
                zoom,
            },
        }
    }
    
    /// Convert React Flow format to internal NodeGraph
    pub fn to_graph(&self) -> Result<(NodeGraph, Vec2, f32), String> {
        let mut nodes: HashMap<Uuid, Node> = HashMap::new();
        let mut connections: Vec<Connection> = Vec::new();
        let mut id_map: HashMap<String, Uuid> = HashMap::new();
        
        // First pass: create nodes
        for rf_node in &self.nodes {
            let node_type = string_to_node_type(&rf_node.node_type)?;
            let uuid = Uuid::new_v4();
            id_map.insert(rf_node.id.clone(), uuid);
            
            let mut node = Node::new(node_type, Vec2::new(rf_node.position.x, rf_node.position.y));
            node.id = uuid;
            node.properties = data_to_properties(&rf_node.data, &node_type);
            
            nodes.insert(uuid, node);
        }
        
        // Second pass: create connections
        for edge in &self.edges {
            let from_id = id_map.get(&edge.source)
                .ok_or_else(|| format!("Unknown source node: {}", edge.source))?;
            let to_id = id_map.get(&edge.target)
                .ok_or_else(|| format!("Unknown target node: {}", edge.target))?;
            
            let from_slot = edge.source_handle
                .as_ref()
                .and_then(|h| h.strip_prefix("output-"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            let to_slot = edge.target_handle
                .as_ref()
                .and_then(|h| h.strip_prefix("input-"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            connections.push(Connection {
                from_node: *from_id,
                from_slot,
                to_node: *to_id,
                to_slot,
            });
        }
        
        let pan = Vec2::new(self.viewport.x, self.viewport.y);
        let zoom = self.viewport.zoom;
        
        Ok((NodeGraph::from_parts(nodes, connections), pan, zoom))
    }
}

/// Convert node type enum to React Flow string
fn node_type_to_string(node_type: &NodeType) -> String {
    match node_type {
        NodeType::Image => "image".to_string(),
        NodeType::Content => "content".to_string(),
        NodeType::Bucket => "bucket".to_string(),
        NodeType::Adjust => "adjust".to_string(),
        NodeType::Effects => "effects".to_string(),
        NodeType::Text => "text".to_string(),
        NodeType::Concat => "concat".to_string(),
        NodeType::Splitter => "splitter".to_string(),
        NodeType::Postit => "postit".to_string(),
        NodeType::Compare => "compare".to_string(),
        NodeType::Composition => "composition".to_string(),
        NodeType::Router => "router".to_string(),
        NodeType::Batch => "batch".to_string(),
        NodeType::Title => "title".to_string(),
        NodeType::Group => "group".to_string(),
        NodeType::Folder => "folder".to_string(),
        NodeType::Convertor => "convertor".to_string(),
        NodeType::Omni => "omni".to_string(),
        NodeType::Llm => "llm".to_string(),
        NodeType::Video => "video".to_string(),
        NodeType::Upscaler => "upscaler".to_string(),
        NodeType::Vector => "vector".to_string(),
        NodeType::Rodin3d => "rodin3d".to_string(),
        NodeType::MindMap => "mind-map".to_string(),
    }
}

/// Convert React Flow type string to node type enum
fn string_to_node_type(type_str: &str) -> Result<NodeType, String> {
    match type_str {
        "image" => Ok(NodeType::Image),
        "content" => Ok(NodeType::Content),
        "bucket" => Ok(NodeType::Bucket),
        "adjust" => Ok(NodeType::Adjust),
        "effects" => Ok(NodeType::Effects),
        "text" => Ok(NodeType::Text),
        "concat" => Ok(NodeType::Concat),
        "splitter" => Ok(NodeType::Splitter),
        "postit" => Ok(NodeType::Postit),
        "compare" => Ok(NodeType::Compare),
        "composition" => Ok(NodeType::Composition),
        "router" => Ok(NodeType::Router),
        "batch" => Ok(NodeType::Batch),
        "title" => Ok(NodeType::Title),
        "group" => Ok(NodeType::Group),
        "folder" => Ok(NodeType::Folder),
        "convertor" => Ok(NodeType::Convertor),
        "omni" => Ok(NodeType::Omni),
        "llm" => Ok(NodeType::Llm),
        "video" => Ok(NodeType::Video),
        "upscaler" => Ok(NodeType::Upscaler),
        "vector" => Ok(NodeType::Vector),
        "rodin3d" => Ok(NodeType::Rodin3d),
        "mind-map" => Ok(NodeType::MindMap),
        _ => Err(format!("Unknown node type: {}", type_str)),
    }
}

/// Convert internal node to React Flow data field
fn node_to_data(node: &Node) -> serde_json::Value {
    match &node.properties {
        NodeProperties::Image { image, .. } => {
            serde_json::json!({
                "image": image,
                "label": node.label.as_deref().unwrap_or("Image")
            })
        }
        
        NodeProperties::Adjust { 
            brightness, contrast, saturation, exposure,
            highlights, shadows, temperature, tint,
            vibrance, gamma, color_boost, hue_rotation, luminance_mix, ..
        } => {
            serde_json::json!({
                "label": "Adjust",
                "settings": {
                    "brightness": brightness,
                    "contrast": contrast,
                    "saturation": saturation,
                    "exposure": exposure,
                    "highlights": highlights,
                    "shadows": shadows,
                    "temperature": temperature,
                    "tint": tint,
                    "vibrance": vibrance,
                    "gamma": gamma,
                    "colorBoost": color_boost,
                    "hueRotation": hue_rotation,
                    "luminanceMix": luminance_mix
                }
            })
        }
        
        NodeProperties::Effects {
            gaussian_blur, directional_blur, directional_blur_angle,
            progressive_blur, progressive_blur_direction, progressive_blur_falloff,
            glass_blinds, glass_blinds_frequency, glass_blinds_angle, glass_blinds_phase,
            grain, grain_size, grain_monochrome, grain_seed,
            sharpen, vignette, vignette_roundness, vignette_smoothness
        } => {
            serde_json::json!({
                "label": "Effects",
                "settings": {
                    "gaussianBlur": gaussian_blur,
                    "directionalBlur": directional_blur,
                    "directionalBlurAngle": directional_blur_angle,
                    "progressiveBlur": progressive_blur,
                    "progressiveBlurDirection": format!("{:?}", progressive_blur_direction).to_lowercase(),
                    "progressiveBlurFalloff": progressive_blur_falloff,
                    "glassBlinds": glass_blinds,
                    "glassBlindsFrequency": glass_blinds_frequency,
                    "glassBlindsAngle": glass_blinds_angle,
                    "glassBlindsPhase": glass_blinds_phase,
                    "grain": grain,
                    "grainSize": grain_size,
                    "grainMonochrome": grain_monochrome,
                    "grainSeed": grain_seed,
                    "sharpen": sharpen,
                    "vignette": vignette,
                    "vignetteRoundness": vignette_roundness,
                    "vignetteSmoothness": vignette_smoothness
                }
            })
        }
        
        NodeProperties::Text { text } => {
            serde_json::json!({ "text": text })
        }
        
        NodeProperties::Omni { model, prompt, negative_prompt, seed } => {
            serde_json::json!({
                "model": model,
                "prompt": prompt,
                "negativePrompt": negative_prompt,
                "seed": seed
            })
        }
        
        // Default for other types
        _ => {
            serde_json::json!({
                "label": node.node_type.name()
            })
        }
    }
}

/// Convert React Flow data field to internal properties
fn data_to_properties(data: &serde_json::Value, node_type: &NodeType) -> NodeProperties {
    match node_type {
        NodeType::Image => {
            NodeProperties::Image {
                image: data.get("image").and_then(|v| v.as_str()).map(String::from),
                thumbnail: None,
                history: Vec::new(),
                texture_id: None,
            }
        }
        
        NodeType::Adjust => {
            let settings = data.get("settings").unwrap_or(data);
            NodeProperties::Adjust {
                brightness: settings.get("brightness").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                contrast: settings.get("contrast").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                saturation: settings.get("saturation").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                exposure: settings.get("exposure").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                highlights: settings.get("highlights").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                shadows: settings.get("shadows").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                temperature: settings.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                tint: settings.get("tint").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                vibrance: settings.get("vibrance").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                gamma: settings.get("gamma").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                lift: ColorWheel::default(),
                gamma_wheel: ColorWheel::default(),
                gain: ColorWheel::default(),
                offset: ColorWheel::default(),
                color_boost: settings.get("colorBoost").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                hue_rotation: settings.get("hueRotation").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                luminance_mix: settings.get("luminanceMix").and_then(|v| v.as_f64()).unwrap_or(100.0) as f32,
                curves_enabled: false,
            }
        }
        
        NodeType::Effects => {
            let settings = data.get("settings").unwrap_or(data);
            NodeProperties::Effects {
                gaussian_blur: settings.get("gaussianBlur").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                directional_blur: settings.get("directionalBlur").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                directional_blur_angle: settings.get("directionalBlurAngle").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                progressive_blur: settings.get("progressiveBlur").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                progressive_blur_direction: BlurDirection::Bottom, // TODO: Parse from string
                progressive_blur_falloff: settings.get("progressiveBlurFalloff").and_then(|v| v.as_f64()).unwrap_or(50.0) as f32,
                glass_blinds: settings.get("glassBlinds").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                glass_blinds_frequency: settings.get("glassBlindsFrequency").and_then(|v| v.as_f64()).unwrap_or(10.0) as f32,
                glass_blinds_angle: settings.get("glassBlindsAngle").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                glass_blinds_phase: settings.get("glassBlindsPhase").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                grain: settings.get("grain").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                grain_size: settings.get("grainSize").and_then(|v| v.as_f64()).unwrap_or(2.0) as f32,
                grain_monochrome: settings.get("grainMonochrome").and_then(|v| v.as_bool()).unwrap_or(true),
                grain_seed: settings.get("grainSeed").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                sharpen: settings.get("sharpen").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                vignette: settings.get("vignette").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                vignette_roundness: settings.get("vignetteRoundness").and_then(|v| v.as_f64()).unwrap_or(50.0) as f32,
                vignette_smoothness: settings.get("vignetteSmoothness").and_then(|v| v.as_f64()).unwrap_or(50.0) as f32,
            }
        }
        
        NodeType::Text => {
            NodeProperties::Text {
                text: data.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            }
        }
        
        // Default: use for_type
        _ => NodeProperties::for_type(*node_type),
    }
}
