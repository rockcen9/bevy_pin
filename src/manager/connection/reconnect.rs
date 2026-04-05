use crate::prelude::*;

use super::ConnectionState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(ConnectionState::Connected), on_reconnect);
}

fn on_reconnect(
    app_state: Res<State<SidebarState>>,
    mut next_app_state: ResMut<NextState<SidebarState>>,
) {
    next_app_state.set(app_state.get().clone());
}
