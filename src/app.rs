//! Main application state and UI

use eframe::egui;
use std::collections::HashMap;
use crate::graph::NodeGraph;
use crate::image_data::{ImageData, TextureHandle};
use crate::executor::Executor;

#[cfg(target_arch = "wasm32")]
use js_sys;

/// The main FlowNode application
pub struct FlowNodeApp {
    /// The node graph editor
    graph: NodeGraph,
    
    /// Show the properties panel
    show_properties: bool,
    
    /// Show the node library
    show_library: bool,
    
    /// Current zoom level (for status bar)
    zoom: f32,
    
    /// Dark mode (always true for now)
    dark_mode: bool,
    
    /// Loaded images (keyed by a unique ID)
    images: HashMap<u64, ImageData>,
    
    /// Texture cache for rendering
    textures: HashMap<u64, TextureHandle>,
    
    /// Next image ID
    next_image_id: u64,
    
    /// Pending image load (node ID waiting for image)
    pending_image_load: Option<uuid::Uuid>,
    
    /// Node graph executor
    executor: Executor,
    
    /// Output image from last execution
    output_image: Option<ImageData>,
    
    /// Output texture for display
    output_texture: Option<TextureHandle>,
}

impl FlowNodeApp {
    /// Create a new FlowNode application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts and style
        let mut style = (*cc.egui_ctx.style()).clone();
        
        // Use a modern, clean look
        style.visuals = egui::Visuals::dark();
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.panel_fill = egui::Color32::from_rgb(26, 26, 46);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(16, 16, 32);
        
        cc.egui_ctx.set_style(style);
        
        Self {
            graph: NodeGraph::new(),
            show_properties: true,
            show_library: true,
            zoom: 1.0,
            images: HashMap::new(),
            textures: HashMap::new(),
            next_image_id: 1,
            pending_image_load: None,
            dark_mode: true,
            executor: Executor::new(),
            output_image: None,
            output_texture: None,
        }
    }
    
    /// Load image from bytes and assign to selected node or create new node
    fn load_image_bytes(&mut self, ctx: &egui::Context, bytes: &[u8]) {
        match crate::image_data::decode_image(bytes) {
            Ok(image_data) => {
                let image_id = self.next_image_id;
                self.next_image_id += 1;
                
                // Create texture for display
                let texture = TextureHandle::from_image_data(
                    ctx,
                    &format!("image_{}", image_id),
                    &image_data,
                );
                
                log::info!("Loaded image {}x{}", image_data.width, image_data.height);
                
                // Store the image and texture
                self.images.insert(image_id, image_data);
                self.textures.insert(image_id, texture);
                
                // Assign to selected node if it's an ImageInput, otherwise create new
                if let Some(node_id) = self.graph.selected_node() {
                    if self.graph.set_node_image(node_id, image_id) {
                        log::info!("Assigned image to selected node");
                        return;
                    }
                }
                
                // Create a new ImageInput node with this image
                let node_id = self.graph.add_node(crate::nodes::NodeType::ImageInput);
                self.graph.set_node_image(node_id, image_id);
                log::info!("Created new ImageInput node with image");
            }
            Err(e) => {
                log::error!("Failed to load image: {}", e);
            }
        }
    }
    
    /// Get texture handle by ID
    pub fn get_texture(&self, image_id: u64) -> Option<&TextureHandle> {
        self.textures.get(&image_id)
    }
    
    /// Export the output image
    fn export_output(&self) {
        if let Some(output) = &self.output_image {
            match crate::image_data::encode_png(output) {
                Ok(png_bytes) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        // Download via JavaScript
                        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
                        let js_code = format!(
                            r#"
                            const link = document.createElement('a');
                            link.href = 'data:image/png;base64,{}';
                            link.download = 'flownode-output.png';
                            link.click();
                            "#,
                            encoded
                        );
                        let _ = js_sys::eval(&js_code);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        log::info!("Export: {} bytes PNG", png_bytes.len());
                    }
                }
                Err(e) => {
                    log::error!("Failed to encode PNG: {}", e);
                }
            }
        }
    }
    
    /// Run the node graph and produce output
    fn run_graph(&mut self, ctx: &egui::Context) {
        log::info!("Running node graph...");
        
        match self.executor.execute(&self.graph, &self.images) {
            Ok(Some(output)) => {
                log::info!("Execution successful! Output: {}x{}", output.width, output.height);
                
                // Create texture for display
                let texture = TextureHandle::from_image_data(
                    ctx,
                    "output",
                    &output,
                );
                
                self.output_texture = Some(texture);
                self.output_image = Some(output);
            }
            Ok(None) => {
                log::info!("Execution complete, no output (no output node connected)");
            }
            Err(e) => {
                log::error!("Execution failed: {}", e);
            }
        }
    }
}

