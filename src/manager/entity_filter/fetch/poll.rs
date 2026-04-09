use super::DiscoveredComponents;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Component)]
struct PollContext {
    entity: u64,
    query: String,
    name_type_path: String,
}

#[derive(Resource)]
struct PollTimer(Timer);

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(PollTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Update, poll_components.run_if(in_state(Pause(false))));
}

fn poll_components(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    components: Res<DiscoveredComponents>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for entry in &components.0 {
        let Some(name_type_path) = &entry.name_type_path else {
            continue;
        };

        let req = commands.brp_get_components(
            &server_url.0,
            entry.entity,
            &[name_type_path.clone()],
            false,
        );

        commands
            .entity(req)
            .insert(PollContext {
                entity: entry.entity,
                query: entry.query.clone(),
                name_type_path: name_type_path.clone(),
            })
            .observe(
                |trigger: On<Add, RpcResponse<BrpGetComponents>>,
                 q: Query<(&RpcResponse<BrpGetComponents>, &PollContext)>,
                 mut components: ResMut<DiscoveredComponents>,
                 mut commands: Commands| {
                    let ecs_entity = trigger.entity;
                    let Ok((response, ctx)) = q.get(ecs_entity) else {
                        commands.entity(ecs_entity).despawn();
                        return;
                    };

                    if let Ok(data) = &response.data {
                        if let Some(value) = data.result["components"]
                            .as_object()
                            .and_then(|m| m.get(&ctx.name_type_path))
                        {
                            if let Some(entry) = components
                                .bypass_change_detection()
                                .0
                                .iter_mut()
                                .find(|e| e.entity == ctx.entity && e.query == ctx.query)
                            {
                                if entry.value.as_ref() != Some(value) {
                                    debug!("Name entity {}: value: {:#}", ctx.entity, value);
                                    entry.value = Some(value.clone());
                                    components.set_changed();
                                }
                            }
                        }
                    } else {
                        debug!("poll Name entity {}: request failed", ctx.entity);
                    }

                    commands.entity(ecs_entity).despawn();
                },
            )
            .observe(
                |trigger: On<Add, TimeoutError>, mut commands: Commands| {
                    commands.entity(trigger.entity).despawn();
                },
            );
    }
}
