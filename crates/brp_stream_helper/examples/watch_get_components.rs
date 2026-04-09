//! Streams component value changes on a live BRP entity.
//!
//! Uses `world.get_components+watch` — fires whenever the requested components
//! change on the entity, delivering their current reflected values as JSON.
//!
//! # Prerequisites
//!
//! Start a Bevy app with `RemotePlugin` + `RemoteHttpPlugin`:
//!
//!   cargo run --example demo_game --features dev_native
//!
//! Then in a second terminal:
//!
//!   cargo run -p brp_stream_helper --example watch_get_components

use bevy::prelude::*;
use brp_helper::{BrpCommandsExt, BrpPlugin, RpcResponse, TimeoutError};
use brp_helper::types::BrpGetComponents;
use brp_stream_helper::{
    BrpGetComponentsWatch, BrpStreamCommandsExt, BrpStreamPlugin, BrpWorldQuery, StreamData,
    StreamDisconnected,
};
use serde_json::json;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const BRP_URL: &str = "http://127.0.0.1:15702";
const BIRD_TYPE: &str = "demo_game::Bird";
const TRANSFORM_TYPE: &str = "bevy_transform::components::transform::Transform";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("watch_get_components=debug,info")),
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

/// Step 2 — fetch initial component values and open the watch stream.
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

    info!("Found Bird entity {entity_id} — fetching initial component values …");
    commands.entity(entity).despawn();

    // Fetch initial snapshot.
    let get_req = commands.brp_get_components(
        BRP_URL,
        entity_id,
        &[BIRD_TYPE.to_string(), TRANSFORM_TYPE.to_string()],
        false,
    );
    commands
        .entity(get_req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpGetComponents>>,
             query: Query<&RpcResponse<BrpGetComponents>>,
             mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(resp) = query.get(entity) {
                    match resp.data.as_ref() {
                        Ok(gc) => info!(
                            "Initial values:\n{}",
                            serde_json::to_string_pretty(&gc.result).unwrap_or_default()
                        ),
                        Err(e) => warn!("get_components error: {e}"),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            warn!("get_components timed out");
            commands.entity(trigger.entity).despawn();
        });

    // Open the watch stream.
    info!("Starting watch for: {BIRD_TYPE}, {TRANSFORM_TYPE}");

    let stream = commands.brp_watch_components(
        BRP_URL,
        entity_id,
        &[BIRD_TYPE, TRANSFORM_TYPE],
        false,
    );

    commands
        .entity(stream)
        .observe(
            |trigger: On<Insert, StreamData<BrpGetComponentsWatch>>,
             query: Query<&StreamData<BrpGetComponentsWatch>>| {
                let entity = trigger.entity;
                if let Ok(data) = query.get(entity) {
                    for item in &data.0 {
                        info!(
                            "Update:\n{}",
                            serde_json::to_string_pretty(&item.result).unwrap_or_default()
                        );
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
