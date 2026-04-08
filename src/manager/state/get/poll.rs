use super::DiscoveredStates;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Component)]
struct PollContext(String);

#[derive(Resource)]
struct PollTimer(Timer);

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(PollTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Update, poll_states.run_if(in_state(Pause(false))));
}

fn poll_states(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    states: Res<DiscoveredStates>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for entry in &states.0 {
        let state_resource = entry.state_resource.clone();
        let req = commands.brp_get_resources(&server_url.0, &state_resource);
        commands
            .entity(req)
            .insert(PollContext(state_resource))
            .observe(
                |trigger: On<Add, RpcResponse<BrpGetResources>>,
                 query: Query<(&RpcResponse<BrpGetResources>, &PollContext)>,
                 mut states: ResMut<DiscoveredStates>,
                 mut commands: Commands| {
                    let entity = trigger.entity;
                    let Ok((response, ctx)) = query.get(entity) else {
                        commands.entity(entity).despawn();
                        return;
                    };
                    let state_resource = ctx.0.clone();
                    let variant = if let Ok(data) = &response.data {
                        data.result["value"].as_str().map(|s| s.to_string())
                    } else {
                        None
                    };
                    if let Some(entry) =
                        states.0.iter_mut().find(|e| e.state_resource == state_resource)
                    {
                        entry.current = variant;
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
