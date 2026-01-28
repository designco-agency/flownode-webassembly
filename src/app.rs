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
    
    /// Clipboard for copy/paste
    clipboard: Option<crate::nodes::Node>,
    
    /// Output image from last execution
    output_image: Option<ImageData>,
    
    /// Output texture for display
    output_texture: Option<TextureHandle>,
    
    /// Status message for user feedback
    status_message: Option<(String, std::time::Instant)>,
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
        
        let mut app = Self {
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
            clipboard: None,
            status_message: None,
        };
        
        // Try to load saved workflow from local storage on startup
        app.load_from_local_storage();
        
        app
    }
    
    /// Set a status message that auto-expires after 3 seconds
    fn set_status(&mut self, message: &str) {
        self.status_message = Some((message.to_string(), std::time::Instant::now()));
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
                
                let msg = format!("✓ Loaded image {}×{}", image_data.width, image_data.height);
                log::info!("{}", msg);
                
                // Store the image and texture
                self.images.insert(image_id, image_data);
                self.textures.insert(image_id, texture);
                
                self.set_status(&msg);
                
                // Assign to selected node if it's an Image node, otherwise create new
                if let Some(node_id) = self.graph.selected_node() {
                    if self.graph.set_node_image(node_id, image_id) {
                        log::info!("Assigned image to selected node");
                        return;
                    }
                }
                
                // Create a new ImageInput node with this image
                let node_id = self.graph.add_node(crate::nodes::NodeType::Image);
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
    
    /// Handle keyboard shortcuts - matches React app
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        use crate::nodes::NodeType;
        
        ctx.input(|i| {
            // Only handle shortcuts when no text input is focused
            // Note: Could use i.events to check for text focus, but skipping for now
            
            let ctrl = i.modifiers.ctrl || i.modifiers.command;
            let shift = i.modifiers.shift;
            
            // === Editor shortcuts ===
            
            // Ctrl+G = Run graph (like Execute in React)
            if ctrl && i.key_pressed(egui::Key::G) && !shift {
                // Will call run_graph after this closure
            }
            
            // Ctrl+C = Copy selected node
            if ctrl && i.key_pressed(egui::Key::C) {
                if let Some(node) = self.graph.get_selected_node_clone() {
                    self.clipboard = Some(node);
                    log::info!("Copied node to clipboard");
                }
            }
            
            // Ctrl+V = Paste node
            if ctrl && i.key_pressed(egui::Key::V) {
                if let Some(ref node) = self.clipboard.clone() {
                    self.graph.paste_node(node);
                    log::info!("Pasted node from clipboard");
                }
            }
            
            // Ctrl+D = Duplicate selected node
            if ctrl && i.key_pressed(egui::Key::D) {
                if let Some(node) = self.graph.get_selected_node_clone() {
                    self.graph.paste_node(&node);
                    log::info!("Duplicated node");
                }
            }
            
            // Escape = Deselect / cancel
            if i.key_pressed(egui::Key::Escape) {
                self.graph.deselect_all();
            }
            
            // Delete/Backspace = Delete selected
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                self.graph.delete_selected();
            }
            
            // === Node creation shortcuts (single key, no modifiers) ===
            if !ctrl && !shift {
                // Content nodes
                if i.key_pressed(egui::Key::I) { self.graph.add_node(NodeType::Image); }
                if i.key_pressed(egui::Key::K) { self.graph.add_node(NodeType::Content); }
                if i.key_pressed(egui::Key::B) { self.graph.add_node(NodeType::Bucket); }
                
                // Editing nodes
                if i.key_pressed(egui::Key::A) { self.graph.add_node(NodeType::Adjust); }
                if i.key_pressed(egui::Key::E) { self.graph.add_node(NodeType::Effects); }
                if i.key_pressed(egui::Key::C) { self.graph.add_node(NodeType::Compare); }
                if i.key_pressed(egui::Key::F) { self.graph.add_node(NodeType::Composition); }
                
                // Text nodes
                if i.key_pressed(egui::Key::T) { self.graph.add_node(NodeType::Text); }
                if i.key_pressed(egui::Key::J) { self.graph.add_node(NodeType::Concat); }
                if i.key_pressed(egui::Key::S) { self.graph.add_node(NodeType::Splitter); }
                if i.key_pressed(egui::Key::N) { self.graph.add_node(NodeType::Postit); }
                
                // Utility nodes
                if i.key_pressed(egui::Key::R) { self.graph.add_node(NodeType::Router); }
                if i.key_pressed(egui::Key::Q) { self.graph.add_node(NodeType::Batch); }
                if i.key_pressed(egui::Key::H) { self.graph.add_node(NodeType::Title); }
                
                // AI nodes
                if i.key_pressed(egui::Key::O) { self.graph.add_node(NodeType::Omni); }
                if i.key_pressed(egui::Key::L) { self.graph.add_node(NodeType::Llm); }
                if i.key_pressed(egui::Key::D) { self.graph.add_node(NodeType::Video); }
                if i.key_pressed(egui::Key::U) { self.graph.add_node(NodeType::Upscaler); }
                if i.key_pressed(egui::Key::V) { self.graph.add_node(NodeType::Vector); }
                if i.key_pressed(egui::Key::M) { self.graph.add_node(NodeType::MindMap); }
                if i.key_pressed(egui::Key::Num3) { self.graph.add_node(NodeType::Rodin3d); }
            }
        });
        
        // Run graph with Ctrl+G
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::G)) {
            self.run_graph(ctx);
        }
        
        // Ctrl+S = Save to local storage
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::S)) {
            self.save_to_local_storage();
        }
    }
    
    /// Save workflow to browser's local storage
    fn save_to_local_storage(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(json) = self.graph.to_json() {
                let js_code = format!(
                    r#"
                    try {{
                        localStorage.setItem('flownode_workflow', `{}`);
                        console.log('Saved workflow to localStorage');
                    }} catch(e) {{
                        console.error('Failed to save:', e);
                    }}
                    "#,
                    json.replace('`', "\\`").replace("${", "\\${")
                );
                let _ = js_sys::eval(&js_code);
                self.set_status("✓ Saved to browser storage");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            log::info!("Local storage save only available in browser");
            self.set_status("⚠ Save only available in browser");
        }
    }
    
    /// Load workflow from browser's local storage
    fn load_from_local_storage(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            let js_code = r#"
                localStorage.getItem('flownode_workflow') || ''
            "#;
            if let Ok(result) = js_sys::eval(js_code) {
                if let Some(json_str) = result.as_string() {
                    if !json_str.is_empty() {
                        match crate::graph::NodeGraph::from_json(&json_str) {
                            Ok(graph) => {
                                self.graph = graph;
                                self.set_status("✓ Loaded from browser storage");
                                log::info!("Loaded workflow from localStorage");
                            }
                            Err(e) => {
                                log::error!("Failed to parse saved workflow: {:?}", e);
                                self.set_status("⚠ Failed to load saved workflow");
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Run the node graph and produce output
    fn run_graph(&mut self, ctx: &egui::Context) {
        log::info!("Running node graph...");
        let start = std::time::Instant::now();
        
        match self.executor.execute(&self.graph, &self.images) {
            Ok(Some(output)) => {
                let elapsed = start.elapsed();
                let msg = format!("✓ Processed {}×{} in {:.0}ms", output.width, output.height, elapsed.as_secs_f64() * 1000.0);
                log::info!("{}", msg);
                
                // Create texture for display
                let texture = TextureHandle::from_image_data(
                    ctx,
                    "output",
                    &output,
                );
                
                self.output_texture = Some(texture);
                self.output_image = Some(output);
                self.set_status(&msg);
            }
            Ok(None) => {
                self.set_status("⚠ No output (connect nodes to Output)");
                log::info!("Execution complete, no output (no output node connected)");
            }
            Err(e) => {
                self.set_status(&format!("✗ Error: {}", e));
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
                    if ui.button("💾 Save to Browser (Ctrl+S)").clicked() {
                        self.save_to_local_storage();
                        ui.close_menu();
                    }
                    if ui.button("📂 Load from Browser").clicked() {
                        self.load_from_local_storage();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export Image...").clicked() {
                        self.export_output();
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
                    
                    // Node library organized like React app
                    egui::CollapsingHeader::new("📷 Content")
                        .default_open(true)
                        .show(ui, |ui| {
                            if ui.button("Image (I)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Image);
                            }
                            if ui.button("Content (K)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Content);
                            }
                            if ui.button("Bucket (B)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Bucket);
                            }
                        });
                    
                    egui::CollapsingHeader::new("🎨 Editing")
                        .default_open(true)
                        .show(ui, |ui| {
                            if ui.button("Adjust (A)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Adjust);
                            }
                            if ui.button("Effects (E)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Effects);
                            }
                            if ui.button("Compare (C)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Compare);
                            }
                            if ui.button("Composition (F)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Composition);
                            }
                        });
                    
                    egui::CollapsingHeader::new("📝 Text")
                        .default_open(false)
                        .show(ui, |ui| {
                            if ui.button("Text (T)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Text);
                            }
                            if ui.button("Concat (J)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Concat);
                            }
                            if ui.button("Splitter (S)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Splitter);
                            }
                            if ui.button("Post-It (N)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Postit);
                            }
                        });
                    
                    egui::CollapsingHeader::new("🔧 Utility")
                        .default_open(false)
                        .show(ui, |ui| {
                            if ui.button("Router (R)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Router);
                            }
                            if ui.button("Batch (Q)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Batch);
                            }
                            if ui.button("Title (H)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Title);
                            }
                        });
                    
                    egui::CollapsingHeader::new("🤖 AI Generation")
                        .default_open(false)
                        .show(ui, |ui| {
                            if ui.button("Omni (O)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Omni);
                            }
                            if ui.button("LLM (L)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Llm);
                            }
                            if ui.button("Video (D)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Video);
                            }
                            if ui.button("Upscaler (U)").clicked() {
                                self.graph.add_node(crate::nodes::NodeType::Upscaler);
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
        
        // Keyboard shortcuts - matching React app exactly
        self.handle_keyboard_shortcuts(ctx);
        
        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Status message (expires after 3 seconds)
                if let Some((msg, time)) = &self.status_message {
                    if time.elapsed().as_secs() < 3 {
                        ui.label(egui::RichText::new(msg).color(egui::Color32::LIGHT_GREEN));
                    }
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("A=Adjust E=Effects | Ctrl+C/V=Copy/Paste | Ctrl+G=Run");
                });
            });
        });
        
        // Request continuous repaint for smooth 60fps (game engine style)
        ctx.request_repaint();
    }
}
