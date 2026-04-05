use super::{DiscoveredResources, ResourceEntry, ResourceScreenRoot};
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Deserialize)]
struct ListResourcesResponse {
    result: Vec<String>,
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(BrpEndpointPlugin::<ListResourcesResponse>::default())
        .add_observer(on_add_resource_screen_root);
}

fn on_add_resource_screen_root(
    _trigger: On<Add, ResourceScreenRoot>,
    mut commands: Commands,
    server_url: Res<ServerUrl>,
) {
    let payload = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "world.list_resources",
        "params": null
    }))
    .unwrap();

    debug!("Sending world.list_resources request");

    commands
        .spawn(BrpRequest::<ListResourcesResponse>::new(
            &server_url.0,
            payload,
        ))
        .observe(
            |trigger: On<Add, BrpResponse<ListResourcesResponse>>,
             query: Query<&BrpResponse<ListResourcesResponse>>,
             mut resources: ResMut<DiscoveredResources>,
             mut commands: Commands| {
                let entity = trigger.entity;
                let Ok(response) = query.get(entity) else {
                    commands.entity(entity).despawn();
                    return;
                };

                if let Ok(data) = &response.data {
                    let total = data.result.len();
                    let discovered: Vec<String> = data
                        .result
                        .iter()
                        .filter(|s| !s.starts_with("bevy_"))
                        .cloned()
                        .collect();

                    debug!(
                        "world.list_resources: {} total, {} user resources found",
                        total,
                        discovered.len()
                    );

                    for type_path in discovered {
                        if resources.0.iter().any(|e| e.type_path == type_path) {
                            continue;
                        }
                        let label = type_path
                            .split("::")
                            .last()
                            .unwrap_or(&type_path)
                            .to_string();
                        debug!("Discovered resource: {}", type_path);
                        resources.0.push(ResourceEntry {
                            label,
                            type_path,
                            value: None,
                        });
                    }
                } else {
                    debug!("world.list_resources request failed");
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
