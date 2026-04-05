use super::{DiscoveredResources, ResourceEntry, ResourceScreenRoot};
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Resource)]
struct DiscoverySender(Sender<Vec<String>>);
#[derive(Resource)]
struct DiscoveryReceiver(Receiver<Vec<String>>);

pub(super) fn plugin(app: &mut App) {
    let (tx, rx) = unbounded();
    app.insert_resource(DiscoverySender(tx))
        .insert_resource(DiscoveryReceiver(rx))
        .add_observer(on_add_resource_screen_root)
        .add_systems(Update, receive_discovery);
}

fn on_add_resource_screen_root(
    _trigger: On<Add, ResourceScreenRoot>,
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
        let Ok(response) = result else {
            debug!("world.list_resources request failed");
            return;
        };
        let Some(text) = response.text() else { return };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
            return;
        };
        let Some(list) = json["result"].as_array() else {
            debug!("world.list_resources: no result array in response");
            return;
        };

        let discovered: Vec<String> = list
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| !s.starts_with("bevy_"))
            .map(|s| s.to_string())
            .collect();

        debug!(
            "world.list_resources: {} total, {} user resources found",
            list.len(),
            discovered.len()
        );

        if !discovered.is_empty() {
            let _ = tx.send(discovered);
        }
    });
}

fn receive_discovery(receiver: Res<DiscoveryReceiver>, mut resources: ResMut<DiscoveredResources>) {
    while let Ok(discovered) = receiver.0.try_recv() {
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
    }
}
