use crate::prelude::*;

use super::ConnectionState;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, on_reconnect);
}

fn on_reconnect(
    conn: Res<ConnectionState>,
    mut prev: Local<ConnectionState>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let was_disconnected = *prev == ConnectionState::Disconnected;
    let is_connected = *conn == ConnectionState::Connected;

    if conn.is_changed() && was_disconnected && is_connected {
        next_app_state.set(app_state.get().clone());
    }

    *prev = conn.clone();
}
