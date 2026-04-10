use crate::prelude::*;

mod discovery;
mod poll;

pub struct ComponentEntry {
    pub entity: u64,
    pub query: String,
    pub name_type_path: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Resource, Default)]
pub struct DiscoveredComponents(pub Vec<ComponentEntry>);

#[derive(Resource, Default)]
pub struct TriggeredDiscoveries(pub HashSet<String>);

fn clear_on_exit(
    mut discovered: ResMut<DiscoveredComponents>,
    mut triggered: ResMut<TriggeredDiscoveries>,
) {
    debug!(
        "OnExit(Component): clearing {} discovered entries and {} triggered queries",
        discovered.0.len(),
        triggered.0.len()
    );
    discovered.0.clear();
    triggered.0.clear();
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DiscoveredComponents>()
        .init_resource::<TriggeredDiscoveries>()
        .add_systems(OnExit(SidebarState::EntityFilter), clear_on_exit);
    discovery::plugin(app);
    poll::plugin(app);
}
