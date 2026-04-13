use crate::manager::entity_filter::entity_list::ui::{ComponentEntityRow, SelectedRow};
use crate::manager::entity_filter::fetch::DiscoveredComponents;
use crate::prelude::*;
use crate::ui_layout::theme::widgets::explorer_card::spawn_explorer_card;

#[derive(Component, FromTemplate)]
pub struct ComponentListRoot;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, show_selected_entity_card);
}

pub fn component_list_root() -> impl Scene {
    bsn! {
        #ComponentListRoot
        ComponentListRoot
        DespawnOnExit::<SidebarState>(SidebarState::EntityFilter)
        DespawnOnExit::<SidebarState>(SidebarState::NewScene)
        DespawnOnExit::<SidebarState>(SidebarState::EntityLookup)
        Node {
      width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    // position_type: PositionType::Relative,
        }
        Children [
            // spawn entity card here
        ]
    }
}

fn show_selected_entity_card(
    selected: Query<&ComponentEntityRow, Added<SelectedRow>>,
    list_root: Query<Entity, With<ComponentListRoot>>,
    components: Res<DiscoveredComponents>,
    mut commands: Commands,
) {
    let Some(row) = selected.iter().next() else {
        return;
    };
    let Ok(root_entity) = list_root.single() else {
        return;
    };
    commands.entity(root_entity).despawn_children();
    let entity_id = row.entity;
    let label = components.display_label(entity_id);

    let left = 10.0;
    let top = 10.0;
    let width = 400.0;
    let height = 800.0;

    let card = commands
        .spawn_scene(spawn_explorer_card(
            label, entity_id, left, top, width, height,
        ))
        .id();
    commands.entity(root_entity).add_child(card);
}
