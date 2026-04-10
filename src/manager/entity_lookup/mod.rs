use crate::prelude::*;

pub mod history;
pub mod lookup;

pub use history::history_panel;
pub use lookup::lookup_panel;

#[derive(Component, Default, Clone, Reflect)]
pub struct EntityLookupRootPanel;

pub fn plugin(app: &mut App) {
    history::plugin(app);
    lookup::plugin(app);
}
