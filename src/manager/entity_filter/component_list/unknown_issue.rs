use crate::manager::entity_filter::component_list::list::{
    ComponentDataState, RemoveComponentButton,
};
use crate::prelude::*;
use crate::ui_layout::theme::widgets::global_message::show_global_message;

const STALE_TIMEOUT_SECS: f32 = 0.5;
/// Drop a pending entry if it lingers this long with no resolution (avoids unbounded growth).
const PENDING_MAX_SECS: f32 = 10.0;

const STALE_MESSAGE: &str = r#"The stream seems to be having a little trouble; please try restarting the game. To keep things running smoothly, it's best to avoid using 'Entity Filter' and 'New Scene' in two browsers at once."#;
// ── Resource ───────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct PendingRemovalTracker {
    pending: Vec<PendingRemoval>,
}

struct PendingRemoval {
    entity_id: u64,
    type_path: String,
    elapsed: f32,
    warned: bool,
}

// ── Plugin ─────────────────────────────────────────────────────────────────

pub fn plugin(app: &mut App) {
    app.init_resource::<PendingRemovalTracker>()
        .add_systems(Update, (track_remove_presses, check_stale_removals).chain());
}

// ── Systems ────────────────────────────────────────────────────────────────

/// Records each remove-button press so we can watch for a stream acknowledgement.
fn track_remove_presses(
    buttons: Query<(&Interaction, &RemoveComponentButton), (Changed<Interaction>, With<Button>)>,
    mut tracker: ResMut<PendingRemovalTracker>,
) {
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            tracker.pending.push(PendingRemoval {
                entity_id: btn.entity_id,
                type_path: btn.type_path.clone(),
                elapsed: 0.0,
                warned: false,
            });
        }
    }
}

/// Checks whether the stream reflected the removal within `STALE_TIMEOUT_SECS`.
/// If not, shows a global warning message.
fn check_stale_removals(
    mut tracker: ResMut<PendingRemovalTracker>,
    state: Res<ComponentDataState>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    tracker.pending.retain_mut(|pending| {
        pending.elapsed += dt;

        let still_present = matches!(
            &*state,
            ComponentDataState::Ready { entity_id, type_paths, .. }
            if *entity_id == pending.entity_id && type_paths.contains(&pending.type_path)
        );

        if !still_present {
            // Stream already delivered the removal — all good, drop entry.
            return false;
        }

        if pending.elapsed >= STALE_TIMEOUT_SECS && !pending.warned {
            pending.warned = true;
            warn!(
                "stale_bug: remove of '{}' on entity #{} not reflected after {:.1}s — stream may be broken",
                pending.type_path, pending.entity_id, STALE_TIMEOUT_SECS
            );
            show_global_message(STALE_MESSAGE, &mut commands);
        }

        // Keep tracking until the entry is too old; then drop silently.
        pending.elapsed < PENDING_MAX_SECS
    });
}
