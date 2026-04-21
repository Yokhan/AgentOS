use crate::state::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

/// Proxy a request to n8n webhook endpoint — validated paths only
#[tauri::command]
pub async fn proxy_webhook(
    state: State<'_, Arc<AppState>>,
    path: String,
    method: String,
    body: Option<String>,
) -> Result<Value, String> {
    // Validate path — must start with /webhook/, no traversal
    if !path.starts_with("/webhook/") {
        return Err("Proxy only allows /webhook/ paths".to_string());
    }
    if path.contains("..") || path.contains("//") {
        return Err("Invalid proxy path".to_string());
    }

    // Validate method
    let method_upper = method.to_uppercase();
    if method_upper != "GET" && method_upper != "POST" {
        return Err(format!("Unsupported method: {}", method));
    }

    // Limit body size (1MB)
    if let Some(ref b) = body {
        if b.len() > 1_048_576 {
            return Err("Body too large (max 1MB)".to_string());
        }
    }

    let url = format!("{}{}", state.n8n_url, path);
    crate::log_info!("[proxy] {} {}", method_upper, path);

    let client = reqwest::Client::new();
    let request = match method_upper.as_str() {
        "GET" => client.get(&url),
        "POST" => {
            let mut req = client.post(&url);
            if let Some(b) = body {
                req = req.header("Content-Type", "application/json").body(b);
            }
            req
        }
        _ => unreachable!(),
    };

    match request.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            match serde_json::from_str::<Value>(&text) {
                Ok(data) => Ok(json!({"status": status, "data": data})),
                Err(_) => Ok(json!({"status": status, "data": text})),
            }
        }
        Err(e) => Ok(json!({"status": 502, "error": e.to_string()})),
    }
}
