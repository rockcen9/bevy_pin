use crate::prelude::*;

pub mod ui;

pub fn plugin(app: &mut App) {
    ui::plugin(app);
}
