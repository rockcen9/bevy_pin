//! End-to-end test for the component-based BRP request/response lifecycle.
//!
//! Requires a Bevy game with `RemoteHttpPlugin` running on port 15702.
//! Start the demo game first:
//!
//!   cargo run --example demo_game
//!
//! Then in a second terminal:
//!
//!   cargo run -p brp_helper --example test_api
//!
//! Override log level with RUST_LOG, e.g.:
//!
//!   RUST_LOG=debug cargo run -p brp_helper --example test_api

use bevy::prelude::*;
use brp_helper::{BrpEndpointPlugin, BrpRequest, BrpResponse, RemoteHelperPlugin, TimeoutError};
use serde::Deserialize;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const BRP_URL: &str = "http://127.0.0.1:15702";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("brp_helper=debug,test_api=debug,info")),
        )
        .init();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(RemoteHelperPlugin)
        .add_plugins(BrpEndpointPlugin::<ListResources>::default())
        .add_systems(Startup, spawn_list_resources_request)
        .run();
}

// --- Response type ---

/// Full BRP envelope returned by `world.list_resources`.
#[derive(Deserialize, Debug)]
struct ListResources {
    result: Vec<String>,
}

// --- System ----

fn spawn_list_resources_request(mut commands: Commands) {
    let payload = serde_json::to_vec(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "world.list_resources",
        "params": null
    }))
    .unwrap();

    info!("Spawning ListResourcesRequest entity...");

    commands
        .spawn(BrpRequest::<ListResources>::new(BRP_URL, payload))
        .observe(
            |trigger: On<Add, BrpResponse<ListResources>>,
             query: Query<&BrpResponse<ListResources>>,
             mut commands: Commands,
             mut app_exit: MessageWriter<AppExit>| {
                let entity = trigger.entity;

                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(body) => {
                            info!("Received {} resources:", body.result.len());
                            for path in &body.result {
                                info!("  {}", path);
                            }
                        }
                        Err(e) => {
                            error!("Request failed: {}", e);
                        }
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
                let entity = trigger.entity;
                warn!("ListResourcesRequest timed out — is the BRP server running at {BRP_URL}?");
                commands.entity(entity).despawn();
                app_exit.write(AppExit::from_code(1));
            },
        );
}
