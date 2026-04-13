use crate::prelude::*;

// ── Data ──────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PinCardEntry {
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
pub struct PinCardTitle(pub u64);

#[derive(Component)]
pub struct PinCardHighlight {
    pub timer: Timer,
}

impl PinCardHighlight {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.2, TimerMode::Once),
        }
    }
}

/// Drives periodic BRP polling for a pincard's component data.
#[derive(Component)]
pub(super) struct PinCardPollTimer(pub(super) Timer);

/// Context stored on a `brp_list_components` request entity.
#[derive(Component)]
pub(super) struct PinCardListCtx {
    pub(super) entity_id: u64,
}

/// Context stored on a `brp_get_components` request entity.
#[derive(Component)]
pub(super) struct PinCardGetCtx {
    pub(super) entity_id: u64,
}

/// Button on a component header row that toggles its expanded state.
#[derive(Component, Clone, Default)]
pub(super) struct PinCardExpandToggle {
    pub(super) entity_id: u64,
    pub(super) type_path: String,
}

/// Right-edge drag handle for resizing the pincard width.
#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeHandle;

/// Left-edge drag handle for resizing the pincard width from the left.
#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeHandleLeft;

/// Bottom-edge drag handle for resizing the pincard height.
#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeHandleBottom;

/// Top-edge drag handle for resizing the pincard height from the top.
#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeHandleTop;

#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeCornerBR; // bottom-right

#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeCornerBL; // bottom-left

#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeCornerTR; // top-right

#[derive(Component, Clone, Default)]
pub(super) struct PinCardResizeCornerTL; // top-left

/// Marks the outer row node of the scrollable list so height resize can target it.
#[derive(Component, Clone)]
pub struct PinCardScrollOuter {
    pub entity_id: u64,
}

/// Marker on an editable field input in a pincard expanded row.
#[derive(Component, Clone, Default)]
pub(super) struct EditablePinCardField {
    pub(super) entity_id: u64,
    pub(super) type_path: String,
    pub(super) field_key: String,
}

/// Marker on the insert-component text input inside a pincard.
#[derive(Component, Clone, Default)]
pub(super) struct PinCardInsertField {
    pub(super) entity_id: u64,
}

/// Button that removes a component from a pincard entity when pressed.
#[derive(Component, Clone)]
pub(super) struct PinCardRemoveComponentButton {
    pub(super) entity_id: u64,
    pub(super) type_path: String,
}

// ── Resources ─────────────────────────────────────────────────────────────────

/// short_name -> full_type_path, populated from registry.schema on first PinCard spawn.
#[derive(Resource, Default)]
pub struct PinCardKnownMarkerComponents(pub HashMap<String, String>);

/// Which component rows are expanded, keyed by `entity_id → set of type_paths`.
#[derive(Resource, Default)]
pub struct PinCardExpandState(pub HashMap<u64, HashSet<String>>);

/// Last-received component data per entity, used for instant re-render on expand.
#[derive(Resource, Default)]
pub struct PinCardDataCache(pub HashMap<u64, serde_json::Map<String, serde_json::Value>>);

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn pincard_key(entity_id: u64) -> String {
    format!("pin-{}", entity_id)
}
