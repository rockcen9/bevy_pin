//! Demonstrates the `brp_helper` crate against a live Bevy BRP server.
//!
//! Runs three sequential BRP requests:
//!   1. `world.list_resources`  — print all resource type paths
//!   2. `world.query`           — find every entity that has a `Name` component
//!   3. `world.list_components` — list all components on the first found entity
//!
//! # Prerequisites
//!
//! Start a Bevy app with `RemotePlugin` + `RemoteHttpPlugin` on the default port:
//!
//!   cargo run --example my_game --features dev_native
//!
//! Then in a second terminal:
//!
//!   cargo run -p brp_helper --example query_world
//!
//! Override log level with RUST_LOG, e.g.:
//!
//!   RUST_LOG=debug cargo run -p brp_helper --example query_world

use bevy::prelude::*;
use brp_helper::{
    BrpCommandsExt, BrpListComponents, BrpListResources, BrpPlugin, BrpWorldQuery, RpcResponse,
    TimeoutError,
};
use serde_json::json;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const BRP_URL: &str = "http://127.0.0.1:15702";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("brp_helper=debug,query_world=debug,info")),
        )
        .init();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(BrpPlugin)
        .add_systems(Startup, list_resources)
        .run();
}

// ── Step 1: list resources ────────────────────────────────────────────────────

fn list_resources(mut commands: Commands) {
    info!("Step 1: world.list_resources");

    let req = commands.brp_list_resources(BRP_URL);

    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpListResources>>,
             query: Query<&RpcResponse<BrpListResources>>,
             mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(body) => {
                            info!("Found {} resources:", body.result.len());
                            for path in &body.result {
                                info!("  {path}");
                            }
                            // kick off step 2
                            commands.run_system_cached(query_entities_with_name);
                        }
                        Err(e) => error!("list_resources failed: {e}"),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(
            |trigger: On<Add, TimeoutError>,
             mut commands: Commands,
             mut app_exit: MessageWriter<AppExit>| {
                warn!("list_resources timed out — is the BRP server running at {BRP_URL}?");
                commands.entity(trigger.entity).despawn();
                app_exit.write(AppExit::from_code(1));
            },
        );
}

// ── Step 2: find entities that have a Name component ─────────────────────────

fn query_entities_with_name(mut commands: Commands) {
    info!("Step 2: world.query (option=all, no filter)");

    let req = commands.brp_world_query(
        BRP_URL,
        json!({
            "data": { "components": [], "option": "all", "has": [] },
            "filter": { "with": [], "without": [] },
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
                            info!("world.query returned {} entities", body.result.len());
                            if let Some(first) = body.result.first() {
                                let id = first.entity;
                                info!("Inspecting first entity: {id}");
                                // kick off step 3
                                commands.run_system_cached_with(list_components_on_entity, id);
                            } else {
                                info!("No entities found.");
                                commands.run_system_cached(exit_success);
                            }
                        }
                        Err(e) => error!("world.query failed: {e}"),
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

// ── Step 3: list all components on one entity ─────────────────────────────────

fn list_components_on_entity(In(entity_id): In<u64>, mut commands: Commands) {
    info!("Step 3: world.list_components for entity {entity_id}");

    let req = commands.brp_list_components(BRP_URL, entity_id);

    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpListComponents>>,
             query: Query<&RpcResponse<BrpListComponents>>,
             mut commands: Commands,
             mut app_exit: MessageWriter<AppExit>| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(body) => {
                            info!("Entity has {} components:", body.result.len());
                            for path in &body.result {
                                info!("  {path}");
                            }
                        }
                        Err(e) => error!("list_components failed: {e}"),
                    }
                }
                commands.entity(entity).despawn();
                app_exit.write(AppExit::Success);
            },
        )
        .observe(
            |trigger: On<Add, TimeoutError>,
             mut commands: Commands,
             mut app_exit: MessageWriter<AppExit>| {
                warn!("list_components timed out");
                commands.entity(trigger.entity).despawn();
                app_exit.write(AppExit::from_code(1));
            },
        );
}

fn exit_success(mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
