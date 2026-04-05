use crate::prelude::*;

pub mod fetch;
pub mod query;
pub mod ui;

pub fn plugin(app: &mut App) {
    fetch::plugin(app);
    query::plugin(app);
    ui::plugin(app);
}
