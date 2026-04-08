/// BRP JSON-RPC method name constants.
///
/// Source: <https://docs.rs/bevy_remote/latest/bevy_remote/builtin_methods/>

// ── Entity ────────────────────────────────────────────────────────────────────
pub const WORLD_SPAWN_ENTITY: &str = "world.spawn_entity";
pub const WORLD_DESPAWN_ENTITY: &str = "world.despawn_entity";
pub const WORLD_REPARENT_ENTITIES: &str = "world.reparent_entities";

// ── Component ─────────────────────────────────────────────────────────────────
pub const WORLD_LIST_COMPONENTS: &str = "world.list_components";
/// Streaming variant — requires watching HTTP semantics, not a one-shot POST.
pub const WORLD_LIST_COMPONENTS_WATCH: &str = "world.list_components+watch";
pub const WORLD_GET_COMPONENTS: &str = "world.get_components";
/// Streaming variant — requires watching HTTP semantics, not a one-shot POST.
pub const WORLD_GET_COMPONENTS_WATCH: &str = "world.get_components+watch";
pub const WORLD_INSERT_COMPONENTS: &str = "world.insert_components";
pub const WORLD_REMOVE_COMPONENTS: &str = "world.remove_components";
pub const WORLD_MUTATE_COMPONENTS: &str = "world.mutate_components";

// ── Resource ──────────────────────────────────────────────────────────────────
pub const WORLD_LIST_RESOURCES: &str = "world.list_resources";
pub const WORLD_GET_RESOURCES: &str = "world.get_resources";
pub const WORLD_INSERT_RESOURCES: &str = "world.insert_resources";
pub const WORLD_REMOVE_RESOURCES: &str = "world.remove_resources";
pub const WORLD_MUTATE_RESOURCES: &str = "world.mutate_resources";

// ── Query / Event ─────────────────────────────────────────────────────────────
pub const WORLD_QUERY: &str = "world.query";
pub const WORLD_TRIGGER_EVENT: &str = "world.trigger_event";

// ── Registry ──────────────────────────────────────────────────────────────────
pub const REGISTRY_SCHEMA: &str = "registry.schema";

// ── RPC / Discovery ───────────────────────────────────────────────────────────
pub const RPC_DISCOVER: &str = "rpc.discover";

// ── Unofficial / heartbeat ────────────────────────────────────────────────────
/// Used as a heartbeat — any successful response indicates the server is up.
/// Not listed in official BRP docs but accepted by `bevy_remote`.
pub const WORLD_GET: &str = "world.get";
