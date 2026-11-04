//! Ergonomic watcher utilities for Relm4 components.
//!
//! Provides the [`watch!`] macro for reactive stream watching with automatic
//! shutdown handling, stream merging, and error logging.
//!
//! For watchers that need to be cancelled before component shutdown (e.g., when
//! a device changes), use [`watch_cancellable!`] with a [`CancellationToken`].
//!
//! [`CancellationToken`]: tokio_util::sync::CancellationToken

use std::{future, pin::Pin};

use futures::stream::{self, SelectAll, Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;
use wayle_config::SubscribeChanges;

/// Manages a cancellable watcher lifecycle.
///
/// Encapsulates the pattern of cancelling an existing watcher before spawning
/// a new one. Call [`reset`](Self::reset) to cancel any active watcher and
/// obtain a fresh token for the new watcher.
///
/// # Example
///
/// ```ignore
/// struct MyComponent {
///     device_watcher: WatcherToken,
/// }
///
/// // When device changes:
/// let token = self.device_watcher.reset();
/// Self::spawn_device_watchers(&sender, &device, token);
/// ```
#[derive(Debug, Default)]
pub struct WatcherToken(Option<CancellationToken>);

impl WatcherToken {
    /// Creates an empty watcher token with no active watcher.
    pub fn new() -> Self {
        Self(None)
    }

    /// Cancels any existing watcher and returns a fresh token.
    ///
    /// The returned token should be passed to `watch_cancellable!` or used
    /// directly with `token.cancelled()` in a `tokio::select!`.
    pub fn reset(&mut self) -> CancellationToken {
        if let Some(token) = self.0.take() {
            token.cancel();
        }
        let token = CancellationToken::new();
        self.0 = Some(token.clone());
        token
    }
}

impl Drop for WatcherToken {
    fn drop(&mut self) {
        if let Some(token) = self.0.take() {
            token.cancel();
        }
    }
}

/// Converts a [`SubscribeChanges`] implementor into a stream.
///
/// Bridges the channel-based `subscribe_changes` API with the stream-based
/// `watch!` macro. The returned stream is `'static` and does not borrow the
/// subscribable - it spawns internal watchers that send to the stream.
pub fn changes_stream<T: SubscribeChanges>(subscribable: &T) -> UnboundedReceiverStream<()> {
    let (tx, rx) = mpsc::unbounded_channel();
    subscribable.subscribe_changes(tx);
    UnboundedReceiverStream::new(rx)
}

/// Type alias for boxed streams used internally by the watch macro.
pub type BoxedStream = Pin<Box<dyn Stream<Item = ()> + Send>>;

/// Emits for every item of `stream` whose `key` differs from the previous
/// item's. The first item is compared with `start`, or always emitted if
/// `start` is `None`.
///
/// A property's `watch()` stream yields the value it has when the stream is
/// first polled, so with no `start` a change made in between is never
/// missed. A caller that has already acted on the value it read can pass that
/// value's key as `start` to skip the redundant first emission.
pub fn key_changes<T, K>(
    stream: impl Stream<Item = T> + Send + 'static,
    start: Option<K>,
    key: impl Fn(&T) -> K + Send + 'static,
) -> impl Stream<Item = ()> + Send + 'static
where
    K: PartialEq + Send + 'static,
{
    let mut last = start;
    stream.filter_map(move |item| {
        let current = key(&item);
        let changed = last.as_ref() != Some(&current);
        if changed {
            last = Some(current);
        }
        future::ready(changed.then_some(()))
    })
}

/// Emits whenever one of the streams `each` makes for the elements of the
/// latest list from `lists` emits. The element streams are made afresh for
/// each new list; streams that emit their current state first (such as
/// [`key_changes`] with no start) then miss nothing while re-subscribing.
pub fn each_changes<T, S>(
    lists: impl Stream<Item = Vec<T>> + Send + 'static,
    each: impl Fn(&T) -> S + Send + 'static,
) -> impl Stream<Item = ()> + Send + 'static
where
    T: Send + 'static,
    S: Stream<Item = ()> + Send + 'static,
{
    let lists = Box::pin(lists);
    let elements: SelectAll<Pin<Box<S>>> = SelectAll::new();

    stream::unfold(
        (lists, elements, each),
        |(mut lists, mut elements, each)| async move {
            loop {
                tokio::select! {
                    list = lists.next() => {
                        elements = list?.iter().map(|element| Box::pin(each(element))).collect();
                    }
                    Some(()) = elements.next() => return Some(((), (lists, elements, each))),
                }
            }
        },
    )
}

/// Watches multiple streams and runs a handler when any emits.
///
/// Automatically handles:
/// - Stream pinning and type erasure
/// - Merging multiple streams with `select_all`
/// - Shutdown handling via Relm4's shutdown receiver
///
/// # Patterns
///
/// ## Auto-send
///
/// Handler returns `Result<T, E>`. On `Ok(value)`, sends `Cmd::Variant(value)`.
/// On `Err`, logs and continues.
///
/// ```ignore
/// watch!(sender, [streams], || fallible_handler() => Cmd::Variant);
/// ```
///
/// ## Manual
///
/// Handler receives the command sender for full control. Supports conditional
/// sends, multiple commands, or custom error handling.
///
/// ```ignore
/// watch!(sender, [streams], |out| {
///     if condition {
///         let _ = out.send(Cmd::A(value));
///     }
///     let _ = out.send(Cmd::B);
/// });
/// ```
///
/// # Examples
///
/// ```ignore
/// watch!(sender,
///     [changes_stream(&config.styling), wallpaper.watch_extraction()],
///     move || compile_css(&config) => ShellCmd::CssRecompiled
/// );
///
/// watch!(sender,
///     [audio.volume.watch()],
///     |out| {
///         let vol = audio.volume.get();
///         let _ = out.send(ShellCmd::VolumeChanged(vol));
///         if vol == 0.0 {
///             let _ = out.send(ShellCmd::Muted);
///         }
///     }
/// );
/// ```
#[macro_export]
macro_rules! watch {
    ($sender:expr, [$($stream:expr),* $(,)?], $handler:expr => $cmd:expr) => {{
        use ::futures::stream::StreamExt;
        use ::futures::stream::select_all;

        let streams: Vec<$crate::watchers::BoxedStream> = vec![
            $(
                Box::pin(StreamExt::map($stream, |_| ()))
                    as $crate::watchers::BoxedStream,
            )*
        ];

        let handler = $handler;
        let mapper = $cmd;

        $sender.command(move |out, shutdown| async move {
            let mut merged = select_all(streams);
            #[allow(unused_mut)]
            let mut handler = handler;

            ::tokio::select! {
                () = shutdown.wait() => {}
                () = async {
                    while merged.next().await.is_some() {
                        match handler() {
                            Ok(value) => {
                                let _ = out.send(mapper(value));
                            }
                            Err(err) => {
                                ::tracing::error!(error = %err, "Watcher handler failed");
                            }
                        }
                    }
                } => {}
            }
        });
    }};

    ($sender:expr, [$($stream:expr),* $(,)?], |$out:ident| $body:expr) => {{
        use ::futures::stream::StreamExt;
        use ::futures::stream::select_all;

        let streams: Vec<$crate::watchers::BoxedStream> = vec![
            $(
                Box::pin(StreamExt::map($stream, |_| ()))
                    as $crate::watchers::BoxedStream,
            )*
        ];

        $sender.command(move |$out, shutdown| async move {
            let mut merged = select_all(streams);

            ::tokio::select! {
                () = shutdown.wait() => {}
                () = async {
                    while merged.next().await.is_some() {
                        {
                            #[allow(clippy::redundant_closure_call)]
                            (|| { $body })();
                        }
                    }
                } => {}
            }
        });
    }};
}

/// Watches streams with cancellation support via [`CancellationToken`].
///
/// Unlike [`watch!`], this variant stops when either:
/// - The component shuts down (via Relm4's shutdown receiver)
/// - The provided cancellation token is cancelled
///
/// Use this for watchers tied to dynamic resources (e.g., audio devices,
/// media players) that may change during the component's lifetime.
///
/// # Example
///
/// ```ignore
/// struct MyComponent {
///     device_watcher_token: Option<CancellationToken>,
/// }
///
/// // When device changes:
/// if let Some(token) = self.device_watcher_token.take() {
///     token.cancel();
/// }
/// let token = CancellationToken::new();
/// self.device_watcher_token = Some(token.clone());
///
/// watch_cancellable!(sender, token, [device.volume.watch()], |out| {
///     let _ = out.send(Cmd::VolumeChanged);
/// });
/// ```
///
/// [`CancellationToken`]: tokio_util::sync::CancellationToken
#[macro_export]
macro_rules! watch_cancellable {
    ($sender:expr, $token:expr, [$($stream:expr),* $(,)?], |$out:ident| $body:expr) => {{
        use ::futures::stream::StreamExt;
        use ::futures::stream::select_all;

        let streams: Vec<$crate::watchers::BoxedStream> = vec![
            $(
                Box::pin(StreamExt::map($stream, |_| ()))
                    as $crate::watchers::BoxedStream,
            )*
        ];

        let token = $token;
        $sender.command(move |$out, shutdown| async move {
            let mut merged = select_all(streams);

            ::tokio::select! {
                () = shutdown.wait() => {}
                () = token.cancelled() => {}
                () = async {
                    while merged.next().await.is_some() {
                        {
                            #[allow(clippy::redundant_closure_call)]
                            (|| { $body })();
                        }
                    }
                } => {}
            }
        });
    }};
}

/// Throttled variant of [`watch_cancellable!`] with leading-edge behavior.
///
/// Forwards the first event immediately, then enforces a
/// minimum interval between subsequent handler invocations. Events arriving
/// during the cooldown are absorbed — the next handler call after the cooldown
/// reads the latest value from the underlying `tokio::sync::watch` channel.
///
///
/// # Example
///
/// ```ignore
/// use std::time::Duration;
///
/// watch_cancellable_throttled!(
///     sender, token, Duration::from_millis(30),
///     [device.volume.watch(), device.muted.watch()],
///     |out| {
///         let _ = out.send(Cmd::VolumeOrMuteChanged);
///     }
/// );
/// ```
#[macro_export]
macro_rules! watch_cancellable_throttled {
    ($sender:expr, $token:expr, $cooldown:expr, [$($stream:expr),* $(,)?], |$out:ident| $body:expr) => {{
        use ::futures::stream::StreamExt;
        use ::futures::stream::select_all;

        let streams: Vec<$crate::watchers::BoxedStream> = vec![
            $(
                Box::pin(StreamExt::map($stream, |_| ()))
                    as $crate::watchers::BoxedStream,
            )*
        ];

        let token = $token;
        let cooldown = $cooldown;
        $sender.command(move |$out, shutdown| async move {
            let mut merged = select_all(streams);

            ::tokio::select! {
                () = shutdown.wait() => {}
                () = token.cancelled() => {}
                () = async {
                    while merged.next().await.is_some() {
                        {
                            #[allow(clippy::redundant_closure_call)]
                            (|| { $body })();
                        }
                        ::tokio::time::sleep(cooldown).await;
                    }
                } => {}
            }
        });
    }};
}

/// Watches a [`DeferredService`] and sends a command when the service becomes
/// available. Ignores the initial `None` state and any `None` transitions.
///
/// # Example
///
/// ```ignore
/// watch_deferred!(sender, &services.bluetooth, BluetoothCmd::ServiceReady);
/// ```
///
/// [`DeferredService`]: wayle_core::DeferredService
#[macro_export]
macro_rules! watch_deferred {
    ($sender:expr, $property:expr, $cmd:expr) => {{
        let property = $property.clone();

        $crate::watch!($sender, [property.watch()], |out| {
            if let Some(service) = property.get() {
                let _ = out.send($cmd(service));
            }
        });
    }};
}

/// Watches streams with an async handler.
///
/// Like [`watch!`], but the handler is async, allowing `.await` inside the
/// callback. Use this when stream events trigger async work (e.g., IPC calls,
/// network requests).
///
/// # Example
///
/// ```ignore
/// use std::time::Duration;
/// use tokio_stream::wrappers::IntervalStream;
///
/// let interval = IntervalStream::new(tokio::time::interval(Duration::from_secs(2)));
///
/// watch_async!(sender, [interval], |out| async {
///     match service::query().await {
///         Ok(state) => { let _ = out.send(Cmd::StateUpdated(state)); }
///         Err(_) => { let _ = out.send(Cmd::QueryFailed); }
///     }
/// });
/// ```
#[macro_export]
macro_rules! watch_async {
    ($sender:expr, [$($stream:expr),* $(,)?], |$out:ident| async $body:expr) => {{
        use ::futures::stream::StreamExt;
        use ::futures::stream::select_all;

        let streams: Vec<$crate::watchers::BoxedStream> = vec![
            $(
                Box::pin(StreamExt::map($stream, |_| ()))
                    as $crate::watchers::BoxedStream,
            )*
        ];

        $sender.command(move |$out, shutdown| async move {
            let mut merged = select_all(streams);

            ::tokio::select! {
                () = shutdown.wait() => {}
                () = async {
                    while merged.next().await.is_some() {
                        $body
                    }
                } => {}
            }
        });
    }};
}

#[cfg(test)]
mod tests {
    use futures::{StreamExt, stream};

    use super::*;

    #[tokio::test]
    async fn key_changes_emits_the_first_item_and_every_change() {
        let changes = key_changes(stream::iter([1, 1, 2, 2, 3]), None, |item: &i32| *item);

        assert_eq!(changes.count().await, 3);
    }

    #[tokio::test]
    async fn key_changes_compares_the_first_item_with_the_start() {
        let changes = key_changes(stream::iter([1, 2]), Some(1), |item: &i32| *item);

        assert_eq!(changes.count().await, 1);
    }

    #[tokio::test]
    async fn each_changes_merges_the_elements_of_the_latest_list() {
        let lists = stream::iter([vec![2_usize, 3]]).chain(stream::pending());
        let changes = each_changes(lists, |count: &usize| stream::iter(vec![(); *count]));

        assert_eq!(changes.take(5).count().await, 5);
    }
}
