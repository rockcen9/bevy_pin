use crate::prelude::*;
use crate::ui_layout::theme::widgets::ScrollableContainer;

use super::load_save::PinboardPendingItem;
use super::pin_card::{EntityCardHighlight, EntityCardTitle};

#[derive(Component, Clone, Default)]
pub struct PinboardContainer;

pub fn pinboard_container() -> impl Scene {
    bsn! {
        PinboardContainer
        Node {
            display: Display::None,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
        }
        template(|_| Ok(Visibility::Hidden))
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, populate_pinboard);
}

fn populate_pinboard(
    mut pending: ResMut<PinboardPendingItem>,
    added: Query<&ScrollableContainer, Added<ScrollableContainer>>,
    mut next_sidebar: ResMut<NextState<SidebarState>>,
    titles: Query<(Entity, &EntityCardTitle)>,
    mut commands: Commands,
) {
    if pending.0.is_empty() {
        return;
    }

    debug!(
        "populate_pinboard: {} pending, {} new containers",
        pending.0.len(),
        added.iter().count()
    );

    for container in &added {
        debug!(
            "populate_pinboard: new ScrollableContainer key={}",
            container.0
        );
        let Some(idx) = pending.0.iter().position(|d| d.key == container.0) else {
            debug!(
                "populate_pinboard: no pending match for key={}",
                container.0
            );
            continue;
        };
        let data = pending.0.remove(idx);
        debug!(
            "populate_pinboard: matched entity_id={} highlight={}, setting Pinboard",
            data.entity_id, data.highlight
        );
        next_sidebar.set(SidebarState::Pinboard);

        if data.highlight {
            if let Some((title_entity, _)) = titles.iter().find(|(_, t)| t.0 == data.entity_id) {
                debug!(
                    "populate_pinboard: inserting PinCardHighlight on {:?}",
                    title_entity
                );
                commands
                    .entity(title_entity)
                    .insert(EntityCardHighlight::new());
            } else {
                debug!(
                    "populate_pinboard: no PinCardTitle found for entity_id={}",
                    data.entity_id
                );
            }
        }
    }

    // Handle cards that already exist in the pinboard (no new container will be added).
    pending.0.retain(|data| {
        let Some((title_entity, _)) = titles.iter().find(|(_, t)| t.0 == data.entity_id) else {
            return true; // card not yet spawned, keep pending
        };
        debug!(
            "populate_pinboard: existing card entity_id={} highlight={}",
            data.entity_id, data.highlight
        );
        next_sidebar.set(SidebarState::Pinboard);
        if data.highlight {
            debug!(
                "populate_pinboard: inserting PinCardHighlight on existing {:?}",
                title_entity
            );
            commands
                .entity(title_entity)
                .insert(EntityCardHighlight::new());
        }
        false // processed, remove from pending
    });
}
