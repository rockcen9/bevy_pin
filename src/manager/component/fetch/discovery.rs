use super::{ComponentEntry, DiscoveredComponents, TriggeredDiscoveries};
use crate::manager::component::query::ComponentQueries;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;

#[derive(Deserialize)]
struct WorldQueryResponse {
    result: Vec<QueryResultEntry>,
}

#[derive(Deserialize)]
struct QueryResultEntry {
    entity: u64,
    components: serde_json::Value,
}

#[derive(Component)]
struct DiscoveryContext {
    raw: String,
    with_names: Vec<String>,
    without_names: Vec<String>,
}

#[derive(Resource)]
struct DiscoveryRefreshTimer(Timer);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(BrpEndpointPlugin::<WorldQueryResponse>::default())
        .insert_resource(DiscoveryRefreshTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .add_systems(Update, (refresh_discovery, trigger_discovery).chain());
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
    mut triggered: ResMut<TriggeredDiscoveries>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !queries.is_changed() && !triggered.is_changed() {
        return;
    }

    for entry in &queries.0 {
        if triggered.0.contains(&entry.raw) {
            continue;
        }
        triggered.0.insert(entry.raw.clone());

        let raw = entry.raw.clone();
        let with_names = entry.with_names();
        let without_names = entry.without_names();

        let payload = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "world.query",
            "params": {
                "data": { "components": [], "option": "all", "has": [] },
                "filter": { "with": [], "without": [] },
                "strict": false
            }
        }))
        .unwrap();

        debug!(
            "Sending world.query for '{}' (with={:?}, without={:?})",
            raw, with_names, without_names
        );

        commands
            .spawn((
                BrpRequest::<WorldQueryResponse>::new(&server_url.0, payload),
                DiscoveryContext {
                    raw,
                    with_names,
                    without_names,
                },
            ))
            .observe(
                |trigger: On<Add, BrpResponse<WorldQueryResponse>>,
                 q: Query<(&BrpResponse<WorldQueryResponse>, &DiscoveryContext)>,
                 mut components: ResMut<DiscoveredComponents>,
                 mut commands: Commands| {
                    let ecs_entity = trigger.entity;
                    let Ok((response, ctx)) = q.get(ecs_entity) else {
                        commands.entity(ecs_entity).despawn();
                        return;
                    };

                    if let Ok(data) = &response.data {
                        let discovered: Vec<(u64, Option<String>)> = data
                            .result
                            .iter()
                            .filter_map(|entry| {
                                let keys: Vec<&str> = entry
                                    .components
                                    .as_object()?
                                    .keys()
                                    .map(String::as_str)
                                    .collect();

                                let all_with = ctx.with_names.iter().all(|name| {
                                    keys.iter()
                                        .any(|k| k.split("::").last().unwrap_or("") == name)
                                });
                                if !all_with {
                                    return None;
                                }

                                let any_without = ctx.without_names.iter().any(|name| {
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

                                Some((entry.entity, name_type_path))
                            })
                            .collect();

                        debug!(
                            "world.query '{}': {} matching entities",
                            ctx.raw,
                            discovered.len()
                        );

                        for (entity_id, name_type_path) in discovered {
                            if components
                                .0
                                .iter()
                                .any(|e| e.entity == entity_id && e.query == ctx.raw)
                            {
                                continue;
                            }
                            debug!(
                                "Discovered '{}' entity: {}, name_type_path: {:?}",
                                ctx.raw, entity_id, name_type_path
                            );
                            components.0.push(ComponentEntry {
                                entity: entity_id,
                                query: ctx.raw.clone(),
                                name_type_path,
                                value: None,
                            });
                        }
                    } else {
                        debug!("world.query '{}' request failed", ctx.raw);
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
