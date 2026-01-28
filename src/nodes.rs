//! Node types and their definitions
//! Matches FlowNode React specification exactly.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All available node types in FlowNode
/// Names match React app exactly for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // Content nodes
    Image,      // Image display with history
    Content,    // Universal content node
    Bucket,     // Multi-image container
    
    // Editing nodes (Phase 1 focus)
    Adjust,     // Full color grading (10 sliders + wheels + curves)
    Effects,    // FX (blur, grain, vignette, sharpen)
    
    // Text nodes
    Text,       // Text input
    Concat,     // Join text
    Splitter,   // Split text
    Postit,     // Sticky note
    
    // Utility nodes
    Compare,    // Side-by-side comparison
    Composition,// Layer-based editor
    Router,     // Signal routing
    Batch,      // Batch processing
    Title,      // Labels
    Group,      // Group container
    Folder,     // Folder container
    Convertor,  // Format conversion
    
    // AI Generation nodes (Phase 3)
    Omni,       // Multi-model image gen
    Llm,        // Text generation
    Video,      // Video generation
    Upscaler,   // Image upscaling
    Vector,     // SVG conversion
    Rodin3d,    // 3D generation
    MindMap,    // AI mind mapping
}

impl NodeType {
    /// Get the display name for this node type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Content => "Content",
            Self::Bucket => "Bucket",
            Self::Adjust => "Adjust",
            Self::Effects => "Effects",
            Self::Text => "Text",
            Self::Concat => "Concat",
            Self::Splitter => "Splitter",
            Self::Postit => "Post-It",
            Self::Compare => "Compare",
            Self::Composition => "Composition",
            Self::Router => "Router",
            Self::Batch => "Batch",
            Self::Title => "Title",
            Self::Group => "Group",
            Self::Folder => "Folder",
            Self::Convertor => "Convertor",
            Self::Omni => "Omni",
            Self::Llm => "LLM",
            Self::Video => "Video",
            Self::Upscaler => "Upscaler",
            Self::Vector => "Vector",
            Self::Rodin3d => "3D",
            Self::MindMap => "Mind Map",
        }
    }
    
    /// Get the keyboard shortcut for this node type
    pub fn shortcut(&self) -> Option<char> {
        match self {
            Self::Text => Some('T'),
            Self::Postit => Some('N'),
            Self::Image => Some('I'),
            Self::Bucket => Some('B'),
            Self::Concat => Some('J'),
            Self::Splitter => Some('S'),
            Self::Compare => Some('C'),
            Self::Composition => Some('F'),
            Self::Omni => Some('O'),
            Self::Llm => Some('L'),
            Self::Upscaler => Some('U'),
            Self::Vector => Some('V'),
            Self::Rodin3d => Some('3'),
            Self::Title => Some('H'),
            Self::MindMap => Some('M'),
            Self::Content => Some('K'),
            Self::Video => Some('D'),
            Self::Batch => Some('Q'),
            Self::Router => Some('R'),
            Self::Adjust => Some('A'),
            Self::Effects => Some('E'),
            _ => None,
        }
    }
    
    /// Get the category color for this node type
    pub fn color(&self) -> egui::Color32 {
        use egui::Color32;
        match self {
            // Content - Green
            Self::Image | Self::Content | Self::Bucket => 
                Color32::from_rgb(76, 175, 80),
            
            // Editing - Orange
            Self::Adjust | Self::Effects | Self::Composition | Self::Compare => 
                Color32::from_rgb(255, 152, 0),
            
            // Text - Cyan
            Self::Text | Self::Concat | Self::Splitter | Self::Postit => 
                Color32::from_rgb(0, 188, 212),
            
            // AI Generation - Purple
            Self::Omni | Self::Llm | Self::Video | Self::Upscaler | 
            Self::Vector | Self::Rodin3d | Self::MindMap => 
                Color32::from_rgb(156, 39, 176),
            
            // Utility - Blue
            Self::Router | Self::Batch | Self::Title | Self::Group | 
            Self::Folder | Self::Convertor => 
                Color32::from_rgb(33, 150, 243),
        }
    }
    
    /// Get the input slots for this node type
    pub fn inputs(&self) -> Vec<SlotInfo> {
        match self {
            // No inputs
            Self::Image | Self::Text | Self::Postit | Self::Title | 
            Self::Batch | Self::Group | Self::Folder => vec![],
            
            // Single content input
            Self::Adjust | Self::Effects | Self::Upscaler | Self::Vector |
            Self::Convertor | Self::Content => vec![
                SlotInfo::new("content-in", SlotType::Content),
            ],
            
            // Multiple inputs
            Self::Compare => vec![
                SlotInfo::new("content-in-1", SlotType::Content),
                SlotInfo::new("content-in-2", SlotType::Content),
            ],
            Self::Composition => vec![
                SlotInfo::new("content-in", SlotType::Content), // Up to 10
            ],
            Self::Bucket => vec![
                SlotInfo::new("content-in", SlotType::Content),
            ],
            
            // Text inputs
            Self::Concat => vec![
                SlotInfo::new("text-in-1", SlotType::Text),
                SlotInfo::new("text-in-2", SlotType::Text),
            ],
            Self::Splitter => vec![
                SlotInfo::new("text-in", SlotType::Text),
            ],
            
            // AI nodes
            Self::Omni => vec![
                SlotInfo::new("prompts-in", SlotType::Text),
                SlotInfo::new("images-in", SlotType::Content),
            ],
            Self::Llm => vec![
                SlotInfo::new("prompts-in", SlotType::Text),
            ],
            Self::Video => vec![
                SlotInfo::new("prompts-in", SlotType::Text),
                SlotInfo::new("images-in", SlotType::Content),
            ],
            Self::Rodin3d => vec![
                SlotInfo::new("images-in", SlotType::Content),
            ],
            Self::MindMap => vec![
                SlotInfo::new("prompts-in", SlotType::Text),
            ],
            
            // Router
            Self::Router => vec![
                SlotInfo::new("router-in", SlotType::Content),
            ],
        }
    }
    
    /// Get the output slots for this node type
    pub fn outputs(&self) -> Vec<SlotInfo> {
        match self {
            // Content output
            Self::Image | Self::Adjust | Self::Effects | Self::Compare |
            Self::Composition | Self::Bucket | Self::Content | Self::Upscaler |
            Self::Vector | Self::Convertor | Self::Omni | Self::Video |
            Self::Rodin3d => vec![
                SlotInfo::new("content-out", SlotType::Content),
            ],
            
            // Text output
            Self::Text | Self::Concat | Self::Llm | Self::MindMap => vec![
                SlotInfo::new("text-out", SlotType::Text),
            ],
            
            // Multiple text outputs
            Self::Splitter => vec![
                SlotInfo::new("text-out", SlotType::Text), // Array of lines
            ],
            
            // Batch output
            Self::Batch => vec![
                SlotInfo::new("batch-out", SlotType::Batch),
            ],
            
            // Router outputs (multiple)
            Self::Router => vec![
                SlotInfo::new("router-out-1", SlotType::Content),
                SlotInfo::new("router-out-2", SlotType::Content),
                SlotInfo::new("router-out-3", SlotType::Content),
            ],
            
            // No outputs
            Self::Postit | Self::Title | Self::Group | Self::Folder => vec![],
        }
    }
}

