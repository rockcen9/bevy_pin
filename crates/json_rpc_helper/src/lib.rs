use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, unbounded};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::time::Duration;
use tracing::{debug, warn};

// --- System sets ---

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RpcTimeoutSet;

// --- Markers ---

/// Auto-inserted via `#[require]` on [`RpcResponse<T>`].
/// Non-generic so [`tick_timeout_system`] can filter it without knowing `T`.
#[derive(Component, Default)]
pub struct RpcRequestReceivedMarker;

/// Auto-inserted via `#[require]` on [`RpcRequest<T>`].
/// Ticked by [`tick_timeout_system`]; fires [`TimeoutError`] if it expires.
#[derive(Component)]
pub struct RpcRequestTimeout(pub Timer);

impl RpcRequestTimeout {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

/// Inserted by `garbage_collection_system` when a [`RequestTimeout`] expires.
#[derive(Component)]
pub struct TimeoutError;

// --- Request / Response ---

/// Spawn this component to trigger an HTTP POST request.
///
/// [`RequestTimeout`] is auto-inserted via `#[require]` (5 s default).
/// The observer fires the HTTP call immediately on `Add`.
///
/// # Example
/// ```ignore
/// commands
///     .spawn(RpcRequest::<MyResponse>::new(url, payload))
///     .observe(|trigger: On<Add, RpcResponse<MyResponse>>, ..| { .. })
///     .observe(|trigger: On<Add, TimeoutError>, ..| { .. });
/// ```
#[derive(Component)]
#[component(storage = "SparseSet")]
#[require(RpcRequestTimeout::new(Duration::from_secs(5)))]
pub struct RpcRequest<T: DeserializeOwned + Send + Sync + 'static> {
    pub url: String,
    pub payload: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync + 'static> RpcRequest<T> {
    pub fn new(url: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            url: url.into(),
            payload,
            _marker: PhantomData,
        }
    }
}

/// Inserted on the same entity when the HTTP response arrives and is parsed.
///
/// [`RequestReceivedMarker`] is auto-inserted via `#[require]`, stopping the
/// timeout from ticking.
///
/// React via an entity-scoped observer or `Added<RpcResponse<T>>`.
/// The entity is **not** despawned automatically — the observer owns cleanup.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
#[require(RpcRequestReceivedMarker)]
pub struct RpcResponse<T: Send + Sync + 'static> {
    pub data: Result<T, String>,
}

// --- Internal per-type channel ---

#[derive(Resource)]
struct RpcQueue<T: Send + Sync + 'static> {
    tx: Sender<(Entity, Result<T, String>)>,
    rx: Receiver<(Entity, Result<T, String>)>,
}

impl<T: Send + Sync + 'static> Default for RpcQueue<T> {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
}

// --- HTTP ---

async fn post_json<T: DeserializeOwned>(url: String, payload: Vec<u8>) -> Result<T, String> {
    reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .body(payload)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<T>()
        .await
        .map_err(|e| e.to_string())
}

fn spawn_http_task<T: DeserializeOwned + Send + Sync + 'static>(
    entity: Entity,
    url: String,
    payload: Vec<u8>,
    tx: Sender<(Entity, Result<T, String>)>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio rt");
        rt.block_on(async move {
            debug!("rpc [{entity:?}]: POST {url}");
            let data = post_json::<T>(url, payload).await;
            match &data {
                Ok(_) => debug!("rpc [{entity:?}]: response ok"),
                Err(e) => debug!("rpc [{entity:?}]: response err: {e}"),
            }
            let _ = tx.send((entity, data));
        });
    });

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        debug!("rpc [{entity:?}]: POST {url}");
        let data = post_json::<T>(url, payload).await;
        match &data {
            Ok(_) => debug!("rpc [{entity:?}]: response ok"),
            Err(e) => debug!("rpc [{entity:?}]: response err: {e}"),
        }
        let _ = tx.send((entity, data));
    });
}

// --- Systems / Observers ---

/// Fires the HTTP request the moment [`RpcRequest<T>`] is added to an entity.
fn on_request_added<T: DeserializeOwned + Send + Sync + 'static>(
    trigger: On<Add, RpcRequest<T>>,
    query: Query<&RpcRequest<T>>,
    queue: Res<RpcQueue<T>>,
) {
    let entity = trigger.entity;
    let Ok(request) = query.get(entity) else {
        return;
    };

    debug!("rpc [{entity:?}]: request spawned → {}", request.url);
    spawn_http_task::<T>(
        entity,
        request.url.clone(),
        request.payload.clone(),
        queue.tx.clone(),
    );
}

/// Delivers completed responses back onto their originating entity.
fn receiver_system<T: DeserializeOwned + Send + Sync + 'static>(
    mut commands: Commands,
    queue: Res<RpcQueue<T>>,
    entities: Query<Entity>,
) {
    while let Ok((entity, data)) = queue.rx.try_recv() {
        if entities.contains(entity) {
            debug!("rpc [{entity:?}]: delivering response to entity");
            commands.entity(entity).insert(RpcResponse::<T> { data });
        } else {
            debug!("rpc [{entity:?}]: response arrived but entity no longer exists, dropping");
        }
    }
}

fn tick_timeout_system(
    time: Res<Time>,
    mut query: Query<
        &mut RpcRequestTimeout,
        (Without<TimeoutError>, Without<RpcRequestReceivedMarker>),
    >,
) {
    for mut timeout in &mut query {
        timeout.0.tick(time.delta());
    }
}

fn garbage_collection_system(
    mut commands: Commands,
    query: Query<
        (Entity, &RpcRequestTimeout),
        (Without<TimeoutError>, Without<RpcRequestReceivedMarker>),
    >,
) {
    for (entity, timeout) in &query {
        if timeout.0.just_finished() {
            warn!("rpc [{entity:?}]: request timed out");
            commands.entity(entity).insert(TimeoutError);
        }
    }
}

// --- Plugins ---

/// Register once per response type `T`.
///
/// ```ignore
/// app.add_plugins(RpcEndpointPlugin::<MyResponse>::default());
/// ```
pub struct RpcEndpointPlugin<T: DeserializeOwned + Send + Sync + 'static>(PhantomData<T>);

impl<T: DeserializeOwned + Send + Sync + 'static> Default for RpcEndpointPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: DeserializeOwned + Send + Sync + 'static> Plugin for RpcEndpointPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<RpcQueue<T>>()
            .add_observer(on_request_added::<T>)
            .add_systems(Update, receiver_system::<T>.before(RpcTimeoutSet))
            .add_systems(
                Update,
                ApplyDeferred
                    .after(receiver_system::<T>)
                    .before(RpcTimeoutSet),
            );
    }
}

/// Top-level plugin. Add once. Then add [`RpcEndpointPlugin<T>`] per response type.
pub struct RemoteHelperPlugin;

impl Plugin for RemoteHelperPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, RpcTimeoutSet).add_systems(
            Update,
            (tick_timeout_system, garbage_collection_system)
                .chain()
                .in_set(RpcTimeoutSet),
        );
    }
}
