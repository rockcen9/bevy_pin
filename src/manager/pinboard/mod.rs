use crate::prelude::*;

pub mod load_save;
pub mod pincard;
pub mod ui;

pub fn plugin(app: &mut App) {
    load_save::plugin(app);
    pincard::plugin(app);
    ui::plugin(app);
}
