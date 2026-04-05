use crate::prelude::*;

pub mod ui;

pub fn plugin(app: &mut App) {
    app.add_plugins(ui::plugin);
}
