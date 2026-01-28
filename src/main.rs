//! FlowNode Canvas - Full Rust/Wasm Node Editor
//!
//! This is the "game engine" approach to building a node-based image editor.
//! Everything is rendered to a single <canvas> element using egui + eframe.

#![warn(clippy::all)]

mod app;
mod nodes;
mod graph;

use app::FlowNodeApp;

// Native entry point
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("FlowNode - Canvas Editor"),
        ..Default::default()
    };
    
    eframe::run_native(
        "FlowNode",
        native_options,
        Box::new(|cc| Ok(Box::new(FlowNodeApp::new(cc)))),
    )
}

// WASM entry point
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect panics to console.error
    console_error_panic_hook::set_once();
    
    // Setup logging to browser console
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logger");
    
    let web_options = eframe::WebOptions::default();
    
    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Ok(Box::new(FlowNodeApp::new(cc)))),
            )
            .await;
        
        // Hide loading screen
        if let Some(window) = web_sys::window() {
            let _ = js_sys::Reflect::get(&window, &"hideLoading".into())
                .ok()
                .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
                .map(|f| f.call0(&window));
        }
        
        if let Err(e) = start_result {
            log::error!("Failed to start eframe: {:?}", e);
        }
    });
}
