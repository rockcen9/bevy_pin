use super::{DiscoveredResources, ResourceEntry, ResourceScreenRoot};
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

const BEVY_ENGINE_CRATES: &[&str] = &[
    "bevy_a11y",
    "bevy_animation",
    "bevy_app",
    "bevy_asset",
    "bevy_audio",
    "bevy_camera",
    "bevy_color",
    "bevy_core",
    "bevy_core_pipeline",
    "bevy_dev_tools",
    "bevy_diagnostic",
    "bevy_ecs",
    "bevy_gilrs",
    "bevy_gizmos",
    "bevy_gizmos_render",
    "bevy_hierarchy",
    "bevy_image",
    "bevy_input",
    "bevy_input_focus",
    "bevy_light",
    "bevy_log",
    "bevy_math",
    "bevy_mesh",
    "bevy_pbr",
    "bevy_picking",
    "bevy_platform",
    "bevy_reflect",
    "bevy_remote",
    "bevy_render",
    "bevy_scene",
    "bevy_sprite",
    "bevy_sprite_render",
    "bevy_state",
    "bevy_tasks",
    "bevy_text",
    "bevy_time",
    "bevy_transform",
    "bevy_ui",
    "bevy_ui_render",
    "bevy_utils",
    "bevy_window",
    "bevy_winit",
];

fn is_bevy_engine_crate(crate_name: &str) -> bool {
    BEVY_ENGINE_CRATES.contains(&crate_name)
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_add_resource_screen_root);
}

fn on_add_resource_screen_root(
    _trigger: On<Add, ResourceScreenRoot>,
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
                        .filter(|s| {
                            let crate_name = s.split("::").next().unwrap_or("");
                            !is_bevy_engine_crate(crate_name)
                        })
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
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}
