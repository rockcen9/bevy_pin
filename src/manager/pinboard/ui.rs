use crate::prelude::*;
use crate::ui_layout::theme::widgets::ScrollableContainer;

use super::load_save::PinboardPendingItem;
use super::pincard::{PinCardHighlight, PinCardTitle};

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
    titles: Query<(Entity, &PinCardTitle)>,
    mut commands: Commands,
) {
    if pending.0.is_empty() {
        return;
    }

    for container in &added {
        let Some(idx) = pending.0.iter().position(|d| d.key == container.0) else {
            continue;
        };
        let data = pending.0.remove(idx);
        next_sidebar.set(SidebarState::Pinboard);

        if data.highlight {
            if let Some((title_entity, _)) = titles.iter().find(|(_, t)| t.0 == data.entity_id) {
                commands.entity(title_entity).insert(PinCardHighlight::new());
            }
        }
    }
}
