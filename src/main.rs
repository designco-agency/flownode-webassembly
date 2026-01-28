//! FlowNode Canvas - Full Rust/Wasm Node Editor
//!
//! This is the "game engine" approach to building a node-based image editor.
//! Everything is rendered to a single <canvas> element using egui + eframe.

#![warn(clippy::all)]

mod app;
mod nodes;
mod graph;
mod ui_components;
mod compat;

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
    use eframe::wasm_bindgen::JsCast;
    
    // Redirect panics to console.error
    console_error_panic_hook::set_once();
    
    // Setup logging to browser console
    console_log::init_with_level(log::Level::Debug).expect("Failed to init logger");
    
    let web_options = eframe::WebOptions::default();
    
    wasm_bindgen_futures::spawn_local(async {
        // Get the canvas element
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");
        
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");
        
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(FlowNodeApp::new(cc)))),
            )
            .await;
        
        // Hide loading screen via JS
        let _ = js_sys::eval("document.getElementById('loading')?.classList.add('hidden')");
        
        if let Err(e) = start_result {
            log::error!("Failed to start eframe: {:?}", e);
        }
    });
}
