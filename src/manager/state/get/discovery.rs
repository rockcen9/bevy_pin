use super::variants::fetch_variants;
use super::{DiscoveredStates, StateEntry};
use crate::manager::connection::ServerUrl;
use crate::manager::state::ui::StatePanelsRoot;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(fetch_all_states);
}

fn fetch_all_states(
    _trigger: On<Add, StatePanelsRoot>,
    mut commands: Commands,
    server_url: Res<ServerUrl>,
) {
    debug!("Sending world.list_resources request");

    let req = commands.brp_list_resources(&server_url.0);
    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpListResources>>,
             query: Query<&RpcResponse<BrpListResources>>,
             mut states: ResMut<DiscoveredStates>,
             mut commands: Commands,
             server_url: Res<ServerUrl>| {
                let entity = trigger.entity;
                let Ok(response) = query.get(entity) else {
                    commands.entity(entity).despawn();
                    return;
                };

                if let Ok(data) = &response.data {
                    let all: Vec<&str> = data.result.iter().map(|s| s.as_str()).collect();

                    let discovered: Vec<(String, String, Option<String>)> = all
                        .iter()
                        .filter(|s| s.contains("bevy_state::state::resources::State<"))
                        .filter_map(|state_res| {
                            let inner = extract_inner_type(state_res)?;
                            let next = all
                                .iter()
                                .find(|s| s.contains("NextState") && s.contains(inner))
                                .map(|s| s.to_string());
                            Some((inner.to_string(), state_res.to_string(), next))
                        })
                        .collect();

                    debug!("Discovered {} state(s)", discovered.len());

                    for (type_path, state_resource, next_state_resource) in discovered {
                        if states.0.iter().any(|e| e.state_type_path == type_path) {
                            continue;
                        }
                        let label = type_path
                            .split("::")
                            .last()
                            .unwrap_or(&type_path)
                            .to_string();
                        fetch_variants(type_path.clone(), &mut commands, &server_url.0);
                        states.0.push(StateEntry {
                            label,
                            state_type_path: type_path,
                            state_resource,
                            next_state_resource,
                            current: None,
                            variants: vec![],
                        });
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

fn extract_inner_type(s: &str) -> Option<&str> {
    let start = s.find('<')? + 1;
    let end = s.rfind('>')?;
    Some(&s[start..end])
}
