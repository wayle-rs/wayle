use libpulse_binding::context::{
    Context,
    subscribe::{Facility, InterestMaskSet, Operation},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::info;

use super::types::{
    ChangeNotification, DeviceStore, EventSender, InternalCommandSender, StreamStore,
};
use crate::error::Error;

pub(crate) mod device;
pub(crate) mod server;
pub(crate) mod stream;

type SubscriptionCallback = Option<Box<dyn FnMut(Option<Facility>, Option<Operation>, u32)>>;

pub(crate) fn start_event_processor(
    context: &mut Context,
    devices: DeviceStore,
    streams: StreamStore,
    events_tx: EventSender,
    internal_command_tx: InternalCommandSender,
    cancellation_token: CancellationToken,
) -> Result<(), Error> {
    let (change_tx, mut change_rx) = mpsc::unbounded_channel::<ChangeNotification>();

    setup_subscription(context, change_tx)?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    info!("Event processor cancelled, stopping");
                    return;
                }
                Some(notification) = change_rx.recv() => {
                    process_change_notification(
                        notification,
                        &devices,
                        &streams,
                        &events_tx,
                        &internal_command_tx
                    )
                    .await;
                }
                else => {
                    info!("Change notification channel closed");
                    return;
                }
            }
        }
    });

    Ok(())
}

fn setup_subscription(
    context: &mut Context,
    change_tx: mpsc::UnboundedSender<ChangeNotification>,
) -> Result<(), Error> {
    let interest_mask = InterestMaskSet::SINK
        | InterestMaskSet::SOURCE
        | InterestMaskSet::SINK_INPUT
        | InterestMaskSet::SOURCE_OUTPUT
        | InterestMaskSet::SERVER;

    let subscription_callback: SubscriptionCallback =
        Some(Box::new(move |facility, operation, index| {
            let (Some(facility), Some(operation)) = (facility, operation) else {
                return;
            };

            let notification = match facility {
                Facility::Sink | Facility::Source => ChangeNotification::Device {
                    facility,
                    operation,
                    index,
                },
                Facility::SinkInput | Facility::SourceOutput => ChangeNotification::Stream {
                    facility,
                    operation,
                    index,
                },
                Facility::Server => ChangeNotification::Server { operation },
                _ => return,
            };

            let _ = change_tx.send(notification);
        }));

    context.set_subscribe_callback(subscription_callback);

    context.subscribe(interest_mask, |_success: bool| {});

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_change_notification(
    notification: ChangeNotification,
    devices: &DeviceStore,
    streams: &StreamStore,
    events_tx: &EventSender,
    command_tx: &InternalCommandSender,
) {
    match notification {
        ChangeNotification::Device {
            facility,
            operation,
            index,
        } => {
            device::handle_change(facility, operation, index, devices, events_tx, command_tx).await;
        }
        ChangeNotification::Stream {
            facility,
            operation,
            index,
        } => {
            stream::handle_change(facility, operation, index, streams, events_tx, command_tx).await;
        }
        ChangeNotification::Server { operation } => {
            server::handle_change(operation, command_tx).await;
        }
    }
}
