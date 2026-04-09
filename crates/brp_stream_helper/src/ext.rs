use bevy::prelude::*;
use brp_helper::methods;
use brp_helper::types::{BrpGetComponentsWatch, BrpListComponentsWatch};
use serde_json::json;
use stream_helper::SseStream;

/// Extension trait for [`Commands`] that spawns BRP stream entities.
///
/// Each method builds the JSON-RPC `+watch` payload, spawns an [`SseStream<T>`]
/// entity, and returns its [`Entity`] id so the caller can chain observers:
///
/// ```ignore
/// let stream = commands.brp_watch_list_components(BRP_URL, entity_id);
/// commands.entity(stream)
///     .observe(|trigger: On<Insert, StreamData<BrpListComponentsWatch>>,
///               query: Query<&StreamData<BrpListComponentsWatch>>| {
///         if let Ok(data) = query.get(trigger.entity) {
///             for item in &data.0 { /* handle */ }
///         }
///     })
///     .observe(|_: On<Add, StreamDisconnected>| { /* reconnect */ });
/// ```
///
/// Despawn the entity (or insert [`AbortStream`]) to cancel the stream.
pub trait BrpStreamCommandsExt {
    /// `world.list_components+watch` — stream component list changes on an entity.
    ///
    /// Fires `On<Insert, StreamData<BrpListComponentsWatch>>` each frame with
    /// `added` / `removed` component type paths.
    fn brp_watch_list_components(&mut self, url: &str, entity_id: u64) -> Entity;

    /// `world.get_components+watch` — stream component value changes on an entity.
    ///
    /// Fires `On<Insert, StreamData<BrpGetComponentsWatch>>` each frame with
    /// the current reflected values of the requested components.
    ///
    /// Pass `strict: true` to fail if any component is missing or unreflectable.
    fn brp_watch_components(
        &mut self,
        url: &str,
        entity_id: u64,
        type_paths: &[&str],
        strict: bool,
    ) -> Entity;
}

impl BrpStreamCommandsExt for Commands<'_, '_> {
    fn brp_watch_list_components(&mut self, url: &str, entity_id: u64) -> Entity {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": methods::WORLD_LIST_COMPONENTS_WATCH,
            "params": { "entity": entity_id }
        });
        self.spawn(SseStream::<BrpListComponentsWatch>::new(
            url,
            body,
            format!("list-watch entity={entity_id}"),
        ))
        .id()
    }

    fn brp_watch_components(
        &mut self,
        url: &str,
        entity_id: u64,
        type_paths: &[&str],
        strict: bool,
    ) -> Entity {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": methods::WORLD_GET_COMPONENTS_WATCH,
            "params": {
                "entity": entity_id,
                "components": type_paths,
                "strict": strict
            }
        });
        self.spawn(SseStream::<BrpGetComponentsWatch>::new(
            url,
            body,
            format!("get-watch entity={entity_id}"),
        ))
        .id()
    }
}
