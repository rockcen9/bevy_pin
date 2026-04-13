use crate::prelude::*;

pub mod load_save;
pub mod pin_card;
pub mod ui;

pub fn plugin(app: &mut App) {
    load_save::plugin(app);
    pin_card::plugin(app);
    ui::plugin(app);
}
