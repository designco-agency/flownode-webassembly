//! Supabase cloud integration for FlowNode
//!
//! Provides workflow load/save from FlowNode.io cloud.
//! Uses the same Supabase backend as the React app.

use serde::{Deserialize, Serialize};

/// Supabase configuration
pub const SUPABASE_URL: &str = "https://wduhlhemvdifvkqzedky.supabase.co";
pub const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6IndkdWhsaGVtdmRpZnZrcXplZGt5Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQ3OTc5MTksImV4cCI6MjA4MDM3MzkxOX0.E-Aemwm33OCYaMpPrr1GfTluHA6KY6MVEgClEqIO68I";

/// Workflow data from Supabase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudWorkflow {
    pub id: String,
    pub name: String,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    #[serde(default)]
    pub gallery: Option<serde_json::Value>,
    #[serde(default)]
    pub viewport: Option<Viewport>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub user_email: Option<String>,
    #[serde(default)]
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

/// Load a workflow from Supabase by ID
/// Returns the raw JSON response
#[cfg(target_arch = "wasm32")]
pub async fn load_workflow(workflow_id: &str) -> Result<CloudWorkflow, String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response, Headers};
    
    let url = format!(
        "{}/rest/v1/workflows?id=eq.{}&select=*",
        SUPABASE_URL, workflow_id
    );
    
    // Create request with headers
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    
    let headers = Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
    headers.set("apikey", SUPABASE_ANON_KEY).map_err(|e| format!("Failed to set apikey: {:?}", e))?;
    headers.set("Authorization", &format!("Bearer {}", SUPABASE_ANON_KEY)).map_err(|e| format!("Failed to set auth: {:?}", e))?;
    headers.set("Content-Type", "application/json").map_err(|e| format!("Failed to set content-type: {:?}", e))?;
    opts.headers(&headers);
    
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    let window = web_sys::window().ok_or("No window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    
    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "Response conversion failed")?;
    
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }
    
    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse failed: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON await failed: {:?}", e))?;
    
    // Parse as array (Supabase returns array even for single item)
    let json_str = js_sys::JSON::stringify(&json)
        .map_err(|_| "JSON stringify failed")?
        .as_string()
        .ok_or("JSON to string failed")?;
    
    let workflows: Vec<CloudWorkflow> = serde_json::from_str(&json_str)
        .map_err(|e| format!("Parse failed: {}", e))?;
    
    workflows.into_iter().next().ok_or("Workflow not found".to_string())
}

/// Non-WASM stub
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_workflow(_workflow_id: &str) -> Result<CloudWorkflow, String> {
    Err("Cloud sync only available in browser".to_string())
}

/// Test workflow ID for development
pub const TEST_WORKFLOW_ID: &str = "8f8bbdbb-b717-41ed-b894-95ea71f17cdc";
