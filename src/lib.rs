use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::{Headers, Request, Response, ResponseInit};

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    version: String,
    status: String,
}

#[wasm_bindgen]
pub struct RepoAnalyzerApi {}

#[wasm_bindgen]
impl RepoAnalyzerApi {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }
}

#[wasm_bindgen]
pub async fn fetch(request: Request) -> Result<Response, JsValue> {
    // Log request
    web_sys::console::log_1(&"Request received".into());

    // Parse URL
    let url = request.url();
    web_sys::console::log_1(&format!("URL: {}", url).into());

    // Create response headers
    let headers = Headers::new()?;
    headers.append("Content-Type", "application/json")?;

    // Create response data
    let data = ApiResponse {
        message: "Hello from Repo Analyzer API!".to_string(),
        version: "0.1.0".to_string(),
        status: "success".to_string(),
    };

    let json = serde_json::to_string(&data).unwrap();

    // Return response
    let mut init = ResponseInit::new();
    init.set_status(200);
    init.set_headers(&headers);

    Response::new_with_opt_str(&Some(json), &init)
}
