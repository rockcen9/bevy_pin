use bevy::prelude::*;
use json_rpc_helper::RpcRequest;
use serde_json::{Value, json};

use crate::methods;
use crate::types::*;

/// Extension trait for [`Commands`] that spawns BRP request entities.
///
/// Each method serializes the JSON-RPC payload, spawns an [`RpcRequest<T>`]
/// entity, and returns its [`Entity`] id so the caller can chain observers:
///
/// ```ignore
/// use brp_helper::{BrpCommandsExt, BrpListComponents, RpcResponse, TimeoutError};
///
/// let req = commands.brp_list_components(&server_url.0, entity_id);
/// commands.entity(req)
///     .observe(|trigger: On<Add, RpcResponse<BrpListComponents>>, q: Query<&RpcResponse<BrpListComponents>>, mut commands: Commands| {
///         let entity = trigger.entity();
///         if let Ok(resp) = q.get(entity) { ... }
///         commands.entity(entity).despawn();
///     })
///     .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
///         commands.entity(trigger.entity()).despawn();
///     });
/// ```
///
/// # Notes on watch methods
/// `world.get_components+watch` and `world.list_components+watch` are streaming
/// methods that push updates every frame. They require a different HTTP transport
/// and are **not** supported by the one-shot [`RpcRequest<T>`] model. Use the
/// method name constants from [`crate::methods`] directly if you need them.
pub trait BrpCommandsExt {
    // ── Component ─────────────────────────────────────────────────────────────

    /// `world.list_components` — list all component type paths on an entity.
    fn brp_list_components(&mut self, url: &str, entity_id: u64) -> Entity;

    /// `world.list_components` (no params) — list all registered component types globally.
    fn brp_list_all_component_types(&mut self, url: &str) -> Entity;

    /// `world.get_components` — fetch `type_paths` values from an entity.
    ///
    /// Pass `strict: true` to fail if any component is missing or unreflectable.
    fn brp_get_components(
        &mut self,
        url: &str,
        entity_id: u64,
        type_paths: &[String],
        strict: bool,
    ) -> Entity;

    /// `world.insert_components` — add/overwrite components on an entity.
    ///
    /// `components`: JSON object mapping `"TypePath"` → component value.
    fn brp_insert_components(&mut self, url: &str, entity_id: u64, components: Value) -> Entity;

    /// `world.remove_components` — remove components from an entity.
    fn brp_remove_components(
        &mut self,
        url: &str,
        entity_id: u64,
        type_paths: &[String],
    ) -> Entity;

    /// `world.mutate_components` — mutate a single field within a component.
    ///
    /// `path`: dot-separated path, e.g. `".translation.x"`.
    fn brp_mutate_component(
        &mut self,
        url: &str,
        entity_id: u64,
        component: &str,
        path: &str,
        value: Value,
    ) -> Entity;

    // ── Entity ────────────────────────────────────────────────────────────────

    /// `world.spawn_entity` — spawn a new entity with the given `components` JSON.
    ///
    /// `components`: JSON object mapping `"TypePath"` → component value.
    fn brp_spawn_entity(&mut self, url: &str, components: Value) -> Entity;

    /// `world.despawn_entity` — despawn an entity by id.
    fn brp_despawn_entity(&mut self, url: &str, entity_id: u64) -> Entity;

    /// `world.reparent_entities` — reparent one or more entities.
    ///
    /// Pass `parent: None` to detach from all parents.
    fn brp_reparent_entities(
        &mut self,
        url: &str,
        entities: &[u64],
        parent: Option<u64>,
    ) -> Entity;

    // ── Query ─────────────────────────────────────────────────────────────────

    /// `world.query` — query entities; pass the full `params` JSON object.
    ///
    /// ```ignore
    /// let params = json!({
    ///     "data": { "components": [], "option": "all", "has": [] },
    ///     "filter": { "with": ["TypePath"], "without": [] },
    ///     "strict": false
    /// });
    /// let req = commands.brp_world_query(&url, params);
    /// ```
    fn brp_world_query(&mut self, url: &str, params: Value) -> Entity;

    // ── Resource ──────────────────────────────────────────────────────────────

    /// `world.list_resources` — list all reflectable registered resource types.
    fn brp_list_resources(&mut self, url: &str) -> Entity;

    /// `world.get_resources` — get a resource by its fully-qualified `type_path`.
    fn brp_get_resources(&mut self, url: &str, type_path: &str) -> Entity;

    /// `world.insert_resources` — insert or overwrite a resource.
    ///
    /// `value`: the reflected resource value as JSON.
    fn brp_insert_resources(&mut self, url: &str, type_path: &str, value: Value) -> Entity;

    /// `world.remove_resources` — remove a resource from the world.
    fn brp_remove_resources(&mut self, url: &str, type_path: &str) -> Entity;

    /// `world.mutate_resources` — mutate a field within a resource.
    ///
    /// `field_path`: dot-separated path, e.g. `".my_field"`.
    fn brp_mutate_resource(
        &mut self,
        url: &str,
        type_path: &str,
        field_path: &str,
        value: Value,
    ) -> Entity;

    // ── Event ─────────────────────────────────────────────────────────────────

    /// `world.trigger_event` — trigger a reflected event.
    ///
    /// Requires the event type to implement `ReflectEvent`.
    fn brp_trigger_event(&mut self, url: &str, event_type_path: &str, value: Value) -> Entity;

