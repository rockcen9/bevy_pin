use crate::prelude::*;

pub mod get;
pub mod ui;

pub fn plugin(app: &mut App) {
    get::plugin(app);
    ui::plugin(app);
}
