use crate::prelude::*;

pub mod state;

pub mod resource;

pub mod connection;

pub mod component;

pub fn plugin(app: &mut App) {
    app.init_state::<AppState>();
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    });

    state::plugin(app);

    resource::plugin(app);

    connection::plugin(app);

    component::plugin(app);
}
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum AppState {
    #[default]
    State,
    Resource,
    Component,
    Disconnected,
}
// #[cfg(feature = "dev")]
// mod debug {
//     use crate::manager::AppState;
//     use bevy::prelude::*;
//     use bevy_inspector_egui::quick::StateInspectorPlugin;
//     pub fn plugin(app: &mut App) {
//         app.add_plugins(
//             StateInspectorPlugin::<AppState>::default()
//                 .run_if(command_key_toggle_active(false, KeyCode::Digit3)),
//         );
//     }
//     pub fn command_key_toggle_active(
//         default: bool,
//         key: KeyCode,
//     ) -> impl FnMut(Res<ButtonInput<KeyCode>>) -> bool + Clone {
//         let mut active = default;
//         move |inputs: Res<ButtonInput<KeyCode>>| {
//             if inputs.pressed(KeyCode::SuperLeft) && inputs.just_pressed(key) {
//                 active = !active;
//             }
//             active
//         }
//     }
// }
