use super::{ComponentEntry, DiscoveredComponents, TriggeredDiscoveries};
use crate::manager::component::query::ComponentQueries;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Resource)]
struct DiscoverySender(Sender<(String, Vec<(u64, Option<String>)>)>);
#[derive(Resource)]
struct DiscoveryReceiver(Receiver<(String, Vec<(u64, Option<String>)>)>);
#[derive(Resource)]
struct DiscoveryRefreshTimer(Timer);

pub(super) fn plugin(app: &mut App) {
    let (tx, rx) = unbounded();
    app.insert_resource(DiscoverySender(tx))
        .insert_resource(DiscoveryReceiver(rx))
        .insert_resource(DiscoveryRefreshTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .add_systems(
            Update,
            (refresh_discovery, trigger_discovery, receive_discovery).chain(),
        );
}

fn refresh_discovery(
    time: Res<Time>,
    mut timer: ResMut<DiscoveryRefreshTimer>,
    mut triggered: ResMut<TriggeredDiscoveries>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        triggered.0.clear();
    }
}

fn trigger_discovery(
    queries: Res<ComponentQueries>,
    sender: Res<DiscoverySender>,
    mut triggered: ResMut<TriggeredDiscoveries>,
    server_url: Res<ServerUrl>,
) {
    if !queries.is_changed() && !triggered.is_changed() {
        return;
    }

    for entry in &queries.0 {
        if triggered.0.contains(&entry.raw) {
            continue;
        }
        triggered.0.insert(entry.raw.clone());

        let tx = sender.0.clone();
        let raw = entry.raw.clone();
        let with_names = entry.with_names();
        let without_names = entry.without_names();

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "world.query",
            "params": {
                "data": {
                    "components": [],
                    "option": "all",
                    "has": []
                },
                "filter": {
                    "with": [],
                    "without": []
                },
                "strict": false
            }
        });

        debug!(
            "Sending world.query for '{}' (with={:?}, without={:?})",
            raw, with_names, without_names
        );

        let request = ehttp::Request::post(&server_url.0, serde_json::to_vec(&body).unwrap());
        ehttp::fetch(request, move |result| match result {
            Err(e) => {
                debug!("world.query '{}' request failed — {}", raw, e);
            }
            Ok(response) => {
                let Some(text) = response.text() else { return };
                let Ok(json) = serde_json::from_str::<serde_json::Value>(text) else {
                    return;
                };
                let Some(list) = json["result"].as_array() else {
                    return;
                };

                let discovered: Vec<(u64, Option<String>)> = list
                    .iter()
                    .filter_map(|entry| {
                        let entity = entry["entity"].as_u64()?;
                        let components = entry["components"].as_object()?;
                        let keys: Vec<&str> = components.keys().map(String::as_str).collect();

                        let all_with = with_names.iter().all(|name| {
                            keys.iter()
                                .any(|k| k.split("::").last().unwrap_or("") == name)
                        });
                        if !all_with {
                            return None;
                        }

                        let any_without = without_names.iter().any(|name| {
                            keys.iter()
                                .any(|k| k.split("::").last().unwrap_or("") == name)
                        });
                        if any_without {
                            return None;
                        }

                        let name_type_path = keys
                            .iter()
                            .find(|k| k.split("::").last().unwrap_or("") == "Name")
                            .map(|s| s.to_string());

                        Some((entity, name_type_path))
                    })
                    .collect();

                debug!(
                    "world.query '{}': {} matching entities",
                    raw,
                    discovered.len()
                );

                if !discovered.is_empty() {
                    let _ = tx.send((raw, discovered));
                }
            }
        });
    }
}

fn receive_discovery(
    receiver: Res<DiscoveryReceiver>,
    mut components: ResMut<DiscoveredComponents>,
) {
    while let Ok((query_str, discovered)) = receiver.0.try_recv() {
        for (entity, name_type_path) in discovered {
            if components
                .0
                .iter()
                .any(|e| e.entity == entity && e.query == query_str)
            {
                continue;
            }

            debug!(
                "Discovered '{}' entity: {}, name_type_path: {:?}",
                query_str, entity, name_type_path
            );
            components.0.push(ComponentEntry {
                entity,
                query: query_str.clone(),
                name_type_path,
                value: None,
            });
        }
    }
}
