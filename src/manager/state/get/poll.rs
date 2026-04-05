use super::DiscoveredStates;
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
        let payload = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "world.get_resources",
            "params": { "resource": state_resource }
        }))
        .unwrap();

        commands
            .spawn((
                BrpRequest::<GetResourceResponse>::new(&server_url.0, payload),
                PollContext(state_resource),
            ))
            .observe(
                |trigger: On<Add, BrpResponse<GetResourceResponse>>,
                 query: Query<(&BrpResponse<GetResourceResponse>, &PollContext)>,
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

pub fn send_next_state(variant: String, next_state_resource: Option<String>, url: &str) {
    let Some(path) = next_state_resource else {
        error!("NextState resource not found — cannot switch state");
        return;
    };

    let body = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "world.insert_resources",
        "params": {
            "resource": path,
            "value": { "Pending": variant }
        }
    });

    let request = ehttp::Request::post(url, serde_json::to_vec(&body).unwrap());
    ehttp::fetch(request, move |result| match result {
        Ok(r) => {
            if let Some(body) = r.text() {
                info!("State switch response: {}", body);
            }
        }
        Err(e) => error!("State switch failed: {}", e),
    });
}
