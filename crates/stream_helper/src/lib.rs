use crossbeam_channel::Sender;
use futures_util::StreamExt;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::marker::PhantomData;
#[cfg(target_arch = "wasm32")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tracing::{debug, error, trace, warn};

use bevy::prelude::*;

// ── Abort handle ───────────────────────────────────────────────────────────

/// Cancels the associated stream when dropped.
///
/// On native the drop closes a tokio oneshot channel, which the stream loop
/// detects at the next `tokio::select!` poll — immediately, even if the
/// server hasn't sent any new data.
///
/// On WASM there is no explicit cancel mechanism; the stream exits on the next
/// `sender.send()` failure (when the crossbeam `Receiver` was already dropped).
pub struct StreamAbortHandle {
    #[cfg(not(target_arch = "wasm32"))]
    _cancel_tx: tokio::sync::oneshot::Sender<()>,
    #[cfg(target_arch = "wasm32")]
    cancel: Arc<AtomicBool>,
}

impl StreamAbortHandle {
    #[cfg(not(target_arch = "wasm32"))]
    fn new(cancel_tx: tokio::sync::oneshot::Sender<()>) -> Self {
        Self {
            _cancel_tx: cancel_tx,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn new(cancel: Arc<AtomicBool>) -> Self {
        Self { cancel }
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for StreamAbortHandle {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

// ── NDJSON/SSE stream readers ──────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
async fn read_sse_stream_native<T: DeserializeOwned + Send>(
    response: reqwest::Response,
    sender: Sender<T>,
    label: String,
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let mut stream = response.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();

    debug!("{label}: stream opened");

    loop {
        tokio::select! {
            biased;
            _ = &mut cancel_rx => {
                debug!("{label}: stream aborted (handle dropped)");
                break;
            }
            chunk_result = stream.next() => {
                match chunk_result {
                    None => break,
                    Some(Err(e)) => {
                        warn!("{label}: stream error: {e}");
                        break;
                    }
                    Some(Ok(bytes)) => {
                        buffer.extend_from_slice(&bytes);
                        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                            let line_bytes = buffer[..pos].to_vec();
                            buffer.drain(..=pos);
                            let line = String::from_utf8_lossy(&line_bytes);
                            let trimmed = line.trim();
                            let json_str = trimmed.strip_prefix("data:").unwrap_or(trimmed).trim();
                            if json_str.is_empty() {
                                continue;
                            }
                            trace!("{label}: raw line: {json_str}");
                            match serde_json::from_str::<T>(json_str) {
                                Ok(parsed) => {
                                    if sender.send(parsed).is_err() {
                                        debug!("{label}: receiver dropped, aborting stream");
                                        return;
                                    }
                                }
                                Err(e) => error!("{label}: json parse error: {e} | raw: {json_str:?}"),
                            }
                        }
                    }
                }
            }
        }
    }

    debug!("{label}: stream closed");
}

#[cfg(target_arch = "wasm32")]
async fn read_sse_stream_wasm<T: DeserializeOwned + Send>(
    response: reqwest::Response,
    sender: Sender<T>,
    label: String,
    cancel: Arc<AtomicBool>,
) {
    let mut stream = response.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();

    debug!("{label}: stream opened");

    while !cancel.load(Ordering::Relaxed) {
        let chunk_result = match stream.next().await {
            Some(r) => r,
            None => break,
        };
        let bytes = match chunk_result {
            Ok(b) => b,
            Err(e) => {
                warn!("{label}: stream error: {e}");
                break;
            }
        };

        buffer.extend_from_slice(&bytes);

        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_bytes = buffer[..pos].to_vec();
            buffer.drain(..=pos);
            let line = String::from_utf8_lossy(&line_bytes);
            let trimmed = line.trim();
            let json_str = trimmed.strip_prefix("data:").unwrap_or(trimmed).trim();
            if json_str.is_empty() {
                continue;
            }
            match serde_json::from_str::<T>(json_str) {
                Ok(parsed) => {
                    if sender.send(parsed).is_err() {
                        debug!("{label}: receiver dropped, aborting stream");
                        return;
                    }
                }
                Err(e) => error!("{label}: json parse error: {e} | raw: {json_str:?}"),
            }
        }
    }

    debug!("{label}: stream closed");
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Spawns a background task that POSTs `body` to `url` and reads the response
/// as an NDJSON/SSE stream, forwarding each parsed `T` to `sender`.
/// Returns a [`StreamAbortHandle`] — drop it to cancel the stream.
///
/// Prefer [`SseStream`] for Bevy ECS integration.
#[cfg(not(target_arch = "wasm32"))]
pub fn start_sse_stream<T: DeserializeOwned + Send + 'static>(
    url: String,
    body: Value,
    label: String,
    sender: Sender<T>,
) -> StreamAbortHandle {
    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("stream_helper: failed to build Tokio runtime");
        rt.block_on(async {
            let client = reqwest::Client::new();
            match client.post(&url).json(&body).send().await {
                Ok(r) => read_sse_stream_native(r, sender, label, cancel_rx).await,
                Err(e) => warn!("SSE stream request failed: {e}"),
            }
        });
    });
    StreamAbortHandle::new(cancel_tx)
}

#[cfg(target_arch = "wasm32")]
pub fn start_sse_stream<T: DeserializeOwned + Send + 'static>(
    url: String,
    body: Value,
    label: String,
    sender: Sender<T>,
) -> StreamAbortHandle {
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_task = Arc::clone(&cancel);
    bevy::tasks::IoTaskPool::get()
        .spawn(async move {
            let client = reqwest::Client::new();
            match client.post(&url).json(&body).send().await {
                Ok(r) => read_sse_stream_wasm(r, sender, label, cancel_task).await,
                Err(e) => warn!("SSE stream request failed: {e}"),
            }
        })
        .detach();
    StreamAbortHandle::new(cancel)
}

