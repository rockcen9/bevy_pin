use crate::prelude::*;

mod list;
pub use list::*;

mod insert_component;
pub use insert_component::insert_component_panel;

mod unknown_issue;

#[derive(Component, FromTemplate)]
pub struct ComponentListRoot;

pub fn plugin(app: &mut App) {
    app.add_plugins((list::plugin, unknown_issue::plugin, insert_component::plugin));
}
pub fn component_list_root() -> impl Scene {
    bsn! {
        #ComponentListRoot
        ComponentListRoot
        DespawnOnExit::<SidebarState>(SidebarState::EntityFilter)
        DespawnOnExit::<SidebarState>(SidebarState::NewScene)
        DespawnOnExit::<SidebarState>(SidebarState::EntityLookup)
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            min_width: Val::Px(280.0),
            max_width: Val::Px(280.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        Children [
            component_list_panel(),
            insert_component_panel(),
        ]
    }
}
