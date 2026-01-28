//! Compatibility layer for FlowNode.io React Flow format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use eframe::egui::{Vec2, Pos2};

use crate::nodes::{Node, NodeType, NodeProperties, BlurType, BlendMode};
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

/// React Flow position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// React Flow edge (connection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "sourceHandle", skip_serializing_if = "Option::is_none")]
    pub source_handle: Option<String>,
    #[serde(rename = "targetHandle", skip_serializing_if = "Option::is_none")]
    pub target_handle: Option<String>,
}

/// React Flow viewport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

/// Full React Flow compatible workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactFlowWorkflow {
    pub nodes: Vec<ReactFlowNode>,
    pub edges: Vec<ReactFlowEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
}

impl ReactFlowWorkflow {
    /// Convert from internal NodeGraph format
    pub fn from_graph(graph: &NodeGraph, pan: Vec2, zoom: f32) -> Self {
        let nodes: Vec<ReactFlowNode> = graph.nodes_iter()
            .map(|(id, node)| {
                let data = node_to_data(node);
                ReactFlowNode {
                    id: id.to_string(),
                    node_type: node_type_to_string(&node.node_type),
                    position: Position {
                        x: node.position.x,
                        y: node.position.y,
                    },
                    data,
                }
            })
            .collect();
        
        let edges: Vec<ReactFlowEdge> = graph.connections_iter()
            .enumerate()
            .map(|(i, conn)| {
                ReactFlowEdge {
                    id: format!("edge-{}", i),
                    source: conn.from_node.to_string(),
                    target: conn.to_node.to_string(),
                    source_handle: Some(format!("output-{}", conn.from_slot)),
                    target_handle: Some(format!("input-{}", conn.to_slot)),
                }
            })
            .collect();
        
        ReactFlowWorkflow {
            nodes,
            edges,
            viewport: Some(Viewport {
                x: pan.x,
                y: pan.y,
                zoom,
            }),
        }
    }
    
