use super::DiscoveredComponents;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Resource)]
struct PollTimer(Timer);
#[derive(Resource)]
struct PollSender(Sender<(u64, String, serde_json::Value)>);
#[derive(Resource)]
struct PollReceiver(Receiver<(u64, String, serde_json::Value)>);

pub(super) fn plugin(app: &mut App) {
    let (tx, rx) = unbounded();
    app.insert_resource(PollTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(PollSender(tx))
        .insert_resource(PollReceiver(rx))
        .add_systems(
            Update,
            (poll_components, receive_poll).run_if(in_state(Pause(false))),
        );
}

fn poll_components(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    components: Res<DiscoveredComponents>,
    sender: Res<PollSender>,
    server_url: Res<ServerUrl>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for entry in &components.0 {
        let Some(name_type_path) = &entry.name_type_path else {
            continue;
        };

        let tx = sender.0.clone();
        let entity = entry.entity;
        let query = entry.query.clone();
        let name_type_path = name_type_path.clone();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "world.get_components",
            "params": {
                "entity": entity,
                "components": [name_type_path]
            }
        });

        let request = ehttp::Request::post(&server_url.0, serde_json::to_vec(&body).unwrap());
        ehttp::fetch(request, move |result| match result {
            Err(e) => {
                debug!("poll Name entity {}: request failed — {}", entity, e);
            }
            Ok(response) => {
                let Some(text) = response.text() else { return };
                let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
                    return;
                };
                if let Some(value) = json["result"]["components"]
                    .as_object()
                    .and_then(|m| m.get(&name_type_path))
                {
                    let _ = tx.send((entity, query, value.clone()));
                }
            }
        });
    }
}

fn receive_poll(receiver: Res<PollReceiver>, mut components: ResMut<DiscoveredComponents>) {
    while let Ok((entity, query, value)) = receiver.0.try_recv() {
        if let Some(entry) = components
            .0
            .iter_mut()
            .find(|e| e.entity == entity && e.query == query)
        {
            debug!("Name entity {}: value: {:#}", entity, value);
            entry.value = Some(value);
        }
    }
}
