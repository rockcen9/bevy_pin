use super::DiscoveredStates;
use crate::prelude::*;

#[derive(Resource)]
pub(super) struct VariantsSender(pub(super) Sender<(String, Vec<String>)>);
#[derive(Resource)]
struct VariantsReceiver(Receiver<(String, Vec<String>)>);

pub(super) fn plugin(app: &mut App) {
    let (tx, rx) = unbounded();
    app.insert_resource(VariantsSender(tx))
        .insert_resource(VariantsReceiver(rx))
        .add_systems(Update, receive_variants);
}

pub(super) fn fetch_variants(type_path: String, tx: Sender<(String, Vec<String>)>, url: &str) {
    let crate_name = type_path.split("::").next().unwrap_or("").to_string();

    debug!("Sending registry.schema request for crate: {}", crate_name);
    let body = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "registry.schema",
        "params": { "with_crates": [crate_name] }
    });

    let request = ehttp::Request::post(url, serde_json::to_vec(&body).unwrap());

    ehttp::fetch(request, move |result| {
        let Ok(response) = result else { return };
        let Some(text) = response.text() else { return };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
            return;
        };

        let schema = &json["result"][&type_path];
        let Some(one_of) = schema["oneOf"].as_array() else {
            return;
        };

        let variants: Vec<String> = one_of
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        if !variants.is_empty() {
            let _ = tx.send((type_path, variants));
        }
    });
}

fn receive_variants(receiver: Res<VariantsReceiver>, mut states: ResMut<DiscoveredStates>) {
    while let Ok((type_path, variants)) = receiver.0.try_recv() {
        if let Some(entry) = states.0.iter_mut().find(|e| e.state_type_path == type_path) {
            entry.variants = variants;
        }
    }
}
