use crate::prelude::*;

pub const LIST_COMPONENTS: &str = "world.list_components";
pub const GET_COMPONENTS: &str = "world.get_components";
pub const WORLD_QUERY: &str = "world.query";

/// Build the payload for `world.list_components`.
pub fn list_components_payload(entity_id: u64) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": LIST_COMPONENTS,
        "params": { "entity": entity_id }
    }))?)
}

/// Build the payload for `world.get_components`.
///
/// `components` should be a JSON array value, e.g. `json!(type_paths)` or
/// `json!([single_type_path])`.
pub fn get_components_payload(
    entity_id: u64,
    components: serde_json::Value,
    strict: bool,
) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": GET_COMPONENTS,
        "params": {
            "entity": entity_id,
            "components": components,
            "strict": strict
        }
    }))?)
}
