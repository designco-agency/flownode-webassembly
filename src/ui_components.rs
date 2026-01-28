//! Reusable UI components for FlowNode
//! 
//! All UI elements should use these components to maintain consistency.
//! This allows easy theming and style updates across the entire application.

use eframe::egui::{self, Color32, Rounding, Stroke, Ui, Vec2, Response};

/// Standard spacing and sizing values
pub mod style {
    /// Node dimensions
    pub const NODE_WIDTH: f32 = 180.0;
    pub const NODE_HEADER_HEIGHT: f32 = 28.0;
    pub const NODE_SLOT_HEIGHT: f32 = 24.0;
    pub const NODE_PADDING: f32 = 8.0;
    pub const NODE_ROUNDING: f32 = 8.0;
    
    /// Slot dimensions  
    pub const SLOT_RADIUS: f32 = 6.0;
    
    /// Connection line width
    pub const CONNECTION_WIDTH: f32 = 3.0;
    
    /// Grid
    pub const GRID_SIZE: f32 = 20.0;
    
    /// Slider dimensions
    pub const SLIDER_HEIGHT: f32 = 18.0;
    pub const SLIDER_ROUNDING: f32 = 4.0;
}

/// Color palette for consistent theming
pub mod colors {
    use super::Color32;
    
    // Canvas
    pub const CANVAS_BG: Color32 = Color32::from_rgb(26, 26, 46);
    pub const GRID_LINE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 15);
    
    // Nodes
    pub const NODE_BG: Color32 = Color32::from_rgb(40, 40, 55);
    pub const NODE_BG_SELECTED: Color32 = Color32::from_rgb(50, 50, 70);
    pub const NODE_SELECTED_OUTLINE: Color32 = Color32::from_rgb(100, 149, 237);
    
    // Node categories
    pub const CAT_INPUT: Color32 = Color32::from_rgb(76, 175, 80);      // Green
    pub const CAT_ADJUST: Color32 = Color32::from_rgb(255, 152, 0);     // Orange
    pub const CAT_FILTER: Color32 = Color32::from_rgb(33, 150, 243);    // Blue
    pub const CAT_COMBINE: Color32 = Color32::from_rgb(156, 39, 176);   // Purple
    pub const CAT_OUTPUT: Color32 = Color32::from_rgb(244, 67, 54);     // Red
    
    // Data types (for slots and connections)
    pub const TYPE_IMAGE: Color32 = Color32::from_rgb(255, 193, 7);     // Amber
    pub const TYPE_COLOR: Color32 = Color32::from_rgb(233, 30, 99);     // Pink
    pub const TYPE_NUMBER: Color32 = Color32::from_rgb(0, 188, 212);    // Cyan
    pub const TYPE_MASK: Color32 = Color32::from_rgb(158, 158, 158);    // Gray
    
    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::WHITE;
    pub const TEXT_SECONDARY: Color32 = Color32::GRAY;
    
    // Slider
    pub const SLIDER_BG: Color32 = Color32::from_rgb(30, 30, 45);
    pub const SLIDER_FILL: Color32 = Color32::from_rgb(100, 149, 237);
    pub const SLIDER_HANDLE: Color32 = Color32::WHITE;
}

/// Unified slider component
/// 
/// Usage:
/// ```
/// let response = FlowSlider::new(&mut value, 0.0..=1.0)
///     .label("Brightness")
///     .show(ui);
/// ```
pub struct FlowSlider<'a> {
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: Option<&'a str>,
    suffix: Option<&'a str>,
    step: Option<f32>,
    logarithmic: bool,
}

impl<'a> FlowSlider<'a> {
    pub fn new(value: &'a mut f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            label: None,
            suffix: None,
            step: None,
            logarithmic: false,
        }
    }
    
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    
    pub fn suffix(mut self, suffix: &'a str) -> Self {
        self.suffix = Some(suffix);
        self
    }
    
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    
    pub fn logarithmic(mut self) -> Self {
        self.logarithmic = true;
        self
    }
    
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut slider = egui::Slider::new(self.value, self.range)
            .clamp_to_range(true);
        
        if let Some(label) = self.label {
            slider = slider.text(label);
        }
        
        if let Some(suffix) = self.suffix {
            slider = slider.suffix(suffix);
        }
        
        if let Some(step) = self.step {
            slider = slider.step_by(step as f64);
        }
        
        if self.logarithmic {
            slider = slider.logarithmic(true);
        }
        
        ui.add(slider)
    }
}

/// Unified color picker component
pub struct FlowColorPicker<'a> {
    color: &'a mut [f32; 4],
    label: Option<&'a str>,
    alpha: bool,
}

impl<'a> FlowColorPicker<'a> {
    pub fn new(color: &'a mut [f32; 4]) -> Self {
        Self {
            color,
            label: None,
            alpha: true,
        }
    }
    
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    
    pub fn no_alpha(mut self) -> Self {
        self.alpha = false;
        self
    }
    
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            if let Some(label) = self.label {
                ui.label(label);
            }
            if self.alpha {
                ui.color_edit_button_rgba_unmultiplied(self.color)
            } else {
                let mut rgb = [self.color[0], self.color[1], self.color[2]];
                let response = ui.color_edit_button_rgb(&mut rgb);
                self.color[0] = rgb[0];
                self.color[1] = rgb[1];
                self.color[2] = rgb[2];
                response
            }
        }).inner
    }
}

/// Unified checkbox component
pub struct FlowCheckbox<'a> {
    checked: &'a mut bool,
    label: &'a str,
}

impl<'a> FlowCheckbox<'a> {
    pub fn new(checked: &'a mut bool, label: &'a str) -> Self {
        Self { checked, label }
    }
    
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.checkbox(self.checked, self.label)
    }
}

/// Unified dropdown/combo box component
pub struct FlowDropdown<'a, T: PartialEq + Clone + std::fmt::Debug> {
    selected: &'a mut T,
    options: &'a [(T, &'a str)],
    label: Option<&'a str>,
    id: &'a str,
}

impl<'a, T: PartialEq + Clone + std::fmt::Debug> FlowDropdown<'a, T> {
    pub fn new(selected: &'a mut T, options: &'a [(T, &'a str)], id: &'a str) -> Self {
        Self {
            selected,
            options,
            label: None,
            id,
        }
    }
    
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    
    pub fn show(self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if let Some(label) = self.label {
                ui.label(label);
            }
            
            let current_label = self.options
                .iter()
                .find(|(v, _)| v == self.selected)
                .map(|(_, l)| *l)
                .unwrap_or("Unknown");
            
            egui::ComboBox::from_id_salt(self.id)
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    for (value, label) in self.options {
                        ui.selectable_value(self.selected, value.clone(), *label);
                    }
                });
        });
    }
}

/// Section header for properties panel
pub fn section_header(ui: &mut Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(title).strong().size(14.0));
    ui.separator();
    ui.add_space(4.0);
}

/// Collapsible section
pub fn collapsible_section(ui: &mut Ui, title: &str, default_open: bool, add_contents: impl FnOnce(&mut Ui)) {
    egui::CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, add_contents);
}
