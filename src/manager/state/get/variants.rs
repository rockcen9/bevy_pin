use super::DiscoveredStates;
use crate::prelude::*;

#[derive(Component)]
struct VariantsContext(String);

pub(super) fn plugin(_app: &mut App) {}

pub(super) fn fetch_variants(type_path: String, commands: &mut Commands, url: &str) {
    let crate_name = type_path.split("::").next().unwrap_or("").to_string();
    debug!("Sending registry.schema request for crate: {}", crate_name);

    let req = commands.brp_registry_schema(url, json!({ "with_crates": [crate_name] }));
    commands
        .entity(req)
        .insert(VariantsContext(type_path))
        .observe(
            |trigger: On<Add, RpcResponse<BrpSchema>>,
             query: Query<(&RpcResponse<BrpSchema>, &VariantsContext)>,
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
