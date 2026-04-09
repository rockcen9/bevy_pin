use crate::prelude::*;

pub mod component_list;
pub mod entity_list;
pub mod fetch;
pub mod inspector;
pub mod query;
pub mod ui;

pub fn plugin(app: &mut App) {
    fetch::plugin(app);
    query::plugin(app);
    entity_list::plugin(app);
    ui::plugin(app);
    component_list::plugin(app);
    inspector::plugin(app);
}
