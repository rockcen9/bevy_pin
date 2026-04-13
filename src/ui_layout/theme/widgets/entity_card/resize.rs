use bevy::window::{CursorIcon, SystemCursorIcon};

use crate::manager::pinboard::load_save::PinboardSaveData;
use crate::prelude::*;

use super::components::{
    EntityCard, EntityCardResizeCornerBL, EntityCardResizeCornerBR, EntityCardResizeCornerTR,
    EntityCardResizeHandle, EntityCardResizeHandleBottom, EntityCardResizeHandleLeft,
    EntityCardResizeHandleTop, EntityCardResizeCornerTL,
};

pub fn plugin(app: &mut App) {
    app.add_observer(on_resize_handle_added)
        .add_observer(on_resize_handle_bottom_added)
        .add_observer(on_resize_handle_left_added)
        .add_observer(on_resize_handle_top_added)
        .add_observer(on_resize_corner_br_added)
        .add_observer(on_resize_corner_bl_added)
        .add_observer(on_resize_corner_tr_added)
        .add_observer(on_resize_corner_tl_added);
}

// Minimum card height: header (~40 px) + minimal scroll area (60 px).
const MIN_CARD_HEIGHT: f32 = 100.0;

// ── Right edge ────────────────────────────────────────────────────────────────

pub(super) fn on_resize_handle_added(
    trigger: On<Add, EntityCardResizeHandle>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_drag)
        .observe(on_resize_drag_end)
        .observe(on_resize_over)
        .observe(on_resize_out);
}

fn on_resize_drag(
    mut trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    mut nodes: Query<&mut Node>,
) {
    trigger.propagate(false);
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(mut node) = nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.x;
    node.width = Val::Px(match node.width {
        Val::Px(w) => (w + delta).max(180.0),
        _ => 280.0 + delta,
    });
}

fn on_resize_drag_end(
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
    let width = match node.width {
        Val::Px(w) => w,
        _ => return,
    };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == entity_card.entity_id {
            entry.width = width;
            break;
        }
    }
    save_data.persist().ok();
}

fn on_resize_over(
    _trigger: On<Pointer<Over>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::EwResize));
    }
}

fn on_resize_out(
    _trigger: On<Pointer<Out>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::Default));
    }
}

// ── Bottom edge ───────────────────────────────────────────────────────────────

pub(super) fn on_resize_handle_bottom_added(
    trigger: On<Add, EntityCardResizeHandleBottom>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_bottom_drag)
        .observe(on_resize_bottom_drag_end)
        .observe(on_resize_bottom_over)
        .observe(on_resize_bottom_out);
}

fn on_resize_bottom_drag(
    mut trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    mut card_nodes: Query<(&EntityCard, &mut Node, &ComputedNode)>,
) {
    trigger.propagate(false);
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok((_, mut node, computed)) = card_nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.y;
    let base = match node.height {
        Val::Px(h) => h,
        _ => computed.size().y,
    };
    node.height = Val::Px((base + delta).max(MIN_CARD_HEIGHT));
}

fn on_resize_bottom_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    card_nodes: Query<(&EntityCard, &Node)>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok((entity_card, node)) = card_nodes.get(parent.0) else {
        return;
    };
    if let Val::Px(h) = node.height {
        for entry in save_data.cards.iter_mut() {
            if entry.entity_id == entity_card.entity_id {
                entry.height = h;
                break;
            }
        }
        save_data.persist().ok();
    }
}

fn on_resize_bottom_over(
    _trigger: On<Pointer<Over>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::NsResize));
    }
}

fn on_resize_bottom_out(
    _trigger: On<Pointer<Out>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::Default));
    }
}

// ── Left edge ─────────────────────────────────────────────────────────────────

pub(super) fn on_resize_handle_left_added(
    trigger: On<Add, EntityCardResizeHandleLeft>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_left_drag)
        .observe(on_resize_left_drag_end)
        .observe(on_resize_over)
        .observe(on_resize_out);
}

fn on_resize_left_drag(
    mut trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    mut nodes: Query<&mut Node, With<EntityCard>>,
) {
    trigger.propagate(false);
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(mut node) = nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.x;
    let current_width = match node.width {
        Val::Px(w) => w,
        _ => 280.0,
    };
    let new_width = (current_width - delta).max(180.0);
    let actual_delta = current_width - new_width;
    node.width = Val::Px(new_width);
    node.left = Val::Px(match node.left {
        Val::Px(x) => x + actual_delta,
        _ => actual_delta,
    });
}

