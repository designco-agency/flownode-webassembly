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
    
    /// Waiting for cloud load to complete
    cloud_load_pending: bool,
    
    /// Waiting for cloud save to complete
    cloud_save_pending: bool,
    
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
            cloud_load_pending: false,
            cloud_save_pending: false,
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
    
    /// Save workflow to Supabase cloud as a NEW workflow
    fn save_to_cloud_new(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            use crate::cloud::{SUPABASE_URL, SUPABASE_ANON_KEY};
            
            // Convert graph to React Flow format
            let workflow_json = self.convert_to_react_flow();
            
            // Generate new UUID for the workflow
            let new_id = uuid::Uuid::new_v4().to_string();
            let timestamp = js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default();
            
            // Build the insert payload
            let payload = serde_json::json!({
                "id": new_id,
                "name": format!("WASM Export {}", &timestamp[0..10]),
                "nodes": workflow_json.get("nodes").unwrap_or(&serde_json::Value::Null),
                "edges": workflow_json.get("edges").unwrap_or(&serde_json::Value::Null),
                "viewport": { "x": 0, "y": 0, "zoom": 1 },
                "is_public": false,
                "user_email": "wasm@flownode.io",
                "created_at": timestamp.clone(),
                "updated_at": timestamp
            });
            
            let payload_str = serde_json::to_string(&payload).unwrap_or_default();
            
            let js_code = format!(
                r#"
                (async () => {{
                    try {{
                        const resp = await fetch('{}/rest/v1/workflows', {{
                            method: 'POST',
                            headers: {{
                                'apikey': '{}',
                                'Authorization': 'Bearer {}',
                                'Content-Type': 'application/json',
                                'Prefer': 'return=representation'
                            }},
                            body: '{}'
                        }});
                        if (!resp.ok) {{
                            const err = await resp.text();
                            console.error('Save failed:', resp.status, err);
                            window.__flownode_save_result = 'error:' + resp.status;
                        }} else {{
                            const data = await resp.json();
                            console.log('Saved workflow:', data);
                            window.__flownode_save_result = 'ok:' + (data[0]?.id || '{}');
                        }}
                    }} catch(e) {{
                        console.error('Save error:', e);
                        window.__flownode_save_result = 'error:' + e.message;
                    }}
                }})();
                "#,
                SUPABASE_URL, SUPABASE_ANON_KEY, SUPABASE_ANON_KEY, 
                payload_str.replace('\\', "\\\\").replace('\'', "\\'"),
                new_id
            );
            let _ = js_sys::eval(&js_code);
            self.set_status("☁️ Saving to cloud...");
            self.cloud_save_pending = true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.set_status("⚠ Cloud sync only in browser");
        }
    }
    
    /// Convert internal graph to React Flow JSON format
    fn convert_to_react_flow(&self) -> serde_json::Value {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Convert nodes
        for (id, node) in &self.graph.nodes {
            let node_type = match node.node_type {
                crate::nodes::NodeType::Image => "image",
                crate::nodes::NodeType::Content => "content",
                crate::nodes::NodeType::Bucket => "bucket",
                crate::nodes::NodeType::Adjust => "adjust",
                crate::nodes::NodeType::Effects => "effects",
                crate::nodes::NodeType::Text => "text",
                crate::nodes::NodeType::Concat => "concat",
                crate::nodes::NodeType::Splitter => "splitter",
                crate::nodes::NodeType::Postit => "postit",
                crate::nodes::NodeType::Compare => "compare",
                crate::nodes::NodeType::Composition => "composition",
                crate::nodes::NodeType::Router => "router",
                crate::nodes::NodeType::Batch => "batch",
                crate::nodes::NodeType::Title => "title",
                crate::nodes::NodeType::Omni => "omni",
                crate::nodes::NodeType::Llm => "llm",
                crate::nodes::NodeType::Video => "video",
                crate::nodes::NodeType::Upscaler => "upscaler",
                crate::nodes::NodeType::Vector => "vector",
                crate::nodes::NodeType::Rodin3d => "rodin3d",
                crate::nodes::NodeType::MindMap => "mind-map",
                _ => "image",
            };
            
            let data = self.node_to_react_data(node);
            
            nodes.push(serde_json::json!({
                "id": id.to_string(),
                "type": node_type,
                "position": {
                    "x": node.position.x,
                    "y": node.position.y
                },
                "data": data
            }));
        }
        
        // Convert connections to edges
        for conn in &self.graph.connections {
            edges.push(serde_json::json!({
                "id": format!("e{}-{}", conn.from_node, conn.to_node),
                "source": conn.from_node.to_string(),
                "target": conn.to_node.to_string(),
                "sourceHandle": format!("output-{}", conn.from_slot),
                "targetHandle": format!("input-{}", conn.to_slot)
            }));
        }
        
        serde_json::json!({
            "nodes": nodes,
            "edges": edges
        })
    }
    
    /// Convert node properties to React Flow data format
    fn node_to_react_data(&self, node: &crate::nodes::Node) -> serde_json::Value {
        match &node.properties {
            crate::nodes::NodeProperties::Adjust { 
                brightness, contrast, saturation, exposure,
                highlights, shadows, temperature, tint,
                vibrance, gamma, ..
            } => {
                serde_json::json!({
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
                        "gamma": gamma
                    }
                })
            }
            crate::nodes::NodeProperties::Effects {
                gaussian_blur, sharpen, grain, vignette,
                directional_blur, directional_blur_angle,
                progressive_blur, progressive_blur_direction, progressive_blur_falloff,
                glass_blinds, glass_blinds_frequency, glass_blinds_angle, glass_blinds_phase,
                grain_size, grain_monochrome, grain_seed,
                vignette_roundness, vignette_smoothness,
            } => {
                serde_json::json!({
                    "settings": {
                        "gaussianBlur": gaussian_blur,
                        "sharpen": sharpen,
                        "grain": grain,
                        "grainSize": grain_size,
                        "grainMonochrome": grain_monochrome,
                        "grainSeed": grain_seed,
                        "vignette": vignette,
                        "vignetteRoundness": vignette_roundness,
                        "vignetteSmoothness": vignette_smoothness,
                        "directionalBlur": directional_blur,
                        "directionalBlurAngle": directional_blur_angle,
                        "progressiveBlur": progressive_blur,
                        "progressiveBlurDirection": format!("{:?}", progressive_blur_direction),
                        "progressiveBlurFalloff": progressive_blur_falloff,
                        "glassBlinds": glass_blinds,
                        "glassBlindsFrequency": glass_blinds_frequency,
                        "glassBlindsAngle": glass_blinds_angle,
                        "glassBlindsPhase": glass_blinds_phase
                    }
                })
            }
            crate::nodes::NodeProperties::Text { text } => {
                serde_json::json!({ "text": text })
            }
            crate::nodes::NodeProperties::Omni { prompt, negative_prompt, model, .. } => {
                serde_json::json!({
                    "prompt": prompt,
                    "negativePrompt": negative_prompt,
                    "model": model
                })
            }
            _ => serde_json::json!({})
        }
    }
    
    /// Check if cloud save completed
    fn check_cloud_save(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if !self.cloud_save_pending {
                return;
            }
            
            if let Ok(result) = js_sys::eval("window.__flownode_save_result || ''") {
                if let Some(result_str) = result.as_string() {
                    if !result_str.is_empty() {
                        self.cloud_save_pending = false;
                        
                        if result_str.starts_with("ok:") {
                            let id = &result_str[3..];
                            self.set_status(&format!("✓ Saved to cloud! ID: {}...", &id[..8.min(id.len())]));
                            log::info!("Workflow saved with ID: {}", id);
                        } else {
                            self.set_status(&format!("✗ Save failed: {}", &result_str[6..]));
                        }
                        
                        let _ = js_sys::eval("window.__flownode_save_result = null");
                    }
                }
            }
        }
    }
    
    /// Load test workflow from Supabase cloud (read-only test)
    fn load_from_cloud_test(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            use crate::cloud::{SUPABASE_URL, SUPABASE_ANON_KEY, TEST_WORKFLOW_ID};
            
            // Use JS fetch for simplicity (async in sync context)
            let js_code = format!(
                r#"
                (async () => {{
                    try {{
                        const resp = await fetch('{}/rest/v1/workflows?id=eq.{}&select=*', {{
                            headers: {{
                                'apikey': '{}',
                                'Authorization': 'Bearer {}',
                                'Content-Type': 'application/json'
                            }}
                        }});
                        if (!resp.ok) throw new Error('HTTP ' + resp.status);
                        const data = await resp.json();
                        if (data.length > 0) {{
                            console.log('Loaded workflow:', data[0].name);
                            // Store in window for Rust to read
                            window.__flownode_cloud_data = JSON.stringify({{
                                name: data[0].name,
                                nodes: data[0].nodes,
                                edges: data[0].edges,
                                viewport: data[0].viewport
                            }});
                            console.log('Stored workflow data for WASM');
                        }} else {{
                            window.__flownode_cloud_data = null;
                            console.error('Workflow not found');
                        }}
                    }} catch(e) {{
                        window.__flownode_cloud_data = null;
                        console.error('Cloud load failed:', e);
                    }}
                }})();
                "#,
                SUPABASE_URL, TEST_WORKFLOW_ID, SUPABASE_ANON_KEY, SUPABASE_ANON_KEY
            );
            let _ = js_sys::eval(&js_code);
            self.set_status("☁️ Loading from cloud...");
            self.cloud_load_pending = true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.set_status("⚠ Cloud sync only in browser");
        }
    }
    
    /// Check if cloud data is ready and process it
    fn check_cloud_load(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if !self.cloud_load_pending {
                return;
            }
            
            // Check if JS has loaded the data
            if let Ok(result) = js_sys::eval("window.__flownode_cloud_data || ''") {
                if let Some(json_str) = result.as_string() {
                    if !json_str.is_empty() {
                        self.cloud_load_pending = false;
                        
                        // Parse and convert the cloud data
                        match self.convert_cloud_workflow(&json_str) {
                            Ok((node_count, edge_count, name)) => {
                                self.set_status(&format!("✓ Loaded: {} ({} nodes, {} edges)", 
                                    name, node_count, edge_count));
                            }
                            Err(e) => {
                                log::error!("Failed to convert workflow: {}", e);
                                self.set_status(&format!("✗ Load failed: {}", e));
                            }
                        }
                        
                        // Clear the JS data
                        let _ = js_sys::eval("window.__flownode_cloud_data = null");
                    }
                }
            }
        }
    }
    
    /// Convert React Flow workflow JSON to our internal format
    fn convert_cloud_workflow(&mut self, json_str: &str) -> Result<(usize, usize, String), String> {
        let cloud_data: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        
        let name = cloud_data.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("Untitled")
            .to_string();
        
        let nodes = cloud_data.get("nodes")
            .and_then(|n| n.as_array())
            .ok_or("No nodes array")?;
        
        let empty_edges = vec![];
        let edges = cloud_data.get("edges")
            .and_then(|e| e.as_array())
            .unwrap_or(&empty_edges);
        
        // Clear current graph
        self.graph = crate::graph::NodeGraph::new();
        
        // Map of old IDs to new UUIDs
        let mut id_map: std::collections::HashMap<String, uuid::Uuid> = std::collections::HashMap::new();
        
        // Convert nodes
        for node in nodes {
            let old_id = node.get("id").and_then(|i| i.as_str()).unwrap_or("");
            let node_type_str = node.get("type").and_then(|t| t.as_str()).unwrap_or("image");
            let x = node.get("position")
                .and_then(|p| p.get("x"))
                .and_then(|v| v.as_f64())
                .unwrap_or(100.0) as f32;
            let y = node.get("position")
                .and_then(|p| p.get("y"))
                .and_then(|v| v.as_f64())
                .unwrap_or(100.0) as f32;
            
            // Convert React Flow type to our NodeType
            let node_type = match node_type_str {
                "image" => crate::nodes::NodeType::Image,
                "content" => crate::nodes::NodeType::Content,
                "bucket" => crate::nodes::NodeType::Bucket,
                "adjust" => crate::nodes::NodeType::Adjust,
                "effects" => crate::nodes::NodeType::Effects,
                "text" => crate::nodes::NodeType::Text,
                "concat" => crate::nodes::NodeType::Concat,
                "splitter" => crate::nodes::NodeType::Splitter,
                "postit" => crate::nodes::NodeType::Postit,
                "compare" => crate::nodes::NodeType::Compare,
                "composition" => crate::nodes::NodeType::Composition,
                "router" => crate::nodes::NodeType::Router,
                "batch" => crate::nodes::NodeType::Batch,
                "title" => crate::nodes::NodeType::Title,
                "omni" => crate::nodes::NodeType::Omni,
                "llm" => crate::nodes::NodeType::Llm,
                "video" => crate::nodes::NodeType::Video,
                "upscaler" => crate::nodes::NodeType::Upscaler,
                "vector" => crate::nodes::NodeType::Vector,
                "rodin3d" => crate::nodes::NodeType::Rodin3d,
                "mind-map" => crate::nodes::NodeType::MindMap,
                _ => {
                    log::warn!("Unknown node type: {}, defaulting to Image", node_type_str);
                    crate::nodes::NodeType::Image
                }
            };
            
            // Create node at the correct position
            let mut new_node = crate::nodes::Node::new(node_type, egui::Vec2::new(x, y));
            
            // Try to extract and apply node data/settings
            if let Some(data) = node.get("data") {
                self.apply_node_data(&mut new_node, data);
            }
            
            let new_id = new_node.id;
            id_map.insert(old_id.to_string(), new_id);
            self.graph.insert_node(new_node);
        }
        
        // Convert edges to connections
        for edge in edges {
            let source = edge.get("source").and_then(|s| s.as_str()).unwrap_or("");
            let target = edge.get("target").and_then(|t| t.as_str()).unwrap_or("");
            
            // Parse handle indices (format: "output-0", "input-0")
            let source_slot = edge.get("sourceHandle")
                .and_then(|h| h.as_str())
                .and_then(|h| h.strip_prefix("output-"))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            
            let target_slot = edge.get("targetHandle")
                .and_then(|h| h.as_str())
                .and_then(|h| h.strip_prefix("input-"))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            
            if let (Some(&from_id), Some(&to_id)) = (id_map.get(source), id_map.get(target)) {
                self.graph.add_connection(from_id, source_slot, to_id, target_slot);
            }
        }
        
        // Apply viewport if present
        if let Some(viewport) = cloud_data.get("viewport") {
            let _x = viewport.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let _y = viewport.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let _zoom = viewport.get("zoom").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            // TODO: Apply viewport to graph
        }
        
        log::info!("Converted {} nodes and {} edges", nodes.len(), edges.len());
        Ok((nodes.len(), edges.len(), name))
    }
    
    /// Apply React Flow node data to our node properties
    fn apply_node_data(&self, node: &mut crate::nodes::Node, data: &serde_json::Value) {
        match &mut node.properties {
            crate::nodes::NodeProperties::Adjust { 
                brightness, contrast, saturation, exposure,
                highlights, shadows, temperature, tint,
                vibrance, gamma, ..
            } => {
                if let Some(settings) = data.get("settings") {
                    *brightness = settings.get("brightness").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *contrast = settings.get("contrast").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *saturation = settings.get("saturation").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *exposure = settings.get("exposure").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *highlights = settings.get("highlights").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *shadows = settings.get("shadows").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *temperature = settings.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *tint = settings.get("tint").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *vibrance = settings.get("vibrance").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *gamma = settings.get("gamma").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                }
            }
            crate::nodes::NodeProperties::Effects {
                gaussian_blur, sharpen, grain, vignette, ..
            } => {
                if let Some(settings) = data.get("settings") {
                    *gaussian_blur = settings.get("gaussianBlur").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *sharpen = settings.get("sharpen").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *grain = settings.get("grain").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    *vignette = settings.get("vignette").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                }
            }
            crate::nodes::NodeProperties::Text { text } => {
                if let Some(t) = data.get("text").and_then(|t| t.as_str()) {
                    *text = t.to_string();
                }
            }
            crate::nodes::NodeProperties::Omni { prompt, negative_prompt, model, .. } => {
                if let Some(p) = data.get("prompt").and_then(|p| p.as_str()) {
                    *prompt = p.to_string();
                }
                if let Some(np) = data.get("negativePrompt").and_then(|p| p.as_str()) {
                    *negative_prompt = np.to_string();
                }
                if let Some(m) = data.get("model").and_then(|m| m.as_str()) {
                    *model = m.to_string();
                }
            }
            _ => {
                // Other node types - use defaults
            }
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
        // Check for pending cloud operations
        self.check_cloud_load();
        self.check_cloud_save();
        
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
                    ui.label(egui::RichText::new("☁️ Cloud").weak());
                    if ui.button("Load Test Workflow").clicked() {
                        self.load_from_cloud_test();
                        ui.close_menu();
                    }
                    if ui.button("💾 Save to Cloud (New)").clicked() {
                        self.save_to_cloud_new();
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
