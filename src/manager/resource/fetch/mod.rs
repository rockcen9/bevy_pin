use crate::{manager::AppState, prelude::*};

mod discovery;
mod poll;

pub struct ResourceEntry {
    pub label: String,
    pub type_path: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Resource, Default)]
pub struct DiscoveredResources(pub Vec<ResourceEntry>);

#[derive(Component, Default, Clone)]
#[require(DespawnOnExit::<AppState>(AppState::Resource), Name::new("ResourcePanelRoot"))]
pub struct ResourceScreenRoot;

pub fn plugin(app: &mut App) {
    app.init_resource::<DiscoveredResources>()
        .add_systems(OnExit(AppState::Resource), clear_discovered_resources);
    discovery::plugin(app);
    poll::plugin(app);
}

fn clear_discovered_resources(mut resources: ResMut<DiscoveredResources>) {
    resources.0.clear();
}