    /// Convert to internal NodeGraph format
    pub fn to_graph(&self) -> Result<(NodeGraph, Vec2, f32), String> {
        let mut nodes = HashMap::new();
        
        for rf_node in &self.nodes {
            let id = Uuid::parse_str(&rf_node.id)
                .map_err(|e| format!("Invalid node ID: {}", e))?;
            
            let node_type = string_to_node_type(&rf_node.node_type)?;
            let properties = data_to_properties(&rf_node.data, &node_type);
            
            let node = Node {
                id,
                node_type,
                position: Pos2::new(rf_node.position.x, rf_node.position.y),
                properties,
            };
            
            nodes.insert(id, node);
        }
        
        let mut connections = Vec::new();
        for rf_edge in &self.edges {
            let from_node = Uuid::parse_str(&rf_edge.source)
                .map_err(|e| format!("Invalid source ID: {}", e))?;
            let to_node = Uuid::parse_str(&rf_edge.target)
                .map_err(|e| format!("Invalid target ID: {}", e))?;
            
            // Parse handle indices
            let from_slot = rf_edge.source_handle
                .as_ref()
                .and_then(|h| h.strip_prefix("output-"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            let to_slot = rf_edge.target_handle
                .as_ref()
                .and_then(|h| h.strip_prefix("input-"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            connections.push(Connection {
                from_node,
                from_slot,
                to_node,
                to_slot,
            });
        }
        
        let (pan, zoom) = self.viewport
            .as_ref()
            .map(|v| (Vec2::new(v.x, v.y), v.zoom))
            .unwrap_or((Vec2::ZERO, 1.0));
        
        let graph = NodeGraph::from_parts(nodes, connections);
        
        Ok((graph, pan, zoom))
    }
}

/// Convert node type enum to React Flow string
fn node_type_to_string(node_type: &NodeType) -> String {
    match node_type {
        NodeType::ImageInput => "image".to_string(),
        NodeType::Color => "color".to_string(),
        NodeType::Number => "number".to_string(),
        NodeType::BrightnessContrast => "image".to_string(), // Processing node
        NodeType::HueSaturation => "image".to_string(),
        NodeType::Levels => "image".to_string(),
        NodeType::Blur => "image".to_string(),
        NodeType::Sharpen => "image".to_string(),
        NodeType::Noise => "image".to_string(),
        NodeType::Blend => "image".to_string(),
        NodeType::Mask => "image".to_string(),
        NodeType::Output => "image".to_string(),
    }
}

/// Convert React Flow type string to node type enum
fn string_to_node_type(type_str: &str) -> Result<NodeType, String> {
    match type_str {
        "image" | "content" => Ok(NodeType::ImageInput), // Will be refined based on data
        "color" => Ok(NodeType::Color),
        "number" => Ok(NodeType::Number),
        _ => Ok(NodeType::ImageInput), // Default fallback
    }
}

/// Convert node to JSON data
fn node_to_data(node: &Node) -> serde_json::Value {
    let mut data = serde_json::Map::new();
    data.insert("label".to_string(), serde_json::Value::String(node.node_type.name().to_string()));
    
    // Add processing info based on properties enum variant
    match &node.properties {
        NodeProperties::BrightnessContrast { brightness, contrast } => {
            let mut processing = serde_json::Map::new();
            processing.insert("type".to_string(), "brightness_contrast".into());
            processing.insert("brightness".to_string(), (*brightness).into());
            processing.insert("contrast".to_string(), (*contrast).into());
            data.insert("processing".to_string(), serde_json::Value::Object(processing));
        }
        NodeProperties::HueSaturation { hue, saturation, lightness } => {
            let mut processing = serde_json::Map::new();
            processing.insert("type".to_string(), "hue_saturation".into());
            processing.insert("hue".to_string(), (*hue).into());
            processing.insert("saturation".to_string(), (*saturation).into());
            processing.insert("lightness".to_string(), (*lightness).into());
            data.insert("processing".to_string(), serde_json::Value::Object(processing));
        }
        NodeProperties::Blur { radius, blur_type } => {
            let mut processing = serde_json::Map::new();
            processing.insert("type".to_string(), "blur".into());
            processing.insert("radius".to_string(), (*radius).into());
            processing.insert("blur_type".to_string(), match blur_type {
                BlurType::Gaussian => "gaussian",
                BlurType::Box => "box",
                BlurType::Motion => "motion",
            }.into());
            data.insert("processing".to_string(), serde_json::Value::Object(processing));
        }
        NodeProperties::Color { color } => {
            data.insert("color".to_string(), serde_json::json!(color));
        }
        NodeProperties::Number { value, min, max } => {
            data.insert("value".to_string(), (*value).into());
            data.insert("min".to_string(), (*min).into());
            data.insert("max".to_string(), (*max).into());
        }
        _ => {}
    }
    
    serde_json::Value::Object(data)
}

/// Convert JSON data to node properties
fn data_to_properties(data: &serde_json::Value, node_type: &NodeType) -> NodeProperties {
    // Start with default for the type
    let mut props = NodeProperties::for_type(*node_type);
    
    // Override with data from JSON if present
    if let Some(processing) = data.get("processing") {
        match node_type {
            NodeType::BrightnessContrast => {
                let brightness = processing.get("brightness").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let contrast = processing.get("contrast").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                props = NodeProperties::BrightnessContrast { brightness, contrast };
            }
            NodeType::HueSaturation => {
                let hue = processing.get("hue").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let saturation = processing.get("saturation").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let lightness = processing.get("lightness").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                props = NodeProperties::HueSaturation { hue, saturation, lightness };
            }
            NodeType::Blur => {
                let radius = processing.get("radius").and_then(|v| v.as_f64()).unwrap_or(5.0) as f32;
                let blur_type_str = processing.get("blur_type").and_then(|v| v.as_str()).unwrap_or("gaussian");
                let blur_type = match blur_type_str {
                    "box" => BlurType::Box,
                    "motion" => BlurType::Motion,
                    _ => BlurType::Gaussian,
                };
                props = NodeProperties::Blur { radius, blur_type };
            }
            _ => {}
        }
    }
    
    props
}
