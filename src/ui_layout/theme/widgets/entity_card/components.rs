use crate::prelude::*;

// ── Data ──────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct EntityCardEntry {
    pub entity_id: u64,
    pub label: String,
    pub left: f32,
    pub top: f32,
    #[serde(default = "default_card_width")]
    pub width: f32,
    #[serde(default = "default_card_height")]
    pub height: f32,
}

fn default_card_width() -> f32 {
    280.0
}

fn default_card_height() -> f32 {
    300.0
}

// ── Constants ─────────────────────────────────────────────────────────────────

/// Sentinel type_path stored in `PinCardExpandState` for the insert-component row.
pub(super) const INSERT_SENTINEL: &str = "__insert__";

// ── Components ────────────────────────────────────────────────────────────────

/// Data component on the outer card container — carries entity_id and initial height.
#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct EntityCard {
    pub entity_id: u64,
    pub height: f32,
}

/// Marker on the header row of an entity card.
#[derive(Component, Clone, Default)]
pub struct EntityCardHeader;

/// Place on the header row (via `drag_bundle`) to enable drag-to-move behaviour.
#[derive(Component, Clone, Default)]
pub struct DragHandle;

#[derive(Component, Clone, Default, Reflect)]
pub struct EntityCardTitle(pub u64);

#[derive(Component)]
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

/// Drives periodic BRP polling for a pincard's component data.
#[derive(Component)]
pub(super) struct EntityCardPollTimer(pub(super) Timer);

/// Context stored on a `brp_list_components` request entity.
#[derive(Component)]
pub(super) struct EntityCardListCtx {
    pub(super) entity_id: u64,
}

/// Context stored on a `brp_get_components` request entity.
#[derive(Component)]
pub(super) struct EntityCardGetCtx {
    pub(super) entity_id: u64,
}

/// Button on a component header row that toggles its expanded state.
#[derive(Component, Clone, Default)]
pub(super) struct EntityCardExpandToggle {
    pub(super) entity_id: u64,
    pub(super) type_path: String,
}

/// Right-edge drag handle for resizing the pincard width.
#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeHandle;

/// Left-edge drag handle for resizing the pincard width from the left.
#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeHandleLeft;

/// Bottom-edge drag handle for resizing the pincard height.
#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeHandleBottom;

/// Top-edge drag handle for resizing the pincard height from the top.
#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeHandleTop;

#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeCornerBR; // bottom-right

#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeCornerBL; // bottom-left

#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeCornerTR; // top-right

#[derive(Component, Clone, Default)]
pub(super) struct EntityCardResizeCornerTL; // top-left

/// Marks the outer row node of the scrollable list so height resize can target it.
#[derive(Component, Clone)]
pub struct EntityCardScrollOuter {
    pub entity_id: u64,
}

/// Marker on an editable field input in a pincard expanded row.
#[derive(Component, Clone, Default)]
pub(super) struct EditableEntityCardField {
    pub(super) entity_id: u64,
    pub(super) type_path: String,
    pub(super) field_key: String,
}

/// Marker on the insert-component text input inside a pincard.
#[derive(Component, Clone, Default)]
pub(super) struct EntityCardInsertField {
    pub(super) entity_id: u64,
}

/// Button that removes a component from a pincard entity when pressed.
#[derive(Component, Clone)]
pub(super) struct EntityCardRemoveComponentButton {
    pub(super) entity_id: u64,
    pub(super) type_path: String,
}

// ── Resources ─────────────────────────────────────────────────────────────────

/// short_name -> full_type_path, populated from registry.schema on first PinCard spawn.
#[derive(Resource, Default)]
pub struct EntityCardKnownMarkerComponents(pub HashMap<String, String>);

/// Which component rows are expanded, keyed by `entity_id → set of type_paths`.
#[derive(Resource, Default)]
pub struct EntityCardExpandState(pub HashMap<u64, HashSet<String>>);

/// Last-received component data per entity, used for instant re-render on expand.
#[derive(Resource, Default)]
pub struct EntityCardDataCache(pub HashMap<u64, serde_json::Map<String, serde_json::Value>>);

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn entity_card_key(entity_id: u64) -> String {
    format!("pin-{}", entity_id)
}
