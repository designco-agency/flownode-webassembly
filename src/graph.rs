//! Node graph state and rendering

use eframe::egui::{self, Pos2, Rect, Vec2};

/// Convert HSV to RGB (h in 0-1, s in 0-1, v in 0-1)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    
    (r + m, g + m, b + m)
}
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::nodes::{Node, NodeType, NodeProperties, SlotType, BlurDirection, BlendMode};
use crate::ui_components::{style, colors};

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
    
    /// Delete a node and all its connections
    pub fn delete_node(&mut self, node_id: Uuid) {
        // Remove all connections involving this node
        self.connections.retain(|c| c.from_node != node_id && c.to_node != node_id);
        
        // Remove the node
        self.nodes.remove(&node_id);
        
        // Clear selection if this was the selected node
        if self.selected_node == Some(node_id) {
            self.selected_node = None;
        }
        
        log::info!("Deleted node {:?}", node_id);
    }
    
    /// Deselect all nodes
    pub fn deselect_all(&mut self) {
        self.selected_node = None;
        self.pending_connection = None;
    }
    
    /// Delete the currently selected node
    pub fn delete_selected(&mut self) {
        if let Some(node_id) = self.selected_node {
            self.delete_node(node_id);
        }
    }
    
    /// Get a clone of the selected node (for copy)
    pub fn get_selected_node_clone(&self) -> Option<Node> {
        self.selected_node.and_then(|id| self.nodes.get(&id).cloned())
    }
    
    /// Paste a node (create a copy with new ID and offset position)
    pub fn paste_node(&mut self, node: &Node) -> Uuid {
        let mut new_node = node.clone();
        new_node.id = Uuid::new_v4();
        new_node.position.x += 30.0;
        new_node.position.y += 30.0;
        
        let node_id = new_node.id;
        self.nodes.insert(node_id, new_node);
        self.selected_node = Some(node_id);
        
        log::info!("Pasted node {:?}", node_id);
        node_id
    }
    
    /// Insert a node directly (for cloud loading)
    pub fn insert_node(&mut self, node: Node) {
        let node_id = node.id;
        self.nodes.insert(node_id, node);
    }
    
    /// Add a connection between nodes
    pub fn add_connection(&mut self, from_node: Uuid, from_slot: usize, to_node: Uuid, to_slot: usize) {
        // Check if connection already exists
        let exists = self.connections.iter().any(|c| 
            c.from_node == from_node && c.from_slot == from_slot &&
            c.to_node == to_node && c.to_slot == to_slot
        );
        
        if !exists {
            self.connections.push(Connection {
                from_node,
                from_slot,
                to_node,
                to_slot,
            });
        }
    }
    
    /// Delete a specific connection
    pub fn delete_connection(&mut self, from_node: Uuid, from_slot: usize, to_node: Uuid, to_slot: usize) {
        self.connections.retain(|c| {
            !(c.from_node == from_node && c.from_slot == from_slot && 
              c.to_node == to_node && c.to_slot == to_slot)
        });
    }
    
    /// Serialize the graph to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Deserialize the graph from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    /// Check if graph has unsaved changes (always true for now, could track dirty state)
    pub fn is_dirty(&self) -> bool {
        !self.nodes.is_empty()
    }
    
    /// Get iterator over nodes (for compatibility layer)
    pub fn nodes_iter(&self) -> impl Iterator<Item = (&Uuid, &Node)> {
        self.nodes.iter()
    }
    
    /// Get iterator over connections (for compatibility layer)
    pub fn connections_iter(&self) -> impl Iterator<Item = &Connection> {
        self.connections.iter()
    }
    
    /// Get current pan offset
    pub fn pan_offset(&self) -> Vec2 {
        self.pan_offset
    }
    
    /// Get current zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom
    }
    
    /// Create graph from parts (for loading)
    pub fn from_parts(nodes: HashMap<Uuid, Node>, connections: Vec<Connection>) -> Self {
        Self {
            nodes,
            connections,
            selected_node: None,
            dragging_node: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            pending_connection: None,
        }
    }
    
    /// Set pan and zoom (for loading viewport)
    pub fn set_viewport(&mut self, pan: Vec2, zoom: f32) {
        self.pan_offset = pan;
        self.zoom = zoom;
    }
    
    /// Set the image ID for an Image node
    /// Returns true if the node was an Image node and was updated
    pub fn set_node_image(&mut self, node_id: Uuid, image_id: u64) -> bool {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            if let NodeProperties::Image { texture_id, .. } = &mut node.properties {
                *texture_id = Some(image_id);
                return true;
            }
        }
        false
    }
    
    /// Get the image ID for a node (if it has one)
    pub fn get_node_image(&self, node_id: Uuid) -> Option<u64> {
        self.nodes.get(&node_id).and_then(|node| {
            if let NodeProperties::Image { texture_id, .. } = &node.properties {
                *texture_id
            } else {
                None
            }
        })
    }
    
    pub fn add_node(&mut self, node_type: NodeType) -> Uuid {
        // Place new nodes in the center of the viewport with slight random offset
        let node_count = self.nodes.len() as f32;
        let offset_x = (node_count * 30.0) % 200.0;
        let offset_y = (node_count * 20.0) % 150.0;
        let position = Vec2::new(
            100.0 + offset_x,
            100.0 + offset_y,
        );
        log::info!("Creating node {:?} at {:?}", node_type, position);
        let node = Node::new(node_type, position);
        let node_id = node.id;
        self.selected_node = Some(node_id);
        self.nodes.insert(node_id, node);
        log::info!("Total nodes: {}", self.nodes.len());
        node_id
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
        painter.rect_filled(canvas_rect, 0.0, colors::CANVAS_BG);
        
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
                self.draw_connection(&painter, from_pos, to_pos, SlotType::Content);
            }
        }
        
        // Draw pending connection (wire following mouse)
        if let Some(pending) = &self.pending_connection {
            if let Some(node) = self.nodes.get(&pending.from_node) {
                let from_pos = self.get_slot_position(node, pending.from_slot, !pending.is_output, canvas_rect);
                if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    // Get the slot type for coloring
                    let slot_type = if pending.is_output {
                        node.node_type.outputs().get(pending.from_slot)
                            .map(|s| s.slot_type)
                            .unwrap_or(SlotType::Content)
                    } else {
                        node.node_type.inputs().get(pending.from_slot)
                            .map(|s| s.slot_type)
                            .unwrap_or(SlotType::Content)
                    };
                    self.draw_connection(&painter, from_pos, mouse_pos, slot_type);
                }
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
        
        // Keyboard shortcuts
        if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            if let Some(node_id) = self.selected_node {
                self.delete_node(node_id);
            }
        }
        
        // Escape to deselect
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_node = None;
            self.pending_connection = None;
        }
        
        // Zoom with scroll wheel
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_factor = 1.0 + scroll_delta * 0.001;
            self.zoom = (self.zoom * zoom_factor).clamp(0.25, 4.0);
        }
        
        // Pan with middle mouse button or Ctrl+drag
        if response.dragged_by(egui::PointerButton::Middle) 
            || (response.dragged() && ui.input(|i| i.modifiers.ctrl)) 
        {
            self.pan_offset += response.drag_delta();
        }
        
        // Cancel pending connection on right click or release without target
        if response.secondary_clicked() {
            self.pending_connection = None;
        }
        
        // Clear pending connection if mouse released without connecting
        if ui.input(|i| i.pointer.any_released()) && self.pending_connection.is_some() {
            // Small delay to allow the slot drop handler to run first
            // If still pending after this frame, clear it
        }
        
        // Actually clear on next frame if no connection was made
        if !ui.input(|i| i.pointer.any_down()) && self.pending_connection.is_some() {
            self.pending_connection = None;
        }
    }
    
    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_size = style::GRID_SIZE * self.zoom;
        let grid_color = colors::GRID_LINE;
        
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
        
        painter.add(egui::Shape::line(points, egui::Stroke::new(style::CONNECTION_WIDTH, color)));
    }
    
    fn draw_node(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, canvas_rect: Rect, node_id: Uuid) {
        let node = match self.nodes.get(&node_id) {
            Some(n) => n.clone(),
            None => return,
        };
        
        let node_width = style::NODE_WIDTH * self.zoom;
        let header_height = style::NODE_HEADER_HEIGHT * self.zoom;
        let slot_height = style::NODE_SLOT_HEIGHT * self.zoom;
        let padding = style::NODE_PADDING * self.zoom;
        
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
            colors::NODE_BG_SELECTED
        } else {
            colors::NODE_BG
        };
        
        painter.rect_filled(node_rect, style::NODE_ROUNDING * self.zoom, bg_color);
        
        // Selection outline
        if is_selected {
            painter.rect_stroke(
                node_rect,
                style::NODE_ROUNDING * self.zoom,
                egui::Stroke::new(2.0, colors::NODE_SELECTED_OUTLINE),
            );
        }
        
        // Header
        let header_rect = Rect::from_min_size(node_pos, Vec2::new(node_width, header_height));
        let rounding = style::NODE_ROUNDING * self.zoom;
        painter.rect_filled(
            header_rect,
            egui::Rounding {
                nw: rounding,
                ne: rounding,
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
        
        // Input slots (with interaction)
        for (i, input) in inputs.iter().enumerate() {
            let slot_pos = Pos2::new(
                node_pos.x,
                node_pos.y + header_height + padding + (i as f32 + 0.5) * slot_height,
            );
            
            let slot_radius = style::SLOT_RADIUS * self.zoom;
            let slot_rect = Rect::from_center_size(slot_pos, Vec2::splat(slot_radius * 3.0));
            let slot_id = egui::Id::new((node_id, "input", i));
            let slot_response = ui.interact(slot_rect, slot_id, egui::Sense::click_and_drag());
            
            // Check if this input has a connection
            let has_connection = self.connections.iter().any(|c| c.to_node == node_id && c.to_slot == i);
            
            // Highlight on hover
            let is_hovered = slot_response.hovered();
            let radius = if is_hovered { slot_radius * 1.3 } else { slot_radius };
            
            // Slot circle - filled if connected, hollow if not
            if has_connection {
                painter.circle_filled(slot_pos, radius, input.slot_type.color());
            } else {
                painter.circle_stroke(slot_pos, radius, egui::Stroke::new(2.0, input.slot_type.color()));
                // Small dot in center
                painter.circle_filled(slot_pos, radius * 0.3, input.slot_type.color());
            }
            if is_hovered {
                painter.circle_stroke(slot_pos, radius + 2.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
            }
            
            // Slot label
            painter.text(
                Pos2::new(slot_pos.x + 12.0 * self.zoom, slot_pos.y),
                egui::Align2::LEFT_CENTER,
                &input.name,
                egui::FontId::proportional(12.0 * self.zoom),
                egui::Color32::GRAY,
            );
            
            // Handle connection drop on input slot
            if slot_response.hovered() && ui.input(|i| i.pointer.any_released()) {
                if let Some(pending) = &self.pending_connection {
                    if pending.is_output && pending.from_node != node_id {
                        // Remove existing connection to this input (only one connection per input)
                        self.connections.retain(|c| !(c.to_node == node_id && c.to_slot == i));
                        
                        // Complete the connection
                        let new_conn = Connection {
                            from_node: pending.from_node,
                            from_slot: pending.from_slot,
                            to_node: node_id,
                            to_slot: i,
                        };
                        self.connections.push(new_conn);
                        log::info!("Connection created: {:?} -> {:?}", pending.from_node, node_id);
                    }
                }
            }
            
            // Click on connected input to delete connection
            if slot_response.clicked() && has_connection && self.pending_connection.is_none() {
                self.connections.retain(|c| !(c.to_node == node_id && c.to_slot == i));
                log::info!("Connection deleted from input slot");
            }
        }
        
        // Output slots (with interaction)
        for (i, output) in outputs.iter().enumerate() {
            let slot_pos = Pos2::new(
                node_pos.x + node_width,
                node_pos.y + header_height + padding + (i as f32 + 0.5) * slot_height,
            );
            
            let slot_radius = style::SLOT_RADIUS * self.zoom;
            let slot_rect = Rect::from_center_size(slot_pos, Vec2::splat(slot_radius * 3.0));
            let slot_id = egui::Id::new((node_id, "output", i));
            let slot_response = ui.interact(slot_rect, slot_id, egui::Sense::click_and_drag());
            
            // Check if this output has connections
            let has_connection = self.connections.iter().any(|c| c.from_node == node_id && c.from_slot == i);
            
            // Highlight on hover
            let is_hovered = slot_response.hovered();
            let radius = if is_hovered { slot_radius * 1.3 } else { slot_radius };
            
            // Slot circle - filled if connected, hollow if not
            if has_connection {
                painter.circle_filled(slot_pos, radius, output.slot_type.color());
            } else {
                painter.circle_stroke(slot_pos, radius, egui::Stroke::new(2.0, output.slot_type.color()));
                painter.circle_filled(slot_pos, radius * 0.3, output.slot_type.color());
            }
            if is_hovered {
                painter.circle_stroke(slot_pos, radius + 2.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
            }
            
            // Slot label
            painter.text(
                Pos2::new(slot_pos.x - 12.0 * self.zoom, slot_pos.y),
                egui::Align2::RIGHT_CENTER,
                &output.name,
                egui::FontId::proportional(12.0 * self.zoom),
                egui::Color32::GRAY,
            );
            
            // Start connection drag from output slot
            if slot_response.drag_started() {
                self.pending_connection = Some(PendingConnection {
                    from_node: node_id,
                    from_slot: i,
                    is_output: true,
                });
                log::info!("Started connection from output slot {}", i);
            }
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
        let node_width = style::NODE_WIDTH * self.zoom;
        let header_height = style::NODE_HEADER_HEIGHT * self.zoom;
        let slot_height = style::NODE_SLOT_HEIGHT * self.zoom;
        let padding = style::NODE_PADDING * self.zoom;
        
        let node_pos = Pos2::new(
            canvas_rect.left() + node.position.x * self.zoom + self.pan_offset.x,
            canvas_rect.top() + node.position.y * self.zoom + self.pan_offset.y,
        );
        
        let x = if is_input { node_pos.x } else { node_pos.x + node_width };
        let y = node_pos.y + header_height + padding + (slot_index as f32 + 0.5) * slot_height;
        
        Pos2::new(x, y)
    }
    
    /// Draw a color wheel for color grading
    fn color_wheel(ui: &mut egui::Ui, label: &str, wheel: &mut crate::nodes::ColorWheel) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).small());
            
            // Size of the wheel
            let size = 60.0;
            let (response, painter) = ui.allocate_painter(
                egui::vec2(size, size),
                egui::Sense::click_and_drag(),
            );
            
            let center = response.rect.center();
            let radius = size / 2.0 - 4.0;
            
            // Draw the wheel background (color gradient)
            for i in 0..36 {
                let angle1 = (i as f32) * std::f32::consts::PI * 2.0 / 36.0;
                let angle2 = ((i + 1) as f32) * std::f32::consts::PI * 2.0 / 36.0;
                
                // HSV to RGB for the wedge color
                let hue = i as f32 / 36.0;
                let (r, g, b) = hsv_to_rgb(hue, 0.7, 0.8);
                let color = egui::Color32::from_rgb(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );
                
                let p1 = center;
                let p2 = center + egui::vec2(angle1.cos() * radius, angle1.sin() * radius);
                let p3 = center + egui::vec2(angle2.cos() * radius, angle2.sin() * radius);
                
                painter.add(egui::Shape::convex_polygon(
                    vec![p1, p2, p3],
                    color,
                    egui::Stroke::NONE,
                ));
            }
            
            // Draw border
            painter.circle_stroke(center, radius, egui::Stroke::new(1.0, egui::Color32::GRAY));
            
            // Draw center point (neutral)
            painter.circle_filled(center, 3.0, egui::Color32::WHITE);
            
            // Draw current position
            let pos_x = center.x + wheel.x * radius * 0.9;
            let pos_y = center.y + wheel.y * radius * 0.9;
            painter.circle_filled(egui::pos2(pos_x, pos_y), 5.0, egui::Color32::WHITE);
            painter.circle_stroke(egui::pos2(pos_x, pos_y), 5.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            
            // Handle interaction
            if response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let dx = (pos.x - center.x) / radius;
                    let dy = (pos.y - center.y) / radius;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    // Clamp to circle
                    if dist > 1.0 {
                        wheel.x = dx / dist;
                        wheel.y = dy / dist;
                    } else {
                        wheel.x = dx;
                        wheel.y = dy;
                    }
                }
            }
            
            // Double-click to reset
            if response.double_clicked() {
                wheel.x = 0.0;
                wheel.y = 0.0;
                wheel.luminance = 0.0;
            }
            
            // Luminance slider below
            ui.add(egui::Slider::new(&mut wheel.luminance, -100.0..=100.0)
                .show_value(false)
                .text(""));
        });
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
            // === Content Nodes ===
            NodeProperties::Image { texture_id, .. } => {
                if let Some(id) = texture_id {
                    ui.label(format!("Image loaded (ID: {})", id));
                } else {
                    ui.label("Drop an image to load");
                }
            }
            
            NodeProperties::Content { content } => {
                if let Some(c) = content {
                    ui.label(format!("Content: {}", c));
                } else {
                    ui.label("No content");
                }
            }
            
            NodeProperties::Bucket { images } => {
                ui.label(format!("{} images", images.len()));
            }
            
            // === Adjust Node (full color grading) ===
            NodeProperties::Adjust { 
                brightness, contrast, saturation, exposure,
                highlights, shadows, temperature, tint,
                vibrance, gamma, 
                lift, gamma_wheel, gain, offset,
                color_boost, hue_rotation, luminance_mix, ..
            } => {
                egui::CollapsingHeader::new("Basic Adjustments")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(brightness, -100.0..=100.0).text("Brightness"));
                        ui.add(egui::Slider::new(contrast, -100.0..=100.0).text("Contrast"));
                        ui.add(egui::Slider::new(saturation, -100.0..=100.0).text("Saturation"));
                        ui.add(egui::Slider::new(exposure, -100.0..=100.0).text("Exposure"));
                    });
                
                egui::CollapsingHeader::new("Tone")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(highlights, -100.0..=100.0).text("Highlights"));
                        ui.add(egui::Slider::new(shadows, -100.0..=100.0).text("Shadows"));
                        ui.add(egui::Slider::new(gamma, -100.0..=100.0).text("Gamma"));
                    });
                
                egui::CollapsingHeader::new("Color")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(temperature, -100.0..=100.0).text("Temperature"));
                        ui.add(egui::Slider::new(tint, -100.0..=100.0).text("Tint"));
                        ui.add(egui::Slider::new(vibrance, -100.0..=100.0).text("Vibrance"));
                        ui.add(egui::Slider::new(color_boost, -100.0..=100.0).text("Color Boost"));
                        ui.add(egui::Slider::new(hue_rotation, -180.0..=180.0).text("Hue Rotation"));
                    });
                
                egui::CollapsingHeader::new("Mix")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(luminance_mix, 0.0..=100.0).text("Luminance Mix"));
                    });
                
                // Color Grading Wheels
                egui::CollapsingHeader::new("🎨 Color Wheels")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            Self::color_wheel(ui, "Lift", lift);
                            Self::color_wheel(ui, "Gamma", gamma_wheel);
                        });
                        ui.horizontal(|ui| {
                            Self::color_wheel(ui, "Gain", gain);
                            Self::color_wheel(ui, "Offset", offset);
                        });
                    });
            }
            
            // === Effects Node ===
            NodeProperties::Effects {
                gaussian_blur, directional_blur, directional_blur_angle,
                progressive_blur, progressive_blur_direction, progressive_blur_falloff,
                glass_blinds, glass_blinds_frequency, glass_blinds_angle, glass_blinds_phase,
                grain, grain_size, grain_monochrome, grain_seed,
                sharpen, vignette, vignette_roundness, vignette_smoothness
            } => {
                egui::CollapsingHeader::new("Blur")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(gaussian_blur, 0.0..=100.0).text("Gaussian"));
                        ui.add(egui::Slider::new(directional_blur, 0.0..=100.0).text("Directional"));
                        if *directional_blur > 0.0 {
                            ui.add(egui::Slider::new(directional_blur_angle, 0.0..=360.0).text("Angle"));
                        }
                        ui.add(egui::Slider::new(progressive_blur, 0.0..=100.0).text("Progressive"));
                        if *progressive_blur > 0.0 {
                            ui.add(egui::Slider::new(progressive_blur_falloff, 0.0..=100.0).text("Falloff"));
                            ui.horizontal(|ui| {
                                ui.label("Direction:");
                                egui::ComboBox::from_id_salt("blur_dir")
                                    .selected_text(format!("{:?}", progressive_blur_direction))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(progressive_blur_direction, BlurDirection::Top, "Top");
                                        ui.selectable_value(progressive_blur_direction, BlurDirection::Bottom, "Bottom");
                                        ui.selectable_value(progressive_blur_direction, BlurDirection::Left, "Left");
                                        ui.selectable_value(progressive_blur_direction, BlurDirection::Right, "Right");
                                    });
                            });
                        }
                    });
                
                egui::CollapsingHeader::new("Glass Blinds")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(glass_blinds, 0.0..=100.0).text("Intensity"));
                        ui.add(egui::Slider::new(glass_blinds_frequency, 1.0..=50.0).text("Frequency"));
                        ui.add(egui::Slider::new(glass_blinds_angle, 0.0..=360.0).text("Angle"));
                        ui.add(egui::Slider::new(glass_blinds_phase, 0.0..=100.0).text("Phase"));
                    });
                
                egui::CollapsingHeader::new("Grain")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(grain, 0.0..=100.0).text("Amount"));
                        ui.add(egui::Slider::new(grain_size, 1.0..=10.0).text("Size"));
                        ui.checkbox(grain_monochrome, "Monochrome");
                        ui.add(egui::DragValue::new(grain_seed).prefix("Seed: "));
                    });
                
                egui::CollapsingHeader::new("Sharpen & Vignette")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(sharpen, 0.0..=100.0).text("Sharpen"));
                        ui.separator();
                        ui.add(egui::Slider::new(vignette, 0.0..=100.0).text("Vignette"));
                        if *vignette > 0.0 {
                            ui.add(egui::Slider::new(vignette_roundness, 0.0..=100.0).text("Roundness"));
                            ui.add(egui::Slider::new(vignette_smoothness, 0.0..=100.0).text("Smoothness"));
                        }
                    });
            }
            
            // === Text Nodes ===
            NodeProperties::Text { text } => {
                ui.text_edit_multiline(text);
            }
            
            NodeProperties::Concat { separator } => {
                ui.horizontal(|ui| {
                    ui.label("Separator:");
                    ui.text_edit_singleline(separator);
                });
            }
            
            NodeProperties::Splitter { delimiter } => {
                ui.horizontal(|ui| {
                    ui.label("Delimiter:");
                    ui.text_edit_singleline(delimiter);
                });
            }
            
            NodeProperties::Postit { text, color } => {
                ui.text_edit_multiline(text);
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_rgba_unmultiplied(color);
                });
            }
            
            // === Utility Nodes ===
            NodeProperties::Compare {} => {
                ui.label("Drop 2 images to compare");
            }
            
            NodeProperties::Composition { layers } => {
                ui.label(format!("{} layers", layers.len()));
            }
            
            NodeProperties::Router { active_output } => {
                ui.add(egui::Slider::new(active_output, 0..=2).text("Output"));
            }
            
            NodeProperties::Batch { items } => {
                ui.label(format!("{} items in batch", items.len()));
            }
            
            NodeProperties::Title { text } => {
                ui.text_edit_singleline(text);
            }
            
            NodeProperties::Group {} | NodeProperties::Folder {} | NodeProperties::Convertor {} => {
                ui.label("No properties");
            }
            
            // === AI Nodes ===
            NodeProperties::Omni { model, prompt, negative_prompt, seed } => {
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.text_edit_singleline(model);
                });
                ui.label("Prompt:");
                ui.text_edit_multiline(prompt);
                ui.label("Negative:");
                ui.text_edit_multiline(negative_prompt);
                ui.horizontal(|ui| {
                    ui.label("Seed:");
                    if let Some(s) = seed {
                        let mut val = *s as i32;
                        if ui.add(egui::DragValue::new(&mut val)).changed() {
                            *seed = Some(val as u32);
                        }
                    } else {
                        ui.label("Random");
                    }
                });
            }
            
            NodeProperties::Llm { model, system_prompt } => {
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.text_edit_singleline(model);
                });
                ui.label("System Prompt:");
                ui.text_edit_multiline(system_prompt);
            }
            
            NodeProperties::Video { model, duration, aspect_ratio } => {
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.text_edit_singleline(model);
                });
                ui.add(egui::Slider::new(duration, 4..=10).text("Duration (s)"));
                ui.horizontal(|ui| {
                    ui.label("Aspect:");
                    egui::ComboBox::from_id_salt("aspect")
                        .selected_text(aspect_ratio.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(aspect_ratio, "16:9".to_string(), "16:9");
                            ui.selectable_value(aspect_ratio, "9:16".to_string(), "9:16");
                            ui.selectable_value(aspect_ratio, "1:1".to_string(), "1:1");
                        });
                });
            }
            
            NodeProperties::Upscaler { model, scale } => {
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.text_edit_singleline(model);
                });
                ui.add(egui::Slider::new(scale, 2..=4).text("Scale"));
            }
            
            NodeProperties::Vector {} | NodeProperties::Rodin3d {} | NodeProperties::MindMap {} => {
                ui.label("AI processing node");
                ui.label("Connect an image/prompt to generate");
            }
        }
    }
}
