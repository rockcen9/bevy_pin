use super::{ComponentEntry, DiscoveredComponents, TriggeredDiscoveries};
use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::query::ComponentQueries;
use crate::prelude::*;

#[derive(Component)]
struct DiscoveryContext {
    raw: String,
    with_names: Vec<String>,
    without_names: Vec<String>,
}

#[derive(Resource)]
struct DiscoveryRefreshTimer(Timer);

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(DiscoveryRefreshTimer(Timer::from_seconds(
            1.0,
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

        debug!(
            "Sending world.query for '{}' (with={:?}, without={:?})",
            raw, with_names, without_names
        );

        let req = commands.brp_world_query(
            &server_url.0,
            json!({
                "data": { "components": [], "option": "all", "has": [] },
                "filter": { "with": [], "without": [] },
                "strict": false
            }),
        );

        commands
            .entity(req)
            .insert(DiscoveryContext {
                raw,
                with_names,
                without_names,
            })
            .observe(
                |trigger: On<Add, RpcResponse<BrpWorldQuery>>,
                 q: Query<(&RpcResponse<BrpWorldQuery>, &DiscoveryContext)>,
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

                        // Remove entities no longer returned by this query (avoid
                        // touching the resource if nothing would change)
                        let discovered_ids: HashSet<u64> =
                            discovered.iter().map(|(id, _)| *id).collect();
                        let has_stale = components
                            .0
                            .iter()
                            .any(|e| e.query == ctx.raw && !discovered_ids.contains(&e.entity));
                        if has_stale {
                            components.0.retain(|e| {
                                e.query != ctx.raw || discovered_ids.contains(&e.entity)
                            });
                            debug!("world.query '{}': pruned stale entities", ctx.raw);
                        }

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
            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                commands.entity(trigger.entity).despawn();
            });
    }
}
