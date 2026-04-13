use crate::manager::entity_filter::fetch::DiscoveredComponents;
use crate::manager::new_scene::{NewScenePanelRoot, SpawnedEntityId, SpawnedEntityPanel};
use crate::prelude::*;
use crate::ui_layout::theme::widgets::explorer_card::spawn_explorer_card;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, sync_spawned_entity_panel);
}

fn sync_spawned_entity_panel(
    spawned_id: Res<SpawnedEntityId>,
    components: Res<DiscoveredComponents>,
    root_query: Query<Entity, With<NewScenePanelRoot>>,
    added_roots: Query<Entity, Added<NewScenePanelRoot>>,
    panel_query: Query<Entity, With<SpawnedEntityPanel>>,
    mut commands: Commands,
) {
    let id_changed = spawned_id.is_changed();
    let root_just_added = !added_roots.is_empty();

    if !id_changed && !root_just_added {
        return;
    }

    if id_changed {
        for entity in &panel_query {
            commands.entity(entity).despawn();
        }
    }

    let Some(entity_id) = spawned_id.0 else {
        return;
    };

    let Ok(parent) = root_query.single() else {
        return;
    };

    let label = components.display_label(entity_id);
    let scene = bsn! {
        #SpawnedEntityPanel
        SpawnedEntityPanel
        DespawnOnExit::<SidebarState>(SidebarState::NewScene)
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
