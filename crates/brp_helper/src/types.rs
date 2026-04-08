use serde::Deserialize;
use serde_json::Value;

// ── Component responses ───────────────────────────────────────────────────────

/// `world.list_components` (with entity) — list all component type paths on an entity.
///
/// `result`: array of fully-qualified type name strings.
#[derive(Deserialize, Debug)]
pub struct BrpListComponents {
    pub result: Vec<String>,
}

/// `world.list_components` (no entity) — list all registered component types globally.
///
/// `result`: array of fully-qualified type name strings.
#[derive(Deserialize, Debug)]
pub struct BrpListAllComponents {
    pub result: Vec<String>,
}

/// `world.list_components+watch` — streaming updates when an entity's component set changes.
///
/// Note: watch methods require streaming HTTP semantics, not a one-shot POST.
/// `result`: `{ "added": ["TypePath", ...], "removed": ["TypePath", ...] }`
#[derive(Deserialize, Debug)]
pub struct BrpListComponentsWatch {
    pub result: BrpListComponentsWatchResult,
}

#[derive(Deserialize, Debug)]
pub struct BrpListComponentsWatchResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

/// `world.get_components` — fetch component values from an entity.
///
/// `result`: `{ "components": { "TypePath": { ... } }, "errors": { "TypePath": "..." } }`
#[derive(Deserialize, Debug)]
pub struct BrpGetComponents {
    pub result: Value,
}

/// `world.get_components+watch` — streaming component value updates.
///
/// Note: watch methods require streaming HTTP semantics, not a one-shot POST.
/// `result`: `{ "components": { ... }, "removed": ["TypePath", ...], "errors": { ... } }`
#[derive(Deserialize, Debug)]
pub struct BrpGetComponentsWatch {
    pub result: Value,
}

// ── Query ─────────────────────────────────────────────────────────────────────

/// `world.query` — query entities matching component filters.
///
/// `result`: array of `{ "entity": <id>, "components": { ... }, "has": { ... } }`.
#[derive(Deserialize, Debug)]
pub struct BrpWorldQuery {
    pub result: Vec<BrpQueryEntry>,
}

#[derive(Deserialize, Debug)]
pub struct BrpQueryEntry {
    pub entity: u64,
    pub components: Value,
}

// ── Entity responses ──────────────────────────────────────────────────────────

/// `world.spawn_entity` — spawn a new entity.
///
/// `result`: `{ "entity": <id> }`
#[derive(Deserialize, Debug)]
pub struct BrpSpawnEntity {
    pub result: BrpSpawnResult,
}

#[derive(Deserialize, Debug)]
pub struct BrpSpawnResult {
    pub entity: u64,
}

// ── Resource responses ────────────────────────────────────────────────────────

/// `world.list_resources` — list all reflectable registered resource types.
///
/// `result`: array of fully-qualified type name strings.
#[derive(Deserialize, Debug)]
pub struct BrpListResources {
    pub result: Vec<String>,
}

/// `world.get_resources` — get a resource value by type path.
///
/// `result`: the reflected resource value.
#[derive(Deserialize, Debug)]
pub struct BrpGetResources {
    pub result: Value,
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// `registry.schema` — get JSON schemas for reflected types.
///
/// `result`: map of `"TypePath"` → schema object.
#[derive(Deserialize, Debug)]
pub struct BrpSchema {
    pub result: Value,
}

// ── Generic / null-result ─────────────────────────────────────────────────────

/// Generic response for BRP methods that return `null` or an opaque value.
///
/// Covers: `world.despawn_entity`, `world.insert_components`,
/// `world.remove_components`, `world.mutate_components`,
/// `world.reparent_entities`, `world.insert_resources`,
/// `world.remove_resources`, `world.mutate_resources`,
/// `world.trigger_event`.
#[derive(Deserialize, Debug)]
pub struct BrpMutate {
    #[serde(default)]
    pub result: Value,
}

/// `world.get` heartbeat — any successful parse = server is up.
#[derive(Deserialize, Debug)]
pub struct BrpHeartbeat {}
