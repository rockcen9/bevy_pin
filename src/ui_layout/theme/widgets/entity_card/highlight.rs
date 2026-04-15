use crate::prelude::*;
use crate::ui_layout::theme::palette::{COLOR_HEADER_BG, COLOR_PAUSED};

// ── Component ─────────────────────────────────────────────────────────────────

#[derive(Component, Default, Clone)]
pub struct EntityCardHighlight {
    pub timer: Timer,
}

impl EntityCardHighlight {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.2, TimerMode::Once),
        }
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub fn plugin(app: &mut App) {
    app.add_observer(on_highlight_component_added)
        .add_observer(on_highlight_component_removed);
    app.add_systems(
        Update,
        (
            debug_highlight_added,
            drive_pincard_highlight.run_if(in_state(SidebarState::Pinboard)),
        ),
    );
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn on_highlight_component_added(
    trigger: On<Add, EntityCardHighlight>,
    bg: Query<Option<&BackgroundColor>>,
) {
    let entity = trigger.entity;
    let has_bg = bg.get(entity).ok().and_then(|b| b).is_some();
    debug!(
        "on_highlight_component_added: {:?} has_background_color={}",
        entity, has_bg
    );
}

fn on_highlight_component_removed(
    trigger: On<Remove, EntityCardHighlight>,
    q: Query<(Option<&EntityCardHeader>, Option<&EntityCardTitle>)>,
) {
    let entity = trigger.entity;
    match q.get(entity) {
        Ok((header, title)) => debug!(
            "on_highlight_component_removed: {:?} has_header={} has_title={}",
            entity,
            header.is_some(),
            title.is_some()
        ),
        Err(_) => debug!(
            "on_highlight_component_removed: {:?} entity NOT in world",
            entity
        ),
    }
}

fn debug_highlight_added(
    with_bg: Query<Entity, (Added<EntityCardHighlight>, With<BackgroundColor>)>,
    without_bg: Query<Entity, (Added<EntityCardHighlight>, Without<BackgroundColor>)>,
) {
    for entity in &with_bg {
        debug!(
            "debug_highlight_added: {:?} has BackgroundColor — query should match",
            entity
        );
    }
    for entity in &without_bg {
        debug!(
            "debug_highlight_added: {:?} MISSING BackgroundColor — query will NOT match",
            entity
        );
    }
}

pub(super) fn drive_pincard_highlight(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut BackgroundColor, &mut EntityCardHighlight)>,
) {
    let count = q.iter().count();
    if count > 0 {
        debug!("drive_pincard_highlight: {} entities with highlight", count);
    }
    for (entity, mut bg, mut highlight) in &mut q {
        highlight.timer.tick(time.delta());
        let t = highlight.timer.fraction();
        let start = COLOR_PAUSED.to_srgba();
        let end = COLOR_HEADER_BG.to_srgba();
        let new_color = Color::srgba(
            start.red + (end.red - start.red) * t,
            start.green + (end.green - start.green) * t,
            start.blue + (end.blue - start.blue) * t,
            start.alpha + (end.alpha - start.alpha) * t,
        );
        debug!(
            "drive_pincard_highlight: entity={:?} t={:.3} color={:?} bg_before={:?}",
            entity, t, new_color, bg.0
        );
        bg.0 = new_color;
        debug!(
            "drive_pincard_highlight: entity={:?} bg_after={:?}",
            entity, bg.0
        );
        if highlight.timer.just_finished() {
            debug!(
                "drive_pincard_highlight: entity={:?} timer finished, removing",
                entity
            );
            commands.entity(entity).remove::<EntityCardHighlight>();
        }
    }
}
