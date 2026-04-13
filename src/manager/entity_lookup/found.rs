use crate::manager::entity_filter::fetch::DiscoveredComponents;
use crate::ui_layout::theme::widgets::explorer_card::spawn_explorer_card;
use crate::{manager::entity_lookup::EntityLookupRootPanel, prelude::*};

use super::FoundEntity;

#[derive(Component, Default, Clone, Reflect)]
pub struct FoundEntityPanel;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, sync_found_entity_panel);
}

fn sync_found_entity_panel(
    found: Res<FoundEntity>,
    components: Res<DiscoveredComponents>,
    panel_query: Query<Entity, With<FoundEntityPanel>>,
    lookup_panel: Query<Entity, With<EntityLookupRootPanel>>,
    mut commands: Commands,
) {
    if !found.is_changed() {
        return;
    }

    for entity in &panel_query {
        commands.entity(entity).despawn();
    }

    let Some(entity_id) = found.0 else {
        return;
    };

    let Ok(parent) = lookup_panel.single() else {
        return;
    };

    let label = components.display_label(entity_id);
    let scene = bsn! {
        #FoundEntityPanel
        FoundEntityPanel
        DespawnOnExit::<SidebarState>(SidebarState::EntityLookup)
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
        }
        Children [
            spawn_explorer_card(label, entity_id, 0.0, 0.0, 400.0, 800.0)
        ]
    };
    let child = commands.spawn_scene(scene).id();
    commands.entity(parent).add_child(child);
}