// ── ECS layer ──────────────────────────────────────────────────────────────

/// Spawn this component to open an SSE stream. Chain `.observe()` to react
/// to incoming data.
///
/// The stream is cancelled automatically when the entity is despawned
/// (the [`StreamAbortHandle`] drops with it). To abort from another system,
/// insert [`AbortStream`] on the entity.
///
/// # Example
/// ```ignore
/// commands
///     .spawn(SseStream::<MyType>::new(url, body, "my-stream"))
///     .observe(|trigger: On<Insert, StreamData<MyType>>,
///               query: Query<&StreamData<MyType>>| {
///         if let Ok(data) = query.get(trigger.entity) {
///             for item in &data.0 { /* handle item */ }
///         }
///     });
/// ```
#[derive(Component)]
pub struct SseStream<T: Send + Sync + 'static> {
    receiver: crossbeam_channel::Receiver<T>,
    _abort_handle: StreamAbortHandle,
}

impl<T: DeserializeOwned + Send + Sync + 'static> SseStream<T> {
    pub fn new(url: impl Into<String>, body: Value, label: impl Into<String>) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(64);
        let _abort_handle = start_sse_stream(url.into(), body, label.into(), tx);
        Self {
            receiver: rx,
            _abort_handle,
        }
    }
}

/// Inserted on the [`SseStream`] entity each frame when new data arrives.
/// Contains all items received since the last frame.
///
/// Reacts with `On<Insert, StreamData<T>>`.
#[derive(Component)]
pub struct StreamData<T: Send + Sync + 'static>(pub Vec<T>);

/// Insert this component on an [`SseStream`] entity to cancel and despawn it.
///
/// ```ignore
/// commands.entity(stream_entity).insert(AbortStream);
/// ```
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AbortStream;

/// Inserted on the [`SseStream`] entity when the background stream closes
/// (server disconnected, network error, or end of stream).
/// The channel receiver returns `Err(TryRecvError::Disconnected)` at that point.
///
/// React with `On<Add, StreamDisconnected>` to handle reconnect logic or cleanup.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct StreamDisconnected;

fn pump_stream_system<T: Send + Sync + 'static>(
    mut commands: Commands,
    active: Query<(Entity, &SseStream<T>), (Without<AbortStream>, Without<StreamDisconnected>)>,
    aborted: Query<Entity, (With<SseStream<T>>, With<AbortStream>)>,
) {
    for entity in aborted.iter() {
        commands.entity(entity).despawn();
    }

    for (entity, stream) in active.iter() {
        let mut items = Vec::new();
        loop {
            match stream.receiver.try_recv() {
                Ok(data) => items.push(data),
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    commands.entity(entity).insert(StreamDisconnected);
                    break;
                }
            }
        }
        if !items.is_empty() {
            commands.entity(entity).insert(StreamData(items));
        }
    }
}

/// Bevy plugin that drives [`SseStream<T>`] entities.
///
/// Register once per stream type:
/// ```ignore
/// app.add_plugins(StreamPlugin::<MyType>::default());
/// ```
pub struct StreamPlugin<T: Send + Sync + 'static>(PhantomData<fn() -> T>);

impl<T: Send + Sync + 'static> Default for StreamPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: DeserializeOwned + Send + Sync + 'static> Plugin for StreamPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pump_stream_system::<T>);
    }
}
