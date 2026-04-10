use crate::prelude::*;

pub mod content;
pub mod sidebar;

#[derive(Component, Default, Clone)]
pub struct BodyPanel;

pub fn plugin(app: &mut App) {
    app.add_plugins((sidebar::plugin, content::plugin));
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
    EntityFilter,
    NewScene,
    EntityLookup,
    State,
    Resource,
}
