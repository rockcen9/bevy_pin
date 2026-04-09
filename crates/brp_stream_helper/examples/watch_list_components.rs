//! Streams component list changes on a live BRP entity.
//!
//! Uses `world.list_components+watch` — fires whenever components are added to
//! or removed from the entity.
//!
//! # Prerequisites
//!
//! Start a Bevy app with `RemotePlugin` + `RemoteHttpPlugin`:
//!
//!   cargo run --example demo_game --features dev_native
//!
//! Then in a second terminal:
//!
//!   cargo run -p brp_stream_helper --example watch_list_components

use bevy::prelude::*;
use brp_helper::{BrpCommandsExt, BrpPlugin, RpcResponse, TimeoutError};
use brp_helper::types::BrpListComponents;
use brp_stream_helper::{
    BrpListComponentsWatch, BrpStreamCommandsExt, BrpStreamPlugin, BrpWorldQuery, StreamData,
    StreamDisconnected,
};
use serde_json::json;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const BRP_URL: &str = "http://127.0.0.1:15702";
const BIRD_TYPE: &str = "demo_game::Bird";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("watch_list_components=debug,info")),
        )
        .init();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(BrpPlugin)
        .add_plugins(BrpStreamPlugin)
        .add_systems(Startup, find_bird)
        .run();
}

/// Step 1 — query for a Bird entity.
fn find_bird(mut commands: Commands) {
    info!("Querying Bird entities at {BRP_URL} …");

    let req = commands.brp_world_query(
        BRP_URL,
        json!({
            "data": { "components": [] },
            "filter": { "with": [BIRD_TYPE], "without": [] },
            "strict": false
        }),
    );

    commands
        .entity(req)
        .observe(on_bird_found)
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            warn!("BRP query timed out");
            commands.entity(trigger.entity).despawn();
        });
}

/// Step 2 — fetch initial component list and open the watch stream.
fn on_bird_found(
    trigger: On<Add, RpcResponse<BrpWorldQuery>>,
    query: Query<&RpcResponse<BrpWorldQuery>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let resp = match query.get(entity) {
        Ok(r) => r,
        Err(_) => return,
    };

    let world_query = match resp.data.as_ref() {
        Ok(q) => q,
        Err(e) => {
            warn!("BRP query error: {e}");
            commands.entity(entity).despawn();
            return;
        }
    };

    let entity_id = match world_query.result.first() {
        Some(entry) => entry.entity,
        None => {
            warn!("No Bird entities found — is the demo game running?");
            commands.entity(entity).despawn();
            return;
        }
    };

    info!("Found Bird entity {entity_id} — fetching initial component list …");
    commands.entity(entity).despawn();

    // Fetch initial snapshot.
    let list_req = commands.brp_list_components(BRP_URL, entity_id);
    commands
        .entity(list_req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpListComponents>>,
             query: Query<&RpcResponse<BrpListComponents>>,
             mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(resp) = query.get(entity) {
                    match resp.data.as_ref() {
                        Ok(lc) => {
                            info!("Initial components ({}):", lc.result.len());
                            for path in &lc.result {
                                info!("  {path}");
                            }
                        }
                        Err(e) => warn!("list_components error: {e}"),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            warn!("list_components timed out");
            commands.entity(trigger.entity).despawn();
        });

    // Open the watch stream.
    info!("Starting watch for component list changes …");

    let stream = commands.brp_watch_list_components(BRP_URL, entity_id);

    commands
        .entity(stream)
        .observe(
            |trigger: On<Insert, StreamData<BrpListComponentsWatch>>,
             query: Query<&StreamData<BrpListComponentsWatch>>| {
                let entity = trigger.entity;
                if let Ok(data) = query.get(entity) {
                    for item in &data.0 {
                        if !item.result.added.is_empty() {
                            info!("Added ({}):", item.result.added.len());
                            for path in &item.result.added {
                                info!("  + {path}");
                            }
                        }
                        if !item.result.removed.is_empty() {
                            info!("Removed ({}):", item.result.removed.len());
                            for path in &item.result.removed {
                                info!("  - {path}");
                            }
                        }
                    }
                }
            },
        )
        .observe(|trigger: On<Add, StreamDisconnected>| {
            warn!(
                "Stream {:?} disconnected — server closed or network error",
                trigger.entity
            );
        });
}