impl eframe::App for FlowNodeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        self.graph = NodeGraph::new();
                        ui.close_menu();
                    }
                    if ui.button("Open...").clicked() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            // Trigger file input click via JS
                            let _ = js_sys::eval("document.getElementById('file-input')?.click()");
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        if let Ok(json) = self.graph.to_json() {
                            #[cfg(target_arch = "wasm32")]
                            {
                                // Download JSON file via JS
                                let js_code = format!(
                                    r#"
                                    const blob = new Blob([`{}`], {{type: 'application/json'}});
                                    const url = URL.createObjectURL(blob);
                                    const a = document.createElement('a');
                                    a.href = url;
                                    a.download = 'flownode-project.json';
                                    a.click();
                                    URL.revokeObjectURL(url);
                                    "#,
                                    json.replace('`', "\\`").replace("${", "\\${")
                                );
                                let _ = js_sys::eval(&js_code);
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                log::info!("Save: {}", json);
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export Image...").clicked() {
                        // TODO: Export
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_library, "Node Library");
                    ui.checkbox(&mut self.show_properties, "Properties");
                    ui.separator();
                    if ui.button("Reset Zoom").clicked() {
                        self.zoom = 1.0;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About FlowNode").clicked() {
                        // TODO: About dialog
                        ui.close_menu();
                    }
                });
                
                // Run button
                ui.separator();
                if ui.button("▶ Run").clicked() {
                    self.run_graph(ctx);
                }
                
                // Right-aligned status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.output_image.is_some() {
                        ui.label("✅ Output ready");
                        ui.separator();
                    }
                    ui.label(format!("Zoom: {:.0}%", self.zoom * 100.0));
                    ui.separator();
                    ui.label(format!("{} nodes", self.graph.node_count()));
                });
            });
        });
        
        // Left panel: Node library
        if self.show_library {
            egui::SidePanel::left("node_library")
                .resizable(true)
                .default_width(200.0)
                .min_width(150.0)
                .show(ctx, |ui| {
                    ui.heading("Nodes");
                    ui.separator();
                    
                    egui::CollapsingHeader::new("📥 Input")
                        .default_open(true)
                        .show(ui, |ui| {
                            if ui.button("Image Input").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::ImageInput);
                            }
                            if ui.button("Color").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Color);
                            }
                            if ui.button("Number").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Number);
                            }
                        });
                    
                    egui::CollapsingHeader::new("🎨 Adjustments")
                        .default_open(true)
                        .show(ui, |ui| {
                            if ui.button("Brightness/Contrast").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::BrightnessContrast);
                            }
                            if ui.button("Hue/Saturation").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::HueSaturation);
                            }
                            if ui.button("Levels").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Levels);
                            }
                        });
                    
                    egui::CollapsingHeader::new("🔧 Filters")
                        .default_open(true)
                        .show(ui, |ui| {
                            if ui.button("Blur").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Blur);
                            }
                            if ui.button("Sharpen").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Sharpen);
                            }
                            if ui.button("Noise").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Noise);
                            }
                            if ui.button("Invert").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Invert);
                            }
                            if ui.button("Grayscale").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Grayscale);
                            }
                        });
                    
                    egui::CollapsingHeader::new("🔀 Combine")
                        .default_open(true)
                        .show(ui, |ui| {
                            if ui.button("Blend").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Blend);
                            }
                            if ui.button("Mask").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Mask);
                            }
                        });
                    
                    egui::CollapsingHeader::new("📤 Output")
                        .default_open(true)
                        .show(ui, |ui| {
                            if ui.button("Output").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Output);
                            }
                        });
                });
        }
        
        // Right panel: Properties + Output Preview
        if self.show_properties {
            egui::SidePanel::right("properties")
                .resizable(true)
                .default_width(280.0)
                .min_width(200.0)
                .show(ctx, |ui| {
                    ui.heading("Properties");
                    ui.separator();
                    
                    if let Some(node_id) = self.graph.selected_node() {
                        self.graph.show_node_properties(ui, node_id);
                    } else {
                        ui.label("Select a node to view properties");
                    }
                    
                    // Output Preview
                    if let Some(texture) = &self.output_texture {
                        ui.separator();
                        ui.heading("Output Preview");
                        
                        // Calculate scaled size to fit panel
                        let max_size = 250.0;
                        let aspect = texture.size[0] as f32 / texture.size[1] as f32;
                        let (w, h) = if aspect > 1.0 {
                            (max_size, max_size / aspect)
                        } else {
                            (max_size * aspect, max_size)
                        };
                        
                        ui.image(egui::ImageSource::Texture(egui::load::SizedTexture {
                            id: texture.handle.id(),
                            size: egui::vec2(w, h),
                        }));
                        
                        ui.label(format!("{}×{}", texture.size[0], texture.size[1]));
                        
                        // Export button
                        if ui.button("💾 Export PNG").clicked() {
                            self.export_output();
                        }
                    }
                });
        }
        
        // Central panel: The node graph canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            // This is where the magic happens - the entire graph is drawn here
            self.graph.show(ui);
        });
        
        // Handle dropped files
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(bytes) = &file.bytes {
                    self.load_image_bytes(ctx, bytes);
                }
            }
        });
        
        // Keyboard shortcut: R to run graph
        if ctx.input(|i| i.key_pressed(egui::Key::R) && !i.modifiers.ctrl && !i.modifiers.command) {
            self.run_graph(ctx);
        }
        
        // Request continuous repaint for smooth 60fps (game engine style)
        ctx.request_repaint();
    }
}