fn on_resize_left_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    nodes: Query<&Node, With<EntityCard>>,
    entity_cards: Query<&EntityCard>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(node) = nodes.get(parent.0) else {
        return;
    };
    let Ok(entity_card) = entity_cards.get(parent.0) else {
        return;
    };
    let (Val::Px(left), Val::Px(width)) = (node.left, node.width) else {
        return;
    };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == entity_card.entity_id {
            entry.left = left;
            entry.width = width;
            break;
        }
    }
    save_data.persist().ok();
}

// ── Top edge ──────────────────────────────────────────────────────────────────

pub(super) fn on_resize_handle_top_added(
    trigger: On<Add, EntityCardResizeHandleTop>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_top_drag)
        .observe(on_resize_top_drag_end)
        .observe(on_resize_bottom_over)
        .observe(on_resize_bottom_out);
}

fn on_resize_top_drag(
    mut trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    mut card_nodes: Query<(&EntityCard, &mut Node, &ComputedNode)>,
) {
    trigger.propagate(false);
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok((_, mut node, computed)) = card_nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.y;
    let current_h = match node.height {
        Val::Px(h) => h,
        _ => computed.size().y,
    };
    let new_h = (current_h - delta).max(MIN_CARD_HEIGHT);
    let actual_delta = current_h - new_h;
    node.height = Val::Px(new_h);
    node.top = Val::Px(match node.top {
        Val::Px(y) => y + actual_delta,
        _ => actual_delta,
    });
}

fn on_resize_top_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    card_nodes: Query<(&EntityCard, &Node)>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok((entity_card, node)) = card_nodes.get(parent.0) else {
        return;
    };
    let (Val::Px(top), Val::Px(height)) = (node.top, node.height) else {
        return;
    };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == entity_card.entity_id {
            entry.top = top;
            entry.height = height;
            break;
        }
    }
    save_data.persist().ok();
}

// ── Corner resize helpers ─────────────────────────────────────────────────────

/// Resize width from right + height from bottom (no position change).
fn corner_drag_br(delta_x: f32, delta_y: f32, card_node: &mut Node, card_computed: &ComputedNode) {
    card_node.width = Val::Px(match card_node.width {
        Val::Px(w) => (w + delta_x).max(180.0),
        _ => 280.0 + delta_x,
    });
    let base = match card_node.height {
        Val::Px(h) => h,
        _ => card_computed.size().y,
    };
    card_node.height = Val::Px((base + delta_y).max(MIN_CARD_HEIGHT));
}

/// Resize width from left (moves card) + height from bottom.
fn corner_drag_bl(delta_x: f32, delta_y: f32, card_node: &mut Node, card_computed: &ComputedNode) {
    let current_w = match card_node.width {
        Val::Px(w) => w,
        _ => 280.0,
    };
    let new_w = (current_w - delta_x).max(180.0);
    let actual_delta = current_w - new_w;
    card_node.width = Val::Px(new_w);
    card_node.left = Val::Px(match card_node.left {
        Val::Px(x) => x + actual_delta,
        _ => actual_delta,
    });
    let base = match card_node.height {
        Val::Px(h) => h,
        _ => card_computed.size().y,
    };
    card_node.height = Val::Px((base + delta_y).max(MIN_CARD_HEIGHT));
}

/// Resize width from right + height from top (moves card).
fn corner_drag_tr(delta_x: f32, delta_y: f32, card_node: &mut Node, card_computed: &ComputedNode) {
    card_node.width = Val::Px(match card_node.width {
        Val::Px(w) => (w + delta_x).max(180.0),
        _ => 280.0 + delta_x,
    });
    let current_h = match card_node.height {
        Val::Px(h) => h,
        _ => card_computed.size().y,
    };
    let new_h = (current_h - delta_y).max(MIN_CARD_HEIGHT);
    let actual_delta = current_h - new_h;
    card_node.height = Val::Px(new_h);
    card_node.top = Val::Px(match card_node.top {
        Val::Px(y) => y + actual_delta,
        _ => actual_delta,
    });
}