    // ── Registry ──────────────────────────────────────────────────────────────

    /// `registry.schema` — get JSON schemas for reflected types.
    ///
    /// `params` controls filtering; pass `Value::Null` to fetch all schemas.
    ///
    /// ```ignore
    /// // Fetch schemas for a specific crate only
    /// let params = json!({ "with_crates": ["my_crate"] });
    /// let req = commands.brp_registry_schema(&url, params);
    /// ```
    fn brp_registry_schema(&mut self, url: &str, params: Value) -> Entity;

    // ── Heartbeat ─────────────────────────────────────────────────────────────

    /// `world.get` — heartbeat; any successful response means the server is up.
    fn brp_heartbeat(&mut self, url: &str) -> Entity;
}

// ── impl ─────────────────────────────────────────────────────────────────────

impl<'w, 's> BrpCommandsExt for Commands<'w, 's> {
    fn brp_list_components(&mut self, url: &str, entity_id: u64) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_LIST_COMPONENTS,
            "params": { "entity": entity_id }
        }));
        self.spawn(RpcRequest::<BrpListComponents>::new(url, payload)).id()
    }

    fn brp_list_all_component_types(&mut self, url: &str) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_LIST_COMPONENTS
        }));
        self.spawn(RpcRequest::<BrpListAllComponents>::new(url, payload)).id()
    }

    fn brp_get_components(
        &mut self,
        url: &str,
        entity_id: u64,
        type_paths: &[String],
        strict: bool,
    ) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_GET_COMPONENTS,
            "params": {
                "entity": entity_id,
                "components": type_paths,
                "strict": strict
            }
        }));
        self.spawn(RpcRequest::<BrpGetComponents>::new(url, payload)).id()
    }

    fn brp_insert_components(&mut self, url: &str, entity_id: u64, components: Value) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_INSERT_COMPONENTS,
            "params": { "entity": entity_id, "components": components }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_remove_components(
        &mut self,
        url: &str,
        entity_id: u64,
        type_paths: &[String],
    ) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_REMOVE_COMPONENTS,
            "params": { "entity": entity_id, "components": type_paths }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_mutate_component(
        &mut self,
        url: &str,
        entity_id: u64,
        component: &str,
        path: &str,
        value: Value,
    ) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_MUTATE_COMPONENTS,
            "params": {
                "entity": entity_id,
                "component": component,
                "path": path,
                "value": value
            }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_spawn_entity(&mut self, url: &str, components: Value) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_SPAWN_ENTITY,
            "params": { "components": components }
        }));
        self.spawn(RpcRequest::<BrpSpawnEntity>::new(url, payload)).id()
    }

    fn brp_despawn_entity(&mut self, url: &str, entity_id: u64) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_DESPAWN_ENTITY,
            "params": { "entity": entity_id }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_reparent_entities(
        &mut self,
        url: &str,
        entities: &[u64],
        parent: Option<u64>,
    ) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_REPARENT_ENTITIES,
            "params": { "entities": entities, "parent": parent }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_world_query(&mut self, url: &str, params: Value) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_QUERY,
            "params": params
        }));
        self.spawn(RpcRequest::<BrpWorldQuery>::new(url, payload)).id()
    }

    fn brp_list_resources(&mut self, url: &str) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_LIST_RESOURCES,
            "params": null
        }));
        self.spawn(RpcRequest::<BrpListResources>::new(url, payload)).id()
    }

    fn brp_get_resources(&mut self, url: &str, type_path: &str) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_GET_RESOURCES,
            "params": { "resource": type_path }
        }));
        self.spawn(RpcRequest::<BrpGetResources>::new(url, payload)).id()
    }

    fn brp_insert_resources(&mut self, url: &str, type_path: &str, value: Value) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_INSERT_RESOURCES,
            "params": { "resource": type_path, "value": value }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_remove_resources(&mut self, url: &str, type_path: &str) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_REMOVE_RESOURCES,
            "params": { "resource": type_path }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_mutate_resource(
        &mut self,
        url: &str,
        type_path: &str,
        field_path: &str,
        value: Value,
    ) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_MUTATE_RESOURCES,
            "params": {
                "resource": type_path,
                "path": field_path,
                "value": value
            }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_trigger_event(&mut self, url: &str, event_type_path: &str, value: Value) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::WORLD_TRIGGER_EVENT,
            "params": { "event": event_type_path, "value": value }
        }));
        self.spawn(RpcRequest::<BrpMutate>::new(url, payload)).id()
    }

    fn brp_registry_schema(&mut self, url: &str, params: Value) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 1,
            "method": methods::REGISTRY_SCHEMA,
            "params": params
        }));
        self.spawn(RpcRequest::<BrpSchema>::new(url, payload)).id()
    }

    fn brp_heartbeat(&mut self, url: &str) -> Entity {
        let payload = to_payload(json!({
            "jsonrpc": "2.0", "id": 0,
            "method": methods::WORLD_GET
        }));
        self.spawn(RpcRequest::<BrpHeartbeat>::new(url, payload)).id()
    }
}

/// Serializes `v` to bytes. The `json!()` macro only produces valid JSON so
/// this will never fail in practice.
fn to_payload(v: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&v).expect("BRP payload serialization failed")
}
