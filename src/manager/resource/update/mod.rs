use crate::prelude::*;

#[derive(Deserialize)]
pub struct MutateResourceResponse {
    pub result: serde_json::Value,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(BrpEndpointPlugin::<MutateResourceResponse>::default());
}

/// Mutate a single field within a resource by dot-separated path (e.g. `.deep_purple`).
pub fn mutate_resource_field(
    type_path: String,
    field_path: String,
    value: serde_json::Value,
    url: &str,
    commands: &mut Commands,
) {
    let payload = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "world.mutate_resources",
        "params": {
            "resource": type_path,
            "path": field_path,
            "value": value
        }
    }))
    .unwrap();

    commands
        .spawn(BrpRequest::<MutateResourceResponse>::new(url, payload))
        .observe(
            |trigger: On<Add, BrpResponse<MutateResourceResponse>>,
             query: Query<&BrpResponse<MutateResourceResponse>>,
             mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(body) => info!("mutate_resource_field response: {:?}", body.result),
                        Err(e) => error!("mutate_resource_field failed: {}", e),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(
            |trigger: On<Add, TimeoutError>, mut commands: Commands| {
                error!("mutate_resource_field request timed out");
                commands.entity(trigger.entity).despawn();
            },
        );
}
