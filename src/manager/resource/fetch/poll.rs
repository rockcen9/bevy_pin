use super::DiscoveredResources;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Resource)]
struct PollTimer(Timer);
#[derive(Resource)]
struct PollSender(Sender<(String, Option<serde_json::Value>)>);
#[derive(Resource)]
struct PollReceiver(Receiver<(String, Option<serde_json::Value>)>);

pub(super) fn plugin(app: &mut App) {
    let (tx, rx) = unbounded();
    app.insert_resource(PollTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(PollSender(tx))
        .insert_resource(PollReceiver(rx))
        .add_systems(
            Update,
            (poll_resources, receive_poll)
                .run_if(in_state(AppState::Resource))
                .run_if(in_state(Pause(false))),
        );
}

fn poll_resources(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    resources: Res<DiscoveredResources>,
    sender: Res<PollSender>,
    server_url: Res<ServerUrl>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for entry in &resources.0 {
        let tx = sender.0.clone();
        let type_path = entry.type_path.clone();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "world.get_resources",
            "params": { "resource": type_path }
        });

        let request = ehttp::Request::post(&server_url.0, serde_json::to_vec(&body).unwrap());
        let key = type_path.clone();

        ehttp::fetch(request, move |result| {
            let Ok(response) = result else { return };
            let Some(text) = response.text() else { return };
            let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
                return;
            };
            // Always send — Some(value) if present, None to clear when resource is gone
            let value = json.get("result").and_then(|r| r.get("value")).cloned();
            let _ = tx.send((key, value));
        });
    }
}

fn receive_poll(receiver: Res<PollReceiver>, mut resources: ResMut<DiscoveredResources>) {
    while let Ok((type_path, value)) = receiver.0.try_recv() {
        if let Some(entry) = resources.0.iter_mut().find(|e| e.type_path == type_path) {
            debug!("Polled resource {}: {:?}", entry.label, value);
            entry.value = value;
        }
    }
}
