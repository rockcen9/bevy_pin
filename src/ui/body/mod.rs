use crate::prelude::*;

pub mod content;
pub mod sidebar;

#[derive(Component, Default, Clone)]
pub struct BodyPanel;

pub fn plugin(app: &mut App) {
    app.add_plugins((sidebar::plugin, content::plugin));
    #[cfg(feature = "dev")]
    debug::plugin(app);
}

pub fn body_panel() -> impl Scene {
    bsn! {
        BodyPanel
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
        }
        Children [
            sidebar::ui::menu_panel(),
            content::content_panel(),
        ]
    }
}
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[source(ConnectionState = ConnectionState::Connected)]
pub enum SidebarState {
    #[default]
    State,
    Resource,
    Component,
    RemoteRPC,
}

#[cfg(feature = "dev")]
mod debug {
    use crate::ui::body::SidebarState;
    use bevy::prelude::*;
    use bevy_inspector_egui::quick::StateInspectorPlugin;
    pub fn plugin(app: &mut App) {
        app.add_plugins(StateInspectorPlugin::<SidebarState>::default().run_if(
            |mut active: Local<bool>, inputs: Res<ButtonInput<KeyCode>>| {
                if inputs.pressed(KeyCode::SuperLeft) && inputs.just_pressed(KeyCode::Digit3) {
                    *active = !*active;
                }
                *active
            },
        ));
    }
}
