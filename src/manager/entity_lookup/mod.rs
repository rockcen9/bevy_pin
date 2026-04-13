use crate::prelude::*;

pub mod found;
pub mod history;
pub mod lookup;

pub use history::history_panel;
pub use lookup::lookup_panel;

#[derive(Component, Default, Clone, Reflect)]
pub struct EntityLookupRootPanel;

#[derive(Component, Default, Clone, Reflect)]
pub struct EntityLookupPanel;

pub fn plugin(app: &mut App) {
    app.init_resource::<FoundEntity>();
    found::plugin(app);
    history::plugin(app);
    lookup::plugin(app);
}

#[derive(Resource, Debug, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct FoundEntity(pub Option<u64>);
