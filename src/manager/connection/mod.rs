use crate::prelude::*;

pub mod reconnect;
pub mod ui;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct ServerUrl(pub String);

impl Default for ServerUrl {
    fn default() -> Self {
        Self("http://127.0.0.1:15702".to_string())
    }
}

#[cfg(target_arch = "wasm32")]
use web_sys::{UrlSearchParams, window};

#[cfg(target_arch = "wasm32")]
pub fn get_url_param(key: &str) -> Option<String> {
    let window = window()?;

    let search = window.location().search().ok()?;

    let params = UrlSearchParams::new_with_str(&search).ok()?;

    params.get(key)
}
fn setup_connection_from_url(mut ui_state: ResMut<ServerUrl>) {
    let raw_host = {
        #[cfg(target_arch = "wasm32")]
        {
            get_url_param("host").unwrap_or_else(|| "127.0.0.1:15702".to_string())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "127.0.0.1:15702".to_string()
        }
    };

    let formatted_host = if raw_host.starts_with("http://") || raw_host.starts_with("https://") {
        raw_host
    } else {
        format!("http://{}", raw_host)
    };

    ui_state.0 = formatted_host;

    #[cfg(not(target_arch = "wasm32"))]
    println!("Native mode: Final address -> {}", ui_state.0);
}
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum ConnectionState {
    #[default]
    Unknown,
    Connected,
    Disconnected,
}

#[derive(Deserialize)]
struct HeartbeatResponse {}

#[derive(Resource)]
struct HeartbeatTimer(Timer);

pub fn plugin(app: &mut App) {
    app.init_resource::<ServerUrl>()
        .add_plugins(BrpEndpointPlugin::<HeartbeatResponse>::default())
        .insert_resource(HeartbeatTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .add_systems(Update, send_heartbeat);

    app.init_state::<ConnectionState>();

    app.add_systems(Startup, setup_connection_from_url);
    reconnect::plugin(app);
    ui::plugin(app);
}

fn send_heartbeat(
    time: Res<Time>,
    mut timer: ResMut<HeartbeatTimer>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let payload = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "world.get"
    }))
    .unwrap();

    commands
        .spawn(BrpRequest::<HeartbeatResponse>::new(&server_url.0, payload))
        .observe(
            |trigger: On<Add, BrpResponse<HeartbeatResponse>>,
             query: Query<&BrpResponse<HeartbeatResponse>>,
             current: Res<State<ConnectionState>>,
             mut next_state: ResMut<NextState<ConnectionState>>,
             mut commands: Commands| {
                let entity = trigger.entity;
                let connected = query.get(entity).map(|r| r.data.is_ok()).unwrap_or(false);
                let next = if connected {
                    ConnectionState::Connected
                } else {
                    ConnectionState::Disconnected
                };
                if *current.get() != next {
                    next_state.set(next);
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(
            |trigger: On<Add, TimeoutError>,
             current: Res<State<ConnectionState>>,
             mut next_state: ResMut<NextState<ConnectionState>>,
             mut commands: Commands| {
                if *current.get() != ConnectionState::Disconnected {
                    next_state.set(ConnectionState::Disconnected);
                }
                commands.entity(trigger.entity).despawn();
            },
        );
}
