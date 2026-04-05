use crate::prelude::*;
use bevy::input::common_conditions::input_just_pressed;
use bevy::ui_render::GlobalUiDebugOptions;

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

fn toggle_debug_ui(mut options: ResMut<GlobalUiDebugOptions>) {
    options.toggle();
}