/// Type of data a slot can carry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    Content,    // Images, videos, documents
    Text,       // Text data
    Batch,      // Batch of items
}

impl SlotType {
    pub fn color(&self) -> egui::Color32 {
        use egui::Color32;
        match self {
            Self::Content => Color32::from_rgb(255, 193, 7),  // Amber
            Self::Text => Color32::from_rgb(0, 188, 212),     // Cyan
            Self::Batch => Color32::from_rgb(156, 39, 176),   // Purple
        }
    }
}

/// Information about an input/output slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub name: &'static str,
    pub slot_type: SlotType,
}

impl SlotInfo {
    pub fn new(name: &'static str, slot_type: SlotType) -> Self {
        Self { name, slot_type }
    }
}

/// A node instance in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub node_type: NodeType,
    pub position: egui::Vec2,
    pub properties: NodeProperties,
    #[serde(default)]
    pub label: Option<String>,
}

impl Node {
    pub fn new(node_type: NodeType, position: egui::Vec2) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type,
            position,
            properties: NodeProperties::for_type(node_type),
            label: None,
        }
    }
}

/// Node-specific properties and settings
/// Matches React app data structures exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeProperties {
    // === Content Nodes ===
    Image {
        image: Option<String>,          // URL or base64
        thumbnail: Option<String>,
        history: Vec<String>,           // Up to 10 entries
        #[serde(skip)]
        texture_id: Option<u64>,
    },
    
    Content {
        content: Option<String>,
    },
    
    Bucket {
        images: Vec<String>,            // Multiple images
    },
    
    // === Editing Nodes (Phase 1 Focus) ===
    
    /// Full color grading - matches React ImageAdjustNode exactly
    Adjust {
        // Basic adjustments (-100 to 100)
        brightness: f32,
        contrast: f32,
        saturation: f32,
        exposure: f32,
        highlights: f32,
        shadows: f32,
        temperature: f32,   // Cool to warm
        tint: f32,          // Green to magenta
        vibrance: f32,
        gamma: f32,         // Maps to 0.1-3.0 internally
        
        // Color grading wheels
        lift: ColorWheel,   // Shadows
        gamma_wheel: ColorWheel,  // Midtones (renamed to avoid conflict)
        gain: ColorWheel,   // Highlights
        offset: ColorWheel, // Master
        
        // Additional
        color_boost: f32,       // -100 to 100
        hue_rotation: f32,      // -180 to 180
        luminance_mix: f32,     // 0 to 100
        
        // RGB Curves (simplified for now)
        curves_enabled: bool,
    },
    
    /// Effects - matches React EffectsNode exactly
    Effects {
        // Gaussian blur
        gaussian_blur: f32,             // 0-100
        
        // Directional blur
        directional_blur: f32,          // 0-100
        directional_blur_angle: f32,    // 0-360
        
        // Progressive blur
        progressive_blur: f32,          // 0-100
        progressive_blur_direction: BlurDirection,
        progressive_blur_falloff: f32,  // 0-100
        
        // Glass blinds
        glass_blinds: f32,              // 0-100
        glass_blinds_frequency: f32,    // 1-50
        glass_blinds_angle: f32,        // 0-360
        glass_blinds_phase: f32,        // 0-100
        
        // Grain
        grain: f32,                     // 0-100
        grain_size: f32,                // 1-10
        grain_monochrome: bool,
        grain_seed: u32,
        
        // Sharpen
        sharpen: f32,                   // 0-100
        
        // Vignette
        vignette: f32,                  // 0-100
        vignette_roundness: f32,        // 0-100
        vignette_smoothness: f32,       // 0-100
    },
    
    // === Text Nodes ===
    Text {
        text: String,
    },
    
    Concat {
        separator: String,
    },
    
    Splitter {
        delimiter: String,
    },
    
    Postit {
        text: String,
        color: [f32; 4],
    },
    
    // === Utility Nodes ===
    Compare {},
    
    Composition {
        layers: Vec<CompositionLayer>,
    },
    
    Router {
        active_output: usize,
    },
    
    Batch {
        items: Vec<String>,
    },
    
    Title {
        text: String,
    },
    
    Group {},
    Folder {},
    Convertor {},
    
    // === AI Generation Nodes (Phase 3) ===
    Omni {
        model: String,
        prompt: String,
        negative_prompt: String,
        seed: Option<u32>,
    },
    
    Llm {
        model: String,
        system_prompt: String,
    },
    
    Video {
        model: String,
        duration: u32,          // 4-10 seconds
        aspect_ratio: String,
    },
    
    Upscaler {
        model: String,
        scale: u32,
    },
    
    Vector {},
    Rodin3d {},
    MindMap {},
}

