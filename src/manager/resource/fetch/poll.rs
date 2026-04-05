use super::DiscoveredResources;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Deserialize)]
struct GetResourceResponse {
    result: serde_json::Value,
}

#[derive(Component)]
struct PollContext(String);

#[derive(Resource)]
struct PollTimer(Timer);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(BrpEndpointPlugin::<GetResourceResponse>::default())
        .insert_resource(PollTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(
            Update,
            poll_resources
                .run_if(in_state(SidebarState::Resource))
                .run_if(in_state(Pause(false))),
        );
}

fn poll_resources(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    resources: Res<DiscoveredResources>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for entry in &resources.0 {
        let type_path = entry.type_path.clone();
        let payload = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "world.get_resources",
            "params": { "resource": type_path }
        }))
        .unwrap();

        commands
            .spawn((
                BrpRequest::<GetResourceResponse>::new(&server_url.0, payload),
                PollContext(type_path),
            ))
            .observe(
                |trigger: On<Add, BrpResponse<GetResourceResponse>>,
                 query: Query<(&BrpResponse<GetResourceResponse>, &PollContext)>,
                 mut resources: ResMut<DiscoveredResources>,
                 mut commands: Commands| {
                    let entity = trigger.entity;
                    let Ok((response, ctx)) = query.get(entity) else {
                        commands.entity(entity).despawn();
                        return;
                    };
                    let type_path = ctx.0.clone();
                    // Always update — Some(value) if present, None to clear when resource is gone
                    let value = if let Ok(data) = &response.data {
                        data.result.get("value").cloned()
                    } else {
                        None
                    };
                    if let Some(entry) =
                        resources.0.iter_mut().find(|e| e.type_path == type_path)
                    {
                        debug!("Polled resource {}: {:?}", entry.label, value);
                        entry.value = value;
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
}