/// Resize width from left (moves card) + height from top (moves card).
fn corner_drag_tl(delta_x: f32, delta_y: f32, card_node: &mut Node, card_computed: &ComputedNode) {
    let current_w = match card_node.width {
        Val::Px(w) => w,
        _ => 280.0,
    };
    let new_w = (current_w - delta_x).max(180.0);
    let actual_dx = current_w - new_w;
    card_node.width = Val::Px(new_w);
    card_node.left = Val::Px(match card_node.left {
        Val::Px(x) => x + actual_dx,
        _ => actual_dx,
    });
    let current_h = match card_node.height {
        Val::Px(h) => h,
        _ => card_computed.size().y,
    };
    let new_h = (current_h - delta_y).max(MIN_CARD_HEIGHT);
    let actual_dy = current_h - new_h;
    card_node.height = Val::Px(new_h);
    card_node.top = Val::Px(match card_node.top {
        Val::Px(y) => y + actual_dy,
        _ => actual_dy,
    });
}

fn corner_save(entity_id: u64, card_node: &Node, save_data: &mut Persistent<PinboardSaveData>) {
    let Val::Px(left) = card_node.left else {
        return;
    };
    let Val::Px(top) = card_node.top else { return };
    let Val::Px(width) = card_node.width else {
        return;
    };
    let Val::Px(height) = card_node.height else {
        return;
    };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == entity_id {
            entry.left = left;
            entry.top = top;
            entry.width = width;
            entry.height = height;
            break;
        }
    }
    save_data.persist().ok();
}

// ── BR corner ─────────────────────────────────────────────────────────────────

pub(super) fn on_resize_corner_br_added(
    trigger: On<Add, EntityCardResizeCornerBR>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(
            |mut trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             entity_cards: Query<&EntityCard>,
             mut card_nodes: Query<(&mut Node, &ComputedNode), With<EntityCard>>| {
                trigger.propagate(false);
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(_) = entity_cards.get(p.0) else { return };
                let Ok((mut cn, computed)) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_br(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    &mut cn,
                    computed,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             card_nodes: Query<(&EntityCard, &Node)>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok((ec, cn)) = card_nodes.get(p.0) else {
                    return;
                };
                corner_save(ec.entity_id, cn, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::SeResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}

// ── BL corner ─────────────────────────────────────────────────────────────────

pub(super) fn on_resize_corner_bl_added(
    trigger: On<Add, EntityCardResizeCornerBL>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(
            |mut trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             entity_cards: Query<&EntityCard>,
             mut card_nodes: Query<(&mut Node, &ComputedNode), With<EntityCard>>| {
                trigger.propagate(false);
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(_) = entity_cards.get(p.0) else { return };
                let Ok((mut cn, computed)) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_bl(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    &mut cn,
                    computed,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             card_nodes: Query<(&EntityCard, &Node)>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok((ec, cn)) = card_nodes.get(p.0) else {
                    return;
                };
                corner_save(ec.entity_id, cn, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::SwResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}

// ── TR corner ─────────────────────────────────────────────────────────────────

pub(super) fn on_resize_corner_tr_added(
    trigger: On<Add, EntityCardResizeCornerTR>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(
            |mut trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             entity_cards: Query<&EntityCard>,
             mut card_nodes: Query<(&mut Node, &ComputedNode), With<EntityCard>>| {
                trigger.propagate(false);
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(_) = entity_cards.get(p.0) else { return };
                let Ok((mut cn, computed)) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_tr(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    &mut cn,
                    computed,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             card_nodes: Query<(&EntityCard, &Node)>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok((ec, cn)) = card_nodes.get(p.0) else {
                    return;
                };
                corner_save(ec.entity_id, cn, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::NeResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}

// ── TL corner ─────────────────────────────────────────────────────────────────

pub(super) fn on_resize_corner_tl_added(
    trigger: On<Add, EntityCardResizeCornerTL>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(
            |mut trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             entity_cards: Query<&EntityCard>,
             mut card_nodes: Query<(&mut Node, &ComputedNode), With<EntityCard>>| {
                trigger.propagate(false);
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(_) = entity_cards.get(p.0) else { return };
                let Ok((mut cn, computed)) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_tl(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    &mut cn,
                    computed,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             card_nodes: Query<(&EntityCard, &Node)>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok((ec, cn)) = card_nodes.get(p.0) else {
                    return;
                };
                corner_save(ec.entity_id, cn, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::NwResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}
