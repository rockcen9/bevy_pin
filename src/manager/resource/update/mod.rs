use crate::prelude::*;

/// Mutate a single field within a resource by dot-separated path (e.g. `.deep_purple`).
pub fn mutate_resource_field(type_path: String, field_path: String, value: serde_json::Value, url: &str) {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "world.mutate_resources",
        "params": {
            "resource": type_path,
            "path": field_path,
            "value": value
        }
    });

    let request = ehttp::Request::post(url, serde_json::to_vec(&body).unwrap());

    ehttp::fetch(request, move |result| match result {
        Ok(r) => {
            if let Some(body) = r.text() {
                info!("mutate_resource_field response: {}", body);
            }
        }
        Err(e) => error!("mutate_resource_field failed: {}", e),
    });
}
