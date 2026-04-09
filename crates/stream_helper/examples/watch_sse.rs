/// Example: watch component list changes on a BRP entity using the generic SSE ECS pattern.
///
/// Demonstrates `SseStream<T>` + `StreamPlugin<T>` + `StreamData<T>` + `AbortStream`.
/// The initial entity lookup uses `brp_helper` (async, WASM-compatible).
/// The BRP payload for the watch stream is built manually — `stream_helper` knows nothing
/// about BRP.
///
/// Start the demo game first:
///   cargo run --example demo_game --features dev_native
///
/// Then in a second terminal:
///   cargo run --example watch_sse -p stream_helper
use bevy::prelude::*;
use brp_helper::{
    BrpCommandsExt, BrpPlugin, RpcResponse, TimeoutError,
    methods,
    types::{BrpListComponentsWatch, BrpWorldQuery},
};
use serde_json::json;
use stream_helper::{AbortStream, SseStream, StreamData, StreamDisconnected, StreamPlugin};
use tracing::{info, warn};
use tracing_subscriber;

const BRP_URL: &str = "http://127.0.0.1:15702";
const BIRD_TYPE: &str = "demo_game::Bird";

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(BrpPlugin)
        .add_plugins(StreamPlugin::<BrpListComponentsWatch>::default())
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

/// Step 2 — spawn the SSE watch stream for the found entity.
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
            warn!("No Bird entities found.");
            commands.entity(entity).despawn();
            return;
        }
    };

    info!("Found Bird entity {entity_id} — starting watch …");
    commands.entity(entity).despawn();

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": methods::WORLD_LIST_COMPONENTS_WATCH,
        "params": { "entity": entity_id }
    });

    commands
        .spawn((
            Name::new(format!("SseStream(bird={entity_id})")),
            SseStream::<BrpListComponentsWatch>::new(
                BRP_URL,
                body,
                format!("list-watch entity={entity_id}"),
            ),
        ))
        .observe(
            |trigger: On<Insert, StreamData<BrpListComponentsWatch>>,
             query: Query<&StreamData<BrpListComponentsWatch>>| {
                let entity = trigger.entity;
                if let Ok(data) = query.get(entity) {
                    for item in &data.0 {
                        let r = &item.result;
                        if !r.added.is_empty() {
                            info!("Added: {:?}", r.added);
                        }
                        if !r.removed.is_empty() {
                            info!("Removed: {:?}", r.removed);
                        }
                    }
                }
            },
        )
        .observe(|trigger: On<Add, StreamDisconnected>| {
            warn!("Stream {:?} disconnected", trigger.entity);
            // reconnect logic or despawn goes here
        });

    info!("Spawned SseStream — watching Bird {entity_id}");

    // To abort from another system, insert AbortStream on the entity:
    //   commands.entity(stream_entity).insert(AbortStream);
    let _ = AbortStream; // suppress unused import warning
}
