use super::DiscoveredStates;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Resource)]
struct PollTimer(Timer);
#[derive(Resource)]
struct PollSender(Sender<(String, Option<String>)>);
#[derive(Resource)]
struct PollReceiver(Receiver<(String, Option<String>)>);

pub(super) fn plugin(app: &mut App) {
    let (tx, rx) = unbounded();
    app.insert_resource(PollTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(PollSender(tx))
        .insert_resource(PollReceiver(rx))
        .add_systems(Update, (poll_states, receive_poll).run_if(in_state(Pause(false))));
}

fn poll_states(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    states: Res<DiscoveredStates>,
    sender: Res<PollSender>,
    server_url: Res<ServerUrl>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for entry in &states.0 {
        let tx = sender.0.clone();
        let state_resource = entry.state_resource.clone();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "world.get_resources",
            "params": { "resource": state_resource }
        });

        let request = ehttp::Request::post(&server_url.0, serde_json::to_vec(&body).unwrap());
        let key = state_resource.clone();

        ehttp::fetch(request, move |result| {
            let Ok(response) = result else { return };
            let Some(text) = response.text() else { return };
            let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
                return;
            };
            // Send Some(variant) if present, or None to clear when substate doesn't exist
            let variant = json["result"]["value"].as_str().map(|s| s.to_string());
            let _ = tx.send((key, variant));
        });
    }
}

fn receive_poll(receiver: Res<PollReceiver>, mut states: ResMut<DiscoveredStates>) {
    while let Ok((state_resource, variant)) = receiver.0.try_recv() {
        if let Some(entry) = states
            .0
            .iter_mut()
            .find(|e| e.state_resource == state_resource)
        {
            entry.current = variant;
        }
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
