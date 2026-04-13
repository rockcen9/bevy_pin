use bevy::{
    input_focus::InputFocus,
    text::{EditableText, TextEdit},
};

use crate::manager::pinboard::load_save::PinboardSaveData;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{COLOR_HEADER_BG, COLOR_PAUSED, COLOR_ROW_HOVER};
use crate::ui_layout::theme::widgets::ScrollableContainer;

use super::components::{
    DragHandle, EditableEntityCardField, EntityCard, EntityCardDataCache, EntityCardExpandState,
    EntityCardExpandToggle, EntityCardHighlight, EntityCardInsertField, EntityCardScrollOuter,
    entity_card_key,
};
use super::layout::render_pincard;

pub fn plugin(app: &mut App) {
    app.add_observer(on_drag_handle_added).add_systems(
        Update,
        (
            drive_pincard_highlight,
            update_header_hover,
            handle_expand_toggle,
            render_from_cache_on_expand_change.after(handle_expand_toggle),
            auto_select_on_focus,
            restore_scroll_height,
        ),
    );
}

// ── Observer: DragHandle ──────────────────────────────────────────────────────

pub(super) fn on_drag_handle_added(trigger: On<Add, DragHandle>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(on_drag)
        .observe(on_drag_end_save);
}

fn on_drag(trigger: On<Pointer<Drag>>, child_of: Query<&ChildOf>, mut nodes: Query<&mut Node>) {
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(mut node) = nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta;
    node.left = Val::Px(match node.left {
        Val::Px(x) => x + delta.x,
        _ => delta.x,
    });
    node.top = Val::Px(match node.top {
        Val::Px(y) => y + delta.y,
        _ => delta.y,
    });
}

fn on_drag_end_save(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    nodes: Query<&Node>,
    entity_cards: Query<&EntityCard>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let card_entity = parent.0;
    let Ok(node) = nodes.get(card_entity) else {
        return;
    };
    let Ok(entity_card) = entity_cards.get(card_entity) else {
        return;
    };
    let (Val::Px(left), Val::Px(top)) = (node.left, node.top) else {
        return;
    };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == entity_card.entity_id {
            entry.left = left;
            entry.top = top;
            break;
        }
    }
    save_data.persist().ok();
}

// ── Expand / collapse ─────────────────────────────────────────────────────────

pub(super) fn handle_expand_toggle(
    toggles: Query<(&Interaction, &EntityCardExpandToggle), (Changed<Interaction>, With<Button>)>,
    mut expand_state: ResMut<EntityCardExpandState>,
) {
    for (interaction, toggle) in &toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let set = expand_state.0.entry(toggle.entity_id).or_default();
        if set.contains(&toggle.type_path) {
            set.remove(&toggle.type_path);
            debug!(
                "expand_toggle: collapsed entity_id={} type_path={}",
                toggle.entity_id, toggle.type_path
            );
        } else {
            set.insert(toggle.type_path.clone());
            debug!(
                "expand_toggle: expanded entity_id={} type_path={}",
                toggle.entity_id, toggle.type_path
            );
        }
    }
}

pub(super) fn render_from_cache_on_expand_change(
    expand_state: Res<EntityCardExpandState>,
    cache: Res<EntityCardDataCache>,
    containers: Query<(Entity, &ScrollableContainer)>,
    input_focus: Res<InputFocus>,
    editable_fields: Query<&EditableEntityCardField>,
    insert_fields: Query<&EntityCardInsertField>,
    mut commands: Commands,
) {
    if !expand_state.is_changed() {
        return;
    }
    let focused_entity_id = input_focus.get().and_then(|e| {
        editable_fields
            .get(e)
            .map(|f| f.entity_id)
            .ok()
            .or_else(|| insert_fields.get(e).map(|f| f.entity_id).ok())
    });

    for (entity_id, components) in &cache.0 {
        if focused_entity_id == Some(*entity_id) {
            continue;
        }
        let key = entity_card_key(*entity_id);
        let matching: Vec<Entity> = containers
            .iter()
            .filter(|(_, c)| c.0 == key)
            .map(|(e, _)| e)
            .collect();
        if matching.is_empty() {
            debug!(
                "render_from_cache_on_expand_change: no container for entity_id={} key={}",
                entity_id, key
            );
        }
        for container_entity in matching {
            debug!(
                "render_from_cache_on_expand_change: re-rendering entity_id={} container={:?}",
                entity_id, container_entity
            );
            render_pincard(
                &mut commands,
                container_entity,
                *entity_id,
                components,
                &expand_state,
            );
        }
    }
}

// ── Header hover ──────────────────────────────────────────────────────────────

pub(super) fn update_header_hover(
    mut headers: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<EntityCardExpandToggle>),
    >,
) {
    for (interaction, mut bg) in &mut headers {
        bg.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_ROW_HOVER,
            _ => Color::NONE,
        }));
    }
}

// ── Auto-select on focus ──────────────────────────────────────────────────────

pub(super) fn auto_select_on_focus(
    input_focus: Res<InputFocus>,
    mut text_inputs: Query<&mut EditableText, With<EditableEntityCardField>>,
) {
    if !input_focus.is_changed() {
        return;
    }
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut text_input) = text_inputs.get_mut(focused) else {
        return;
    };
    text_input.queue_edit(TextEdit::SelectAll);
}

// ── Highlight animation ───────────────────────────────────────────────────────

pub(super) fn drive_pincard_highlight(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut BackgroundColor, &mut EntityCardHighlight)>,
) {
    for (entity, mut bg, mut highlight) in &mut q {
        highlight.timer.tick(time.delta());
        let t = highlight.timer.fraction();
        let start = COLOR_PAUSED.to_srgba();
        let end = COLOR_HEADER_BG.to_srgba();
        bg.0 = Color::srgba(
            start.red + (end.red - start.red) * t,
            start.green + (end.green - start.green) * t,
            start.blue + (end.blue - start.blue) * t,
            start.alpha + (end.alpha - start.alpha) * t,
        );
        if highlight.timer.just_finished() {
            commands.entity(entity).remove::<EntityCardHighlight>();
        }
    }
}

// ── Restore scroll height ─────────────────────────────────────────────────────

/// When a scroll outer node is first tagged, set `height` from save data so the
/// loaded height matches exactly what was saved (not just a `max_height` cap).
pub(super) fn restore_scroll_height(
    added: Query<(Entity, &EntityCardScrollOuter), Added<EntityCardScrollOuter>>,
    save_data: Option<Res<Persistent<PinboardSaveData>>>,
    mut nodes: Query<&mut Node>,
) {
    for (entity, outer) in &added {
        let saved_height = save_data
            .as_ref()
            .and_then(|sd| sd.cards.iter().find(|c| c.entity_id == outer.entity_id))
            .map(|c| c.height)
            .unwrap_or(300.0);
        if let Ok(mut node) = nodes.get_mut(entity) {
            node.height = Val::Px(saved_height);
        }
    }
}
