//! Node types and their definitions

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All available node types in FlowNode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // Input
    ImageInput,
    Color,
    Number,
    
    // Adjustments
    BrightnessContrast,
    HueSaturation,
    Levels,
    
    // Filters
    Blur,
    Sharpen,
    Noise,
    Invert,
    Grayscale,
    
    // Combine
    Blend,
    Mask,
    
    // Output
    Output,
}

impl NodeType {
    /// Get the display name for this node type
    pub fn name(&self) -> &'static str {
        match self {
            Self::ImageInput => "Image Input",
            Self::Color => "Color",
            Self::Number => "Number",
            Self::BrightnessContrast => "Brightness/Contrast",
            Self::HueSaturation => "Hue/Saturation",
            Self::Levels => "Levels",
            Self::Blur => "Blur",
            Self::Sharpen => "Sharpen",
            Self::Noise => "Noise",
            Self::Invert => "Invert",
            Self::Grayscale => "Grayscale",
            Self::Blend => "Blend",
            Self::Mask => "Mask",
            Self::Output => "Output",
        }
    }
    
    /// Get the category color for this node type
    pub fn color(&self) -> egui::Color32 {
        use egui::Color32;
        match self {
            Self::ImageInput | Self::Color | Self::Number => Color32::from_rgb(76, 175, 80), // Green
            Self::BrightnessContrast | Self::HueSaturation | Self::Levels => Color32::from_rgb(255, 152, 0), // Orange
            Self::Blur | Self::Sharpen | Self::Noise | Self::Invert | Self::Grayscale => Color32::from_rgb(33, 150, 243), // Blue
            Self::Blend | Self::Mask => Color32::from_rgb(156, 39, 176), // Purple
            Self::Output => Color32::from_rgb(244, 67, 54), // Red
        }
    }
    
    /// Get the input slots for this node type
    pub fn inputs(&self) -> Vec<SlotInfo> {
        match self {
            Self::ImageInput => vec![],
            Self::Color => vec![],
            Self::Number => vec![],
            
            Self::BrightnessContrast => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            Self::HueSaturation => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            Self::Levels => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            
            Self::Blur => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            Self::Sharpen => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            Self::Noise => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            Self::Invert => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            Self::Grayscale => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
            
            Self::Blend => vec![
                SlotInfo::new("Base", SlotType::Image),
                SlotInfo::new("Blend", SlotType::Image),
            ],
            Self::Mask => vec![
                SlotInfo::new("Image", SlotType::Image),
                SlotInfo::new("Mask", SlotType::Image),
            ],
            
            Self::Output => vec![
                SlotInfo::new("Image", SlotType::Image),
            ],
        }
    }
    
    /// Get the output slots for this node type
    pub fn outputs(&self) -> Vec<SlotInfo> {
        match self {
            Self::ImageInput => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::Color => vec![SlotInfo::new("Color", SlotType::Color)],
            Self::Number => vec![SlotInfo::new("Value", SlotType::Number)],
            
            Self::BrightnessContrast => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::HueSaturation => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::Levels => vec![SlotInfo::new("Image", SlotType::Image)],
            
            Self::Blur => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::Sharpen => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::Noise => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::Invert => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::Grayscale => vec![SlotInfo::new("Image", SlotType::Image)],
            
            Self::Blend => vec![SlotInfo::new("Image", SlotType::Image)],
            Self::Mask => vec![SlotInfo::new("Image", SlotType::Image)],
            
            Self::Output => vec![], // Output has no outputs
        }
    }
}

/// Type of data a slot can carry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    Image,
    Color,
    Number,
}

impl SlotType {
    pub fn color(&self) -> egui::Color32 {
        use egui::Color32;
        match self {
            Self::Image => Color32::from_rgb(255, 193, 7),  // Amber
            Self::Color => Color32::from_rgb(233, 30, 99),  // Pink
            Self::Number => Color32::from_rgb(0, 188, 212), // Cyan
        }
    }
}

/// Information about a slot (input or output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub name: String,
    pub slot_type: SlotType,
}

impl SlotInfo {
    pub fn new(name: &str, slot_type: SlotType) -> Self {
        Self {
            name: name.to_string(),
            slot_type,
        }
    }
}

/// A node instance in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub node_type: NodeType,
    pub position: egui::Pos2,
    pub properties: NodeProperties,
}

impl Node {
    pub fn new(node_type: NodeType, position: egui::Pos2) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type,
            position,
            properties: NodeProperties::for_type(node_type),
        }
    }
}

/// Properties specific to each node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeProperties {
    ImageInput {
        file_path: Option<String>,
        #[serde(skip)]
        texture_id: Option<u64>, // Runtime texture cache ID
    },
    Color {
        color: [f32; 4],
    },
    Number {
        value: f32,
        min: f32,
        max: f32,
    },
    BrightnessContrast {
        brightness: f32,
        contrast: f32,
    },
    HueSaturation {
        hue: f32,
        saturation: f32,
        lightness: f32,
    },
    Levels {
        black_point: f32,
        white_point: f32,
        gamma: f32,
    },
    Blur {
        radius: f32,
        blur_type: BlurType,
    },
    Sharpen {
        amount: f32,
        radius: f32,
    },
    Noise {
        amount: f32,
        monochrome: bool,
    },
    Invert {},
    Grayscale {},
    Blend {
        mode: BlendMode,
        opacity: f32,
    },
    Mask {
        invert: bool,
    },
    Output {},
}

impl NodeProperties {
    pub fn for_type(node_type: NodeType) -> Self {
        match node_type {
            NodeType::ImageInput => Self::ImageInput { file_path: None, texture_id: None },
            NodeType::Color => Self::Color { color: [1.0, 1.0, 1.0, 1.0] },
            NodeType::Number => Self::Number { value: 0.0, min: 0.0, max: 1.0 },
            NodeType::BrightnessContrast => Self::BrightnessContrast { brightness: 0.0, contrast: 0.0 },
            NodeType::HueSaturation => Self::HueSaturation { hue: 0.0, saturation: 0.0, lightness: 0.0 },
            NodeType::Levels => Self::Levels { black_point: 0.0, white_point: 1.0, gamma: 1.0 },
            NodeType::Blur => Self::Blur { radius: 5.0, blur_type: BlurType::Gaussian },
            NodeType::Sharpen => Self::Sharpen { amount: 1.0, radius: 1.0 },
            NodeType::Noise => Self::Noise { amount: 0.1, monochrome: false },
            NodeType::Invert => Self::Invert {},
            NodeType::Grayscale => Self::Grayscale {},
            NodeType::Blend => Self::Blend { mode: BlendMode::Normal, opacity: 1.0 },
            NodeType::Mask => Self::Mask { invert: false },
            NodeType::Output => Self::Output {},
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlurType {
    Gaussian,
    Box,
    Motion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
}

use eframe::egui;
