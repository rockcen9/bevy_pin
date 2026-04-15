use crate::manager::entity_filter::fetch::DiscoveredComponents;
use crate::manager::pinboard::{
    load_save::{PinboardPendingData, PinboardPendingItem, PinboardSaveData},
    pin_card::spawn_pin_card,
    ui::PinboardContainer,
};
use crate::prelude::entity_card::*;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{COLOR_LABEL_SECONDARY, COLOR_LABEL_TERTIARY};
use crate::ui_layout::theme::widgets::pin_button;

pub(super) const CHILDOF_SENTINEL: &str = "__childof__";

const MAX_DEPTH: u32 = 8;

/// Payload on a pin button inside an ancestor entity row.
#[derive(Component, Clone)]
pub(super) struct AncestorPinButton {
    pub entity_id: u64,
}

/// Maps entity_id → parent_id, derived from `EntityCardChildrenCache`.
#[derive(Resource, Default)]
pub struct EntityCardParentCache(pub HashMap<u64, u64>);

pub fn plugin(app: &mut App) {
    app.init_resource::<EntityCardParentCache>()
        .add_systems(Update, (update_parent_cache, on_ancestor_pin_button));
}

fn update_parent_cache(
    children_cache: Res<super::children::EntityCardChildrenCache>,
    mut parent_cache: ResMut<EntityCardParentCache>,
) {
    if !children_cache.is_changed() {
        return;
    }
    parent_cache.0.clear();
    for (parent_id, children) in &children_cache.0 {
        for child_id in children {
            parent_cache.0.insert(*child_id, *parent_id);
        }
    }
}

/// Returns the ancestor chain for `entity_id`, oldest ancestor first, direct parent last.
pub(super) fn collect_ancestors(entity_id: u64, parent_cache: &EntityCardParentCache) -> Vec<u64> {
    let mut chain = Vec::new();
    let mut current = entity_id;
    for _ in 0..MAX_DEPTH {
        let Some(&parent_id) = parent_cache.0.get(&current) else {
            break;
        };
        chain.push(parent_id);
        current = parent_id;
    }
    chain
}

/// Spawns one ancestor row per entry in the ancestor chain into `container_entity`.
pub(super) fn render_ancestor_tree(
    commands: &mut Commands,
    container_entity: Entity,
    entity_id: u64,
    parent_cache: &EntityCardParentCache,
    components: &Res<DiscoveredComponents>,
) {
    let ancestors = collect_ancestors(entity_id, parent_cache);
    for (depth, ancestor_id) in ancestors.iter().enumerate() {
        let row = spawn_ancestor_row(commands, *ancestor_id, depth as u32, components);
        commands.entity(container_entity).add_child(row);
    }
}

fn spawn_ancestor_row(
    commands: &mut Commands,
    ancestor_id: u64,
    depth: u32,
    components: &Res<DiscoveredComponents>,
) -> Entity {
    let indent = 42.0 + (depth as f32) * 10.0;
    let label = components.display_label(ancestor_id);
    commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                padding: UiRect {
                    left: Val::Px({ indent }),
                    right: Val::Px(6.0),
                    top: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                },
                column_gap: Val::Px(4.0),
            }
            Children [
                (
                    Text::new( label.clone() )
                    template(|_| Ok(TextFont::from_font_size(11.0)))
                    TextColor(COLOR_LABEL_SECONDARY)
                    TextLayout { linebreak: LineBreak::NoWrap }
                ),
                (
                    pin_button::pin_button(AncestorPinButton { entity_id: ancestor_id })
                ),
            ]
        })
        .id()
}

fn on_ancestor_pin_button(
    buttons: Query<(&Interaction, &AncestorPinButton), (Changed<Interaction>, With<Button>)>,
    pinboard: Query<Entity, With<PinboardContainer>>,
    components: Res<DiscoveredComponents>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
    mut pending: ResMut<PinboardPendingItem>,
    mut next_sidebar: ResMut<NextState<SidebarState>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.entity_id;

        if save_data.as_ref().map_or(false, |sd| {
            sd.cards.iter().any(|c| c.entity_id == entity_id)
        }) {
            pending.0.push(PinboardPendingData {
                entity_id,
                key: entity_card_key(entity_id),
                highlight: true,
            });
            next_sidebar.set(SidebarState::Pinboard);
            continue;
        }

        let Ok(pinboard_entity) = pinboard.single() else {
            continue;
        };

        let label = components.display_label(entity_id);
        let left = 20.0;
        let top = 20.0;
        let width = 280.0;
        let height = 400.0;
        let key = entity_card_key(entity_id);

        let panel = commands
            .spawn_scene(spawn_pin_card(
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
            sd.cards.push(EntityCardEntry {
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
        next_sidebar.set(SidebarState::Pinboard);
    }
}

// ── Top-level header scene ────────────────────────────────────────────────────

pub(super) fn childof_header(
    entity_id: u64,
    is_expanded: bool,
    ancestor_count: usize,
) -> impl Scene {
    let icon = if is_expanded { "V" } else { ">" };
    let label = format!("ChildOf ({})", ancestor_count);
    bsn! {
        Button
        EntityCardExpandToggle {
            entity_id: { entity_id },
            type_path: { CHILDOF_SENTINEL.to_string() },
        }
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
            column_gap: Val::Px(4.0),
        }
        BackgroundColor(Color::NONE)
        Children [
            (Node { width: Val::Px(20.0), height: Val::Px(20.0), flex_shrink: 0.0 }),
            (
                Text::new( icon.to_string() )
                template(|_| Ok(TextFont::from_font_size(9.0)))
                TextColor(COLOR_LABEL_TERTIARY)
            ),
            (
                Text::new( label.clone() )
                template(|_| Ok(TextFont::from_font_size(12.0)))
                TextColor(COLOR_LABEL_SECONDARY)
                TextLayout { linebreak: LineBreak::NoWrap }
            ),
        ]
    }
}
