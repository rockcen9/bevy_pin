use crate::manager::entity_filter::fetch::DiscoveredComponents;
use crate::{
    manager::pinboard::{
        load_save::{PinboardPendingData, PinboardPendingItem, PinboardSaveData},
        pincard::spawn_pincard,
        ui::PinboardContainer,
    },
    prelude::*,
    ui_layout::theme::widgets::{
        DragHandle,
        entity_card::{EntityCard, PinCardEntry, pincard_key, spawn_entity_card},
        pin_button,
    },
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, on_pin_button);
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct UnPinCard;

#[derive(Component, Clone)]
pub struct UnPinCardPinButton {
    pub entity_id: u64,
}

pub fn spawn_unpincard(
    label: String,
    entity_id: u64,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> impl Scene {
    bsn! {
        #UnPinCard
        UnPinCard
        spawn_entity_card(
            label,
            entity_id,
            left,
            top,
            width,
            height,
            DragHandle,
            bsn_list![pin_button::pin_button(UnPinCardPinButton { entity_id })]
        )
        Node {
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::Px(width),
            height: Val::Px(height),
        }
    }
}

fn on_pin_button(
    buttons: Query<(&Interaction, &UnPinCardPinButton), (Changed<Interaction>, With<Button>)>,
    cards: Query<(&EntityCard, &Node), With<UnPinCard>>,
    pinboard: Query<Entity, With<PinboardContainer>>,
    components: Res<DiscoveredComponents>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
    mut pending: ResMut<PinboardPendingItem>,
    mut next_sidebar: ResMut<NextState<SidebarState>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        debug!(
            "on_pin_button: interaction={:?} entity_id={}",
            interaction, btn.entity_id
        );
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.entity_id;
        debug!("on_pin_button: pressed for entity_id={}", entity_id);

        let Some((_ec, node)) = cards.iter().find(|(ec, _)| ec.entity_id == entity_id) else {
            debug!(
                "on_pin_button: no UnPinCard found for entity_id={}",
                entity_id
            );
            continue;
        };
        let left = match node.left {
            Val::Px(v) => v,
            _ => 10.0,
        };
        let top = match node.top {
            Val::Px(v) => v,
            _ => 10.0,
        };
        let width = match node.width {
            Val::Px(v) => v,
            _ => 400.0,
        };
        let height = 400.;
        let label = components.display_label(entity_id);
        debug!(
            "on_pin_button: card found label={} left={} top={} width={} height={}",
            label, left, top, width, height
        );

        // If already pinned, highlight the existing card and switch to pinboard
        if save_data.as_ref().map_or(false, |sd| {
            sd.cards.iter().any(|c| c.entity_id == entity_id)
        }) {
            debug!(
                "on_pin_button: entity_id={} already pinned, highlighting existing card",
                entity_id
            );
            pending.0.push(PinboardPendingData {
                entity_id,
                key: pincard_key(entity_id),
                highlight: true,
            });
            next_sidebar.set(SidebarState::Pinboard);
            continue;
        }

        let Ok(pinboard_entity) = pinboard.single() else {
            debug!("on_pin_button: PinboardContainer not found");
            continue;
        };
        debug!(
            "on_pin_button: spawning pincard for entity_id={} on pinboard {:?}",
            entity_id, pinboard_entity
        );
        let key = pincard_key(entity_id);
        let panel = commands
            .spawn_scene(spawn_pincard(
                label.clone(),
                entity_id,
                left,
                top,
                width,
                height,
            ))
            .id();
        commands.entity(pinboard_entity).add_child(panel);

        if let Some(sd) = save_data.as_mut() {
            sd.cards.push(PinCardEntry {
                entity_id,
                label: label.clone(),
                left,
                top,
                width,
                height,
            });
            sd.persist().ok();
        }

        pending.0.push(PinboardPendingData {
            entity_id,
            key,
            highlight: true,
        });
        debug!("on_pin_button: done, setting SidebarState::Pinboard");
        next_sidebar.set(SidebarState::Pinboard);
    }
}
