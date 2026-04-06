use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, unbounded};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::time::Duration;

// --- System sets ---

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrpTimeoutSet;

// --- Markers ---

/// Auto-inserted via `#[require]` on [`BrpResponse<T>`].
/// Non-generic so [`tick_timeout_system`] can filter it without knowing `T`.
#[derive(Component, Default)]
pub struct RequestReceivedMarker;

/// Auto-inserted via `#[require]` on [`BrpRequest<T>`].
/// Ticked by [`tick_timeout_system`]; fires [`TimeoutError`] if it expires.
#[derive(Component)]
pub struct RequestTimeout(pub Timer);

impl RequestTimeout {
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
///     .spawn(BrpRequest::<MyResponse>::new(url, payload))
///     .observe(|trigger: On<Add, BrpResponse<MyResponse>>, ..| { .. })
///     .observe(|trigger: On<Add, TimeoutError>, ..| { .. });
/// ```
#[derive(Component)]
#[require(RequestTimeout::new(Duration::from_secs(5)))]
pub struct BrpRequest<T: DeserializeOwned + Send + Sync + 'static> {
    pub url: String,
    pub payload: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + Sync + 'static> BrpRequest<T> {
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
/// React via an entity-scoped observer or `Added<BrpResponse<T>>`.
/// The entity is **not** despawned automatically — the observer owns cleanup.
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
#[require(RequestReceivedMarker)]
pub struct BrpResponse<T: Send + Sync + 'static> {
    pub data: Result<T, String>,
}

// --- Internal per-type channel ---

#[derive(Resource)]
struct BrpQueue<T: Send + Sync + 'static> {
    tx: Sender<(Entity, Result<T, String>)>,
    rx: Receiver<(Entity, Result<T, String>)>,
}

impl<T: Send + Sync + 'static> Default for BrpQueue<T> {
    fn default() -> Self {
        // Unbounded: one message per request; receiver_system drains every frame.
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
}

// --- Systems / Observers ---

/// Fires the HTTP request the moment [`BrpRequest<T>`] is added to an entity.
/// Runs exactly once per entity — no sentinel component needed.
fn on_request_added<T: DeserializeOwned + Send + Sync + 'static>(
    trigger: On<Add, BrpRequest<T>>,
    query: Query<&BrpRequest<T>>,
    queue: Res<BrpQueue<T>>,
) {
    let entity = trigger.entity;
    let Ok(request) = query.get(entity) else {
        return;
    };

    let tx = queue.tx.clone();
    let http_req = ehttp::Request::post(&request.url, request.payload.clone());

    ehttp::fetch(http_req, move |result| {
        let data = match result {
            Ok(resp) => serde_json::from_slice::<T>(&resp.bytes).map_err(|e| e.to_string()),
            Err(e) => Err(e),
        };
        let _ = tx.send((entity, data));
    });
}

/// Delivers completed responses back onto their originating entity.
fn receiver_system<T: DeserializeOwned + Send + Sync + 'static>(
    mut commands: Commands,
    queue: Res<BrpQueue<T>>,
    entities: Query<Entity>,
) {
    while let Ok((entity, data)) = queue.rx.try_recv() {
        if entities.contains(entity) {
            commands.entity(entity).insert(BrpResponse::<T> { data });
        }
    }
}

fn tick_timeout_system(
    time: Res<Time>,
    mut query: Query<&mut RequestTimeout, (Without<TimeoutError>, Without<RequestReceivedMarker>)>,
) {
    for mut timeout in &mut query {
        timeout.0.tick(time.delta());
    }
}

fn garbage_collection_system(
    mut commands: Commands,
    query: Query<
        (Entity, &RequestTimeout),
        (Without<TimeoutError>, Without<RequestReceivedMarker>),
    >,
) {
    for (entity, timeout) in &query {
        if timeout.0.just_finished() {
            tracing::warn!("BRP request timeout for {:?}.", entity);
            commands.entity(entity).insert(TimeoutError);
        }
    }
}

// --- Plugins ---

/// Register once per response type `T`.
///
/// ```ignore
/// app.add_plugins(BrpEndpointPlugin::<MyResponse>::default());
/// ```
pub struct BrpEndpointPlugin<T: DeserializeOwned + Send + Sync + 'static>(PhantomData<T>);

impl<T: DeserializeOwned + Send + Sync + 'static> Default for BrpEndpointPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: DeserializeOwned + Send + Sync + 'static> Plugin for BrpEndpointPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<BrpQueue<T>>()
            .add_observer(on_request_added::<T>)
            .add_systems(Update, receiver_system::<T>.before(BrpTimeoutSet))
            .add_systems(
                Update,
                ApplyDeferred
                    .after(receiver_system::<T>)
                    .before(BrpTimeoutSet),
            );
    }
}

/// Top-level plugin. Add once. Then add [`BrpEndpointPlugin<T>`] per response type.
///
/// Owns the timeout systems shared across all `T`:
/// - `tick_timeout_system` — advances [`RequestTimeout`] timers each frame
/// - `garbage_collection_system` — inserts [`TimeoutError`] when a timer fires
pub struct RemoteHelperPlugin;

impl Plugin for RemoteHelperPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, BrpTimeoutSet).add_systems(
            Update,
            (tick_timeout_system, garbage_collection_system)
                .chain()
                .in_set(BrpTimeoutSet),
        );
    }
}
