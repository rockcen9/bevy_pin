use crate::prelude::*;
use crate::ui::theme::palette::COLOR_BG_BASE;

use crate::{ui::body::body_panel, ui::header::head_panel};
pub mod body;
pub mod debug;
pub mod header;
pub mod root;
pub mod theme;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_root_panel);
    app.add_plugins((header::plugin, body::plugin, debug::plugin));
}

#[derive(Component, Default, Clone)]
pub struct RootPanel;

fn spawn_root_panel(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        RootPanel
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(COLOR_BG_BASE)
        Children [
            head_panel(),
            body_panel()

        ]
    });
}
