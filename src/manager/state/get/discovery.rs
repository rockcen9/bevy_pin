use super::variants::{VariantsSender, fetch_variants};
use super::{DiscoveredStates, StateEntry};
use crate::manager::connection::ServerUrl;
use crate::manager::state::ui::StatePanelsRoot;
use crate::prelude::*;

#[derive(Resource)]
struct DiscoverySender(Sender<Vec<(String, String, Option<String>)>>);
#[derive(Resource)]
struct DiscoveryReceiver(Receiver<Vec<(String, String, Option<String>)>>);

pub(super) fn plugin(app: &mut App) {
    let (tx, rx) = unbounded();
    app.insert_resource(DiscoverySender(tx))
        .insert_resource(DiscoveryReceiver(rx))
        .add_observer(fetch_all_states)
        .add_systems(Update, receive_discovery);
}

fn fetch_all_states(
    _trigger: On<Add, StatePanelsRoot>,
    sender: Res<DiscoverySender>,
    server_url: Res<ServerUrl>,
) {
    let tx = sender.0.clone();

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "world.list_resources",
        "params": null
    });

    let request = ehttp::Request::post(&server_url.0, serde_json::to_vec(&body).unwrap());
    debug!("Sending world.list_resources request");
    ehttp::fetch(request, move |result| {
        let Ok(response) = result else { return };
        let Some(text) = response.text() else { return };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
            return;
        };
        let Some(list) = json["result"].as_array() else {
            return;
        };

        let all: Vec<&str> = list.iter().filter_map(|v| v.as_str()).collect();

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

        if !discovered.is_empty() {
            let _ = tx.send(discovered);
        }
    });
}

fn extract_inner_type(s: &str) -> Option<&str> {
    let start = s.find('<')? + 1;
    let end = s.rfind('>')?;
    Some(&s[start..end])
}

fn receive_discovery(
    receiver: Res<DiscoveryReceiver>,
    mut states: ResMut<DiscoveredStates>,
    variants_sender: Res<VariantsSender>,
    server_url: Res<ServerUrl>,
) {
    while let Ok(discovered) = receiver.0.try_recv() {
        debug!("Discovered state received: {}", discovered.len());
        for (type_path, state_resource, next_state_resource) in discovered {
            if states.0.iter().any(|e| e.state_type_path == type_path) {
                continue;
            }

            let label = type_path
                .split("::")
                .last()
                .unwrap_or(&type_path)
                .to_string();

            fetch_variants(type_path.clone(), variants_sender.0.clone(), &server_url.0);

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
}
