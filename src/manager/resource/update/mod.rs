use crate::prelude::*;

pub fn plugin(_app: &mut App) {}

/// Mutate a single field within a resource by dot-separated path (e.g. `.deep_purple`).
pub fn mutate_resource_field(
    type_path: String,
    field_path: String,
    value: serde_json::Value,
    url: &str,
    commands: &mut Commands,
) {
    let req = commands.brp_mutate_resource(url, &type_path, &field_path, value);
    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpMutate>>,
             query: Query<&RpcResponse<BrpMutate>>,
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
