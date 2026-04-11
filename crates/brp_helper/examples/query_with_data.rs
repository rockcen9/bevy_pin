//! Demonstrates fetching component *data* via `world.query`.
//!
//! Runs two sequential BRP requests:
//!   1. `world.query` with `filter.with = ["bevy_core::name::Name"]`
//!      and `data.components = ["bevy_core::name::Name",
//!                              "bevy_transform::components::transform::Transform"]`
//!      — returns each matching entity together with the serialized values of
//!      those two components.
//!   2. Same query but using `data.option` instead of `data.components` so that
//!      entities without `Transform` are still included (component absent = key
//!      omitted from the response rather than a hard failure).
//!
//! # Prerequisites
//!
//! Start a Bevy app with `RemotePlugin` + `RemoteHttpPlugin` on the default port:
//!
//!   cargo run --example my_game --features dev_native
//!
//! Then in a second terminal:
//!
//!   cargo run -p brp_helper --example query_with_data
//!
//! Override log level with RUST_LOG, e.g.:
//!
//!   RUST_LOG=debug cargo run -p brp_helper --example query_with_data

use bevy::prelude::*;
use brp_helper::{BrpCommandsExt, BrpPlugin, BrpWorldQuery, RpcResponse, TimeoutError};
use serde_json::json;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const BRP_URL: &str = "http://127.0.0.1:15702";

/// Components we want data for.
///
/// Tip: run the `query_world` example first — Step 3 prints the real type paths
/// for entities in your app so you can update these constants to match.
const NAME_PATH: &str = "demo_game::Bird";
const TRANSFORM_PATH: &str = "bevy_transform::components::transform::Transform";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("brp_helper=debug,query_with_data=debug,info")),
        )
        .init();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(BrpPlugin)
        .add_systems(Startup, query_required_components)
        .run();
}

// ── Step 1: query with required components (data.components) ──────────────────
//
// Only entities that have *both* Name and Transform are returned.
// The response `components` map contains the serialized values for both.

fn query_required_components(mut commands: Commands) {
    info!("Step 1: world.query — required components (Name + Transform)");

    let req = commands.brp_world_query(
        BRP_URL,
        json!({
            "data": {
                "components": [NAME_PATH, TRANSFORM_PATH],
                "option": [],
                "has": []
            },
            "filter": {
                "with":    [],
                "without": []
            },
            "strict": false
        }),
    );

    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpWorldQuery>>,
             query: Query<&RpcResponse<BrpWorldQuery>>,
             mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(body) => {
                            info!(
                                "Step 1 returned {} entities with Name + Transform:",
                                body.result.len()
                            );
                            for entry in &body.result {
                                info!("  entity {}:", entry.entity);
                                info!("    components = {}", entry.components);
                            }
                            commands.run_system_cached(query_optional_components);
                        }
                        Err(e) => error!("world.query (required) failed: {e}"),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(
            |trigger: On<Add, TimeoutError>,
             mut commands: Commands,
             mut app_exit: MessageWriter<AppExit>| {
                warn!("world.query timed out — is the BRP server running at {BRP_URL}?");
                commands.entity(trigger.entity).despawn();
                app_exit.write(AppExit::from_code(1));
            },
        );
}

// ── Step 2: query with optional components (data.option) ─────────────────────
//
// All entities that have Name are returned regardless of whether they also have
// Transform.  Entities without Transform simply omit that key from `components`.

fn query_optional_components(mut commands: Commands) {
    info!("Step 2: world.query — optional Transform (data.option)");

    let req = commands.brp_world_query(
        BRP_URL,
        json!({
            "data": {
                "components": [NAME_PATH],
                "option":     [TRANSFORM_PATH],
                "has":        []
            },
            "filter": {
                "with":    [NAME_PATH],
                "without": []
            },
            "strict": false
        }),
    );

    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpWorldQuery>>,
             query: Query<&RpcResponse<BrpWorldQuery>>,
             mut commands: Commands,
             mut app_exit: MessageWriter<AppExit>| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(body) => {
                            info!(
                                "Step 2 returned {} entities with Name (Transform optional):",
                                body.result.len()
                            );
                            for entry in &body.result {
                                let has_transform = entry.components.get(TRANSFORM_PATH).is_some();
                                info!(
                                    "  entity {} — transform present: {}",
                                    entry.entity, has_transform
                                );
                                info!("    components = {}", entry.components);
                            }
                            app_exit.write(AppExit::Success);
                        }
                        Err(e) => error!("world.query (optional) failed: {e}"),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(
            |trigger: On<Add, TimeoutError>,
             mut commands: Commands,
             mut app_exit: MessageWriter<AppExit>| {
                warn!("world.query timed out");
                commands.entity(trigger.entity).despawn();
                app_exit.write(AppExit::from_code(1));
            },
        );
}