/// Color wheel for color grading (lift/gamma/gain/offset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorWheel {
    pub x: f32,         // -1 to 1 (hue position)
    pub y: f32,         // -1 to 1 (hue position)
    pub luminance: f32, // -100 to 100
}

impl Default for ColorWheel {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, luminance: 0.0 }
    }
}

/// Blur direction for progressive blur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlurDirection {
    Top,
    Bottom,
    Left,
    Right,
}

impl Default for BlurDirection {
    fn default() -> Self {
        Self::Bottom
    }
}

/// Layer in composition node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionLayer {
    pub image: String,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub position: (f32, f32),
    pub scale: f32,
}

/// Blend modes for composition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    HardLight,
    ColorDodge,
    ColorBurn,
    Difference,
    Exclusion,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl NodeProperties {
    pub fn for_type(node_type: NodeType) -> Self {
        match node_type {
            NodeType::Image => Self::Image {
                image: None,
                thumbnail: None,
                history: Vec::new(),
                texture_id: None,
            },
            NodeType::Content => Self::Content { content: None },
            NodeType::Bucket => Self::Bucket { images: Vec::new() },
            
            NodeType::Adjust => Self::Adjust {
                brightness: 0.0,
                contrast: 0.0,
                saturation: 0.0,
                exposure: 0.0,
                highlights: 0.0,
                shadows: 0.0,
                temperature: 0.0,
                tint: 0.0,
                vibrance: 0.0,
                gamma: 0.0,
                lift: ColorWheel::default(),
                gamma_wheel: ColorWheel::default(),
                gain: ColorWheel::default(),
                offset: ColorWheel::default(),
                color_boost: 0.0,
                hue_rotation: 0.0,
                luminance_mix: 100.0,
                curves_enabled: false,
            },
            
            NodeType::Effects => Self::Effects {
                gaussian_blur: 0.0,
                directional_blur: 0.0,
                directional_blur_angle: 0.0,
                progressive_blur: 0.0,
                progressive_blur_direction: BlurDirection::Bottom,
                progressive_blur_falloff: 50.0,
                glass_blinds: 0.0,
                glass_blinds_frequency: 10.0,
                glass_blinds_angle: 0.0,
                glass_blinds_phase: 0.0,
                grain: 0.0,
                grain_size: 2.0,
                grain_monochrome: true,
                grain_seed: 0,
                sharpen: 0.0,
                vignette: 0.0,
                vignette_roundness: 50.0,
                vignette_smoothness: 50.0,
            },
            
            NodeType::Text => Self::Text { text: String::new() },
            NodeType::Concat => Self::Concat { separator: String::new() },
            NodeType::Splitter => Self::Splitter { delimiter: "\n".to_string() },
            NodeType::Postit => Self::Postit { 
                text: String::new(), 
                color: [1.0, 0.95, 0.6, 1.0] // Yellow
            },
            
            NodeType::Compare => Self::Compare {},
            NodeType::Composition => Self::Composition { layers: Vec::new() },
            NodeType::Router => Self::Router { active_output: 0 },
            NodeType::Batch => Self::Batch { items: Vec::new() },
            NodeType::Title => Self::Title { text: "Title".to_string() },
            NodeType::Group => Self::Group {},
            NodeType::Folder => Self::Folder {},
            NodeType::Convertor => Self::Convertor {},
            
            NodeType::Omni => Self::Omni {
                model: "flux-1.1-pro".to_string(),
                prompt: String::new(),
                negative_prompt: String::new(),
                seed: None,
            },
            NodeType::Llm => Self::Llm {
                model: "claude-3-5-sonnet".to_string(),
                system_prompt: String::new(),
            },
            NodeType::Video => Self::Video {
                model: "veo-3.1-gemini".to_string(),
                duration: 4,
                aspect_ratio: "16:9".to_string(),
            },
            NodeType::Upscaler => Self::Upscaler {
                model: "freepik-precision-v2".to_string(),
                scale: 2,
            },
            NodeType::Vector => Self::Vector {},
            NodeType::Rodin3d => Self::Rodin3d {},
            NodeType::MindMap => Self::MindMap {},
        }
    }
}
