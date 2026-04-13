use crate::manager::pinboard::load_save::PinboardSaveData;
use crate::prelude::*;
use crate::ui_layout::theme::widgets::entity_card::spawn_entity_card;
use crate::ui_layout::theme::widgets::{DragHandle, close_button};

// ── Re-exports for sibling modules that import from `super::pincard::*` ──────

pub use crate::ui_layout::theme::widgets::entity_card::{
    EntityCard, EntityCardDataCache, EntityCardEntry, EntityCardExpandState, EntityCardHighlight,
    EntityCardTitle, entity_card_key,
};

// ── Marker / close button components ─────────────────────────────────────────

/// Marks a card as belonging to the pinboard — only these cards receive a close button.
#[derive(Component, Clone, Default)]
pub struct PinCard;

/// Close button payload — identifies which pincard to remove.
#[derive(Component, Clone)]
pub struct PinCardCloseButton {
    pub entity_id: u64,
}

// ── Scene builders ────────────────────────────────────────────────────────────

/// Spawns a pincard using [`entity_card`] as the visual base. The
/// [`on_pincard_added`] observer fills in the close button once [`PinCard`]
/// is inserted on the root entity.
pub fn spawn_pin_card(
    label: String,
    entity_id: u64,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> impl Scene {
    bsn! {
        #PinCard
        PinCard
        spawn_entity_card(
            label,
            entity_id,
            left,
            top,
            width,
            height,
            DragHandle,
            bsn_list![close_button::close_button(PinCardCloseButton { entity_id })]
        )
        Node {
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::Px(width),
            height: Val::Px(height),
        }
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub fn plugin(app: &mut App) {
    app.add_systems(Update, on_pin_card_close);
}

// ── System: close pincard on button press ────────────────────────────────────

fn on_pin_card_close(
    buttons: Query<(&Interaction, &PinCardCloseButton), (Changed<Interaction>, With<Button>)>,
    pin_cards: Query<(Entity, &EntityCard), With<PinCard>>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
    mut cache: ResMut<EntityCardDataCache>,
    mut expand_state: ResMut<EntityCardExpandState>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.entity_id;

        if let Some((card_entity, _)) = pin_cards.iter().find(|(_, ec)| ec.entity_id == entity_id) {
            commands.entity(card_entity).despawn();
        }

        if let Some(ref mut sd) = save_data {
            sd.cards.retain(|c| c.entity_id != entity_id);
            sd.persist().ok();
        }

        cache.0.remove(&entity_id);
        expand_state.0.remove(&entity_id);
    }
}
