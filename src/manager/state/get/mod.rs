use crate::prelude::*;

mod discovery;
mod poll;
mod variants;

pub struct StateEntry {
    pub label: String,
    pub state_type_path: String,
    pub state_resource: String,
    pub next_state_resource: Option<String>,
    pub current: Option<String>,
    pub variants: Vec<String>,
}

#[derive(Resource, Default)]
pub struct DiscoveredStates(pub Vec<StateEntry>);

pub fn plugin(app: &mut App) {
    app.init_resource::<DiscoveredStates>();
    variants::plugin(app);
    discovery::plugin(app);
    poll::plugin(app);
    app.add_systems(OnExit(SidebarState::State), clear_discovered_states);
}

fn clear_discovered_states(mut states: ResMut<DiscoveredStates>) {
    states.0.clear();
}
