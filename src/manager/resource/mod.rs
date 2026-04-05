use crate::prelude::*;

pub mod fetch;
pub mod ui;
pub mod update;

pub fn plugin(app: &mut App) {
    fetch::plugin(app);
    ui::plugin(app);
}
