use crate::prelude::*;

pub mod component_data;
pub mod fetch;
pub mod inspector;
pub mod monitor;
pub mod query;
pub mod ui;

pub fn plugin(app: &mut App) {
    fetch::plugin(app);
    query::plugin(app);
    monitor::plugin(app);
    ui::plugin(app);
    component_data::plugin(app);
    inspector::plugin(app);
}
