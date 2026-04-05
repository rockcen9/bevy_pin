use super::DiscoveredStates;
use crate::prelude::*;

#[derive(Deserialize)]
struct SchemaResponse {
    result: serde_json::Value,
}

#[derive(Component)]
struct VariantsContext(String);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(BrpEndpointPlugin::<SchemaResponse>::default());
}

pub(super) fn fetch_variants(type_path: String, commands: &mut Commands, url: &str) {
    let crate_name = type_path.split("::").next().unwrap_or("").to_string();
    debug!("Sending registry.schema request for crate: {}", crate_name);

    let payload = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "registry.schema",
        "params": { "with_crates": [crate_name] }
    }))
    .unwrap();

    commands
        .spawn((
            BrpRequest::<SchemaResponse>::new(url, payload),
            VariantsContext(type_path),
        ))
        .observe(
            |trigger: On<Add, BrpResponse<SchemaResponse>>,
             query: Query<(&BrpResponse<SchemaResponse>, &VariantsContext)>,
             mut states: ResMut<DiscoveredStates>,
             mut commands: Commands| {
                let entity = trigger.entity;
                let Ok((response, ctx)) = query.get(entity) else {
                    commands.entity(entity).despawn();
                    return;
                };
                let type_path = ctx.0.clone();
                if let Ok(data) = &response.data {
                    let schema = &data.result[&type_path];
                    if let Some(one_of) = schema["oneOf"].as_array() {
                        let variants: Vec<String> = one_of
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        if !variants.is_empty() {
                            if let Some(entry) =
                                states.0.iter_mut().find(|e| e.state_type_path == type_path)
                            {
                                entry.variants = variants;
                            }
                        }
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(
            |trigger: On<Add, TimeoutError>, mut commands: Commands| {
                commands.entity(trigger.entity).despawn();
            },
        );
}
