//! Node graph state and rendering

use eframe::egui::{self, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::nodes::{Node, NodeType, NodeProperties, SlotType, BlurType, BlendMode};

/// A connection between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: Uuid,
    pub from_slot: usize,
    pub to_node: Uuid,
    pub to_slot: usize,
}

/// The entire node graph
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeGraph {
    nodes: HashMap<Uuid, Node>,
    connections: Vec<Connection>,
    
    #[serde(skip)]
    selected_node: Option<Uuid>,
    
    #[serde(skip)]
    dragging_node: Option<Uuid>,
    
    #[serde(skip)]
    pan_offset: Vec2,
    
    #[serde(skip)]
    zoom: f32,
    
    #[serde(skip)]
    pending_connection: Option<PendingConnection>,
}

#[derive(Debug)]
struct PendingConnection {
    from_node: Uuid,
    from_slot: usize,
    is_output: bool,
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            selected_node: None,
            dragging_node: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            pending_connection: None,
        }
    }
    
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn selected_node(&self) -> Option<Uuid> {
        self.selected_node
    }
    
    pub fn add_node(&mut self, node_type: NodeType) {
        // Place new nodes in the center of the viewport with slight random offset
        let node_count = self.nodes.len() as f32;
        let offset_x = (node_count * 30.0) % 200.0;
        let offset_y = (node_count * 20.0) % 150.0;
        let position = Pos2::new(
            (300.0 + offset_x - self.pan_offset.x) / self.zoom,
            (200.0 + offset_y - self.pan_offset.y) / self.zoom,
        );
        let node = Node::new(node_type, position);
        self.selected_node = Some(node.id);
        self.nodes.insert(node.id, node);
    }
    
    /// Show the node graph in the UI
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );
        
        let canvas_rect = response.rect;
        
        // Handle panning (drag on empty space)
        if response.dragged() && self.dragging_node.is_none() && self.pending_connection.is_none() {
            self.pan_offset += response.drag_delta();
        }
        
        // Handle zoom (scroll wheel)
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            let zoom_factor = 1.0 + scroll * 0.001;
            self.zoom = (self.zoom * zoom_factor).clamp(0.25, 4.0);
        }
        
        // Fill canvas with dark background
        painter.rect_filled(canvas_rect, 0.0, egui::Color32::from_rgb(26, 26, 46));
        
        // Draw grid background
        self.draw_grid(&painter, canvas_rect);
        
        // Draw connections
        for conn in &self.connections {
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.get(&conn.from_node),
                self.nodes.get(&conn.to_node),
            ) {
                let from_pos = self.get_slot_position(from_node, conn.from_slot, false, canvas_rect);
                let to_pos = self.get_slot_position(to_node, conn.to_slot, true, canvas_rect);
                self.draw_connection(&painter, from_pos, to_pos, SlotType::Image);
            }
        }
        
        // Draw pending connection
        if let Some(pending) = &self.pending_connection {
            if let Some(node) = self.nodes.get(&pending.from_node) {
                let from_pos = self.get_slot_position(node, pending.from_slot, !pending.is_output, canvas_rect);
                let to_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(from_pos);
                self.draw_connection(&painter, from_pos, to_pos, SlotType::Image);
            }
        }
        
        // Draw nodes
        let node_ids: Vec<Uuid> = self.nodes.keys().copied().collect();
        for node_id in node_ids {
            self.draw_node(ui, &painter, canvas_rect, node_id);
        }
        
        // Handle click to deselect
        if response.clicked() && self.selected_node.is_some() {
            self.selected_node = None;
        }
        
        // Cancel pending connection on right click
        if response.secondary_clicked() {
            self.pending_connection = None;
        }
    }
    
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = 20.0 * self.zoom;
        let grid_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15);
        
        let start_x = (rect.left() + self.pan_offset.x % grid_size) as i32;
        let start_y = (rect.top() + self.pan_offset.y % grid_size) as i32;
        
        for x in (start_x..rect.right() as i32).step_by(grid_size as usize) {
            painter.line_segment(
                [Pos2::new(x as f32, rect.top()), Pos2::new(x as f32, rect.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        
        for y in (start_y..rect.bottom() as i32).step_by(grid_size as usize) {
            painter.line_segment(
                [Pos2::new(rect.left(), y as f32), Pos2::new(rect.right(), y as f32)],
                egui::Stroke::new(1.0, grid_color),
            );
        }
    }
    
    fn draw_connection(&self, painter: &egui::Painter, from: Pos2, to: Pos2, slot_type: SlotType) {
        let color = slot_type.color();
        
        // Draw a bezier curve
        let control_distance = ((to.x - from.x).abs() * 0.5).max(50.0);
        let control1 = Pos2::new(from.x + control_distance, from.y);
        let control2 = Pos2::new(to.x - control_distance, to.y);
        
        let points: Vec<Pos2> = (0..=32)
            .map(|i| {
                let t = i as f32 / 32.0;
                let t2 = t * t;
                let t3 = t2 * t;
                let mt = 1.0 - t;
                let mt2 = mt * mt;
                let mt3 = mt2 * mt;
                
                Pos2::new(
                    mt3 * from.x + 3.0 * mt2 * t * control1.x + 3.0 * mt * t2 * control2.x + t3 * to.x,
                    mt3 * from.y + 3.0 * mt2 * t * control1.y + 3.0 * mt * t2 * control2.y + t3 * to.y,
                )
            })
            .collect();
        
        painter.add(egui::Shape::line(points, egui::Stroke::new(3.0, color)));
    }
    
    fn draw_node(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, canvas_rect: Rect, node_id: Uuid) {
        let node = match self.nodes.get(&node_id) {
            Some(n) => n.clone(),
            None => return,
        };
        
        let node_width = 180.0 * self.zoom;
        let header_height = 28.0 * self.zoom;
        let slot_height = 24.0 * self.zoom;
        let padding = 8.0 * self.zoom;
        
        let inputs = node.node_type.inputs();
        let outputs = node.node_type.outputs();
        let content_height = (inputs.len().max(outputs.len()) as f32) * slot_height + padding * 2.0;
        let node_height = header_height + content_height;
        
        let node_pos = Pos2::new(
            canvas_rect.left() + node.position.x * self.zoom + self.pan_offset.x,
            canvas_rect.top() + node.position.y * self.zoom + self.pan_offset.y,
        );
        
        let node_rect = Rect::from_min_size(node_pos, Vec2::new(node_width, node_height));
        
        // Skip if outside visible area
        if !canvas_rect.intersects(node_rect) {
            return;
        }
        
        let is_selected = self.selected_node == Some(node_id);
        
        // Node background
        let bg_color = if is_selected {
            egui::Color32::from_rgb(50, 50, 70)
        } else {
            egui::Color32::from_rgb(40, 40, 55)
        };
        
        painter.rect_filled(node_rect, 8.0 * self.zoom, bg_color);
        
        // Selection outline
        if is_selected {
            painter.rect_stroke(
                node_rect,
                8.0 * self.zoom,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 149, 237)),
            );
        }
        
        // Header
        let header_rect = Rect::from_min_size(node_pos, Vec2::new(node_width, header_height));
        painter.rect_filled(
            header_rect,
            egui::Rounding {
                nw: 8.0 * self.zoom,
                ne: 8.0 * self.zoom,
                sw: 0.0,
                se: 0.0,
            },
            node.node_type.color(),
        );
        
        // Header text
        painter.text(
            header_rect.center(),
            egui::Align2::CENTER_CENTER,
            node.node_type.name(),
            egui::FontId::proportional(14.0 * self.zoom),
            egui::Color32::WHITE,
        );
        
        // Input slots
        for (i, input) in inputs.iter().enumerate() {
            let slot_pos = Pos2::new(
                node_pos.x,
                node_pos.y + header_height + padding + (i as f32 + 0.5) * slot_height,
            );
            
            // Slot circle
            painter.circle_filled(slot_pos, 6.0 * self.zoom, input.slot_type.color());
            
            // Slot label
            painter.text(
                Pos2::new(slot_pos.x + 12.0 * self.zoom, slot_pos.y),
                egui::Align2::LEFT_CENTER,
                &input.name,
                egui::FontId::proportional(12.0 * self.zoom),
                egui::Color32::GRAY,
            );
        }
        
        // Output slots
        for (i, output) in outputs.iter().enumerate() {
            let slot_pos = Pos2::new(
                node_pos.x + node_width,
                node_pos.y + header_height + padding + (i as f32 + 0.5) * slot_height,
            );
            
            // Slot circle
            painter.circle_filled(slot_pos, 6.0 * self.zoom, output.slot_type.color());
            
            // Slot label
            painter.text(
                Pos2::new(slot_pos.x - 12.0 * self.zoom, slot_pos.y),
                egui::Align2::RIGHT_CENTER,
                &output.name,
                egui::FontId::proportional(12.0 * self.zoom),
                egui::Color32::GRAY,
            );
        }
        
        // Handle node interaction
        let node_response = ui.interact(node_rect, egui::Id::new(node_id), egui::Sense::click_and_drag());
        
        if node_response.clicked() {
            self.selected_node = Some(node_id);
        }
        
        if node_response.drag_started() {
            self.dragging_node = Some(node_id);
        }
        
        if node_response.dragged() && self.dragging_node == Some(node_id) {
            if let Some(n) = self.nodes.get_mut(&node_id) {
                n.position += node_response.drag_delta() / self.zoom;
            }
        }
        
        if node_response.drag_stopped() {
            self.dragging_node = None;
        }
    }
    
    fn get_slot_position(&self, node: &Node, slot_index: usize, is_input: bool, canvas_rect: Rect) -> Pos2 {
        let node_width = 180.0 * self.zoom;
        let header_height = 28.0 * self.zoom;
        let slot_height = 24.0 * self.zoom;
        let padding = 8.0 * self.zoom;
        
        let node_pos = Pos2::new(
            canvas_rect.left() + node.position.x * self.zoom + self.pan_offset.x,
            canvas_rect.top() + node.position.y * self.zoom + self.pan_offset.y,
        );
        
        let x = if is_input { node_pos.x } else { node_pos.x + node_width };
        let y = node_pos.y + header_height + padding + (slot_index as f32 + 0.5) * slot_height;
        
        Pos2::new(x, y)
    }
    
    /// Show properties panel for a node
    pub fn show_node_properties(&mut self, ui: &mut egui::Ui, node_id: Uuid) {
        let node = match self.nodes.get_mut(&node_id) {
            Some(n) => n,
            None => return,
        };
        
        ui.label(egui::RichText::new(node.node_type.name()).heading());
        ui.separator();
        
        match &mut node.properties {
            NodeProperties::ImageInput { file_path } => {
                ui.horizontal(|ui| {
                    ui.label("File:");
                    if ui.button("Browse...").clicked() {
                        // TODO: File picker
                    }
                });
                if let Some(path) = file_path {
                    ui.label(format!("Loaded: {}", path));
                }
            }
            
            NodeProperties::Color { color } => {
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_rgba_unmultiplied(color);
                });
            }
            
            NodeProperties::Number { value, min, max } => {
                ui.add(egui::Slider::new(value, *min..=*max).text("Value"));
            }
            
            NodeProperties::BrightnessContrast { brightness, contrast } => {
                ui.add(egui::Slider::new(brightness, -1.0..=1.0).text("Brightness"));
                ui.add(egui::Slider::new(contrast, -1.0..=1.0).text("Contrast"));
            }
            
            NodeProperties::HueSaturation { hue, saturation, lightness } => {
                ui.add(egui::Slider::new(hue, -180.0..=180.0).text("Hue"));
                ui.add(egui::Slider::new(saturation, -1.0..=1.0).text("Saturation"));
                ui.add(egui::Slider::new(lightness, -1.0..=1.0).text("Lightness"));
            }
            
            NodeProperties::Levels { black_point, white_point, gamma } => {
                ui.add(egui::Slider::new(black_point, 0.0..=1.0).text("Black Point"));
                ui.add(egui::Slider::new(white_point, 0.0..=1.0).text("White Point"));
                ui.add(egui::Slider::new(gamma, 0.1..=3.0).text("Gamma"));
            }
            
            NodeProperties::Blur { radius, blur_type } => {
                ui.add(egui::Slider::new(radius, 0.0..=50.0).text("Radius"));
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("blur_type")
                        .selected_text(format!("{:?}", blur_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(blur_type, BlurType::Gaussian, "Gaussian");
                            ui.selectable_value(blur_type, BlurType::Box, "Box");
                            ui.selectable_value(blur_type, BlurType::Motion, "Motion");
                        });
                });
            }
            
            NodeProperties::Sharpen { amount, radius } => {
                ui.add(egui::Slider::new(amount, 0.0..=5.0).text("Amount"));
                ui.add(egui::Slider::new(radius, 0.1..=5.0).text("Radius"));
            }
            
            NodeProperties::Noise { amount, monochrome } => {
                ui.add(egui::Slider::new(amount, 0.0..=1.0).text("Amount"));
                ui.checkbox(monochrome, "Monochrome");
            }
            
            NodeProperties::Blend { mode, opacity } => {
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    egui::ComboBox::from_id_salt("blend_mode")
                        .selected_text(format!("{:?}", mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(mode, BlendMode::Normal, "Normal");
                            ui.selectable_value(mode, BlendMode::Multiply, "Multiply");
                            ui.selectable_value(mode, BlendMode::Screen, "Screen");
                            ui.selectable_value(mode, BlendMode::Overlay, "Overlay");
                            ui.selectable_value(mode, BlendMode::SoftLight, "Soft Light");
                            ui.selectable_value(mode, BlendMode::HardLight, "Hard Light");
                            ui.selectable_value(mode, BlendMode::Difference, "Difference");
                            ui.selectable_value(mode, BlendMode::Exclusion, "Exclusion");
                        });
                });
                ui.add(egui::Slider::new(opacity, 0.0..=1.0).text("Opacity"));
            }
            
            NodeProperties::Mask { invert } => {
                ui.checkbox(invert, "Invert Mask");
            }
            
            NodeProperties::Output {} => {
                ui.label("Final output node");
                if ui.button("Export Image...").clicked() {
                    // TODO: Export
                }
            }
        }
    }
}
