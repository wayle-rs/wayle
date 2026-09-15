pub(crate) mod commands;
pub(crate) mod conversion;
pub(crate) mod dispatcher;
pub(crate) mod events;
pub(crate) mod types;

use std::{
    collections::HashMap,
    future::poll_fn,
    sync::{Arc, RwLock},
};

use commands::Command;
use conversion::volume::to_pulse as convert_volume_to_pulse;
use dispatcher::{handle_external_command, handle_internal_command};
use libpulse_binding::context::{Context, FlagSet as ContextFlags};
use tokio::{
    runtime::Handle,
    sync::mpsc,
    task::{JoinHandle, spawn, spawn_blocking},
};
use tokio_util::sync::CancellationToken;
use tracing::info;
use types::{
    CommandReceiver, DefaultDevice, DeviceStore, EventSender, ExternalCommand, InternalRefresh,
    StreamStore,
};

use crate::{Error, tokio_mainloop::TokioMain};

struct BackendState {
    devices: DeviceStore,
    streams: StreamStore,
    default_input: DefaultDevice,
    default_output: DefaultDevice,
}

impl BackendState {
    fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
            default_input: Arc::new(RwLock::new(None)),
            default_output: Arc::new(RwLock::new(None)),
        }
    }
}

pub(crate) struct PulseBackend {
    state: BackendState,
    mainloop: TokioMain,
    context: Context,
}

impl PulseBackend {
    pub fn start(
        command_rx: CommandReceiver,
        event_tx: EventSender,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<Result<(), Error>> {
        spawn_blocking(move || {
            let runtime = Handle::current();

            runtime.block_on(async move {
                info!("Starting PulseAudio backend");

                let backend = Self::new().await?;
                backend.run(command_rx, event_tx, cancellation_token).await
            })
        })
    }

    async fn new() -> Result<Self, Error> {
        use libpulse_binding::context::State;

        let mut mainloop = TokioMain::new();
        info!("Creating PulseAudio context");
        let mut context =
            Context::new(&mainloop, "wayle-pulse").ok_or(Error::ContextCreationFailed)?;

        info!("Connecting to PulseAudio server");
        context
            .connect(None, ContextFlags::NOFLAGS, None)
            .map_err(Error::ConnectionFailed)?;

        info!("Waiting for PulseAudio context to become ready");
        let state = mainloop
            .wait_for_ready(&context)
            .await
            .map_err(|_| Error::ContextStateFailed(State::Terminated))?;

        if state != State::Ready {
            return Err(Error::ContextStateFailed(state));
        }

        Ok(Self {
            state: BackendState::new(),
            mainloop,
            context,
        })
    }

    fn setup_event_monitoring(
        &mut self,
        event_tx: EventSender,
        cancellation_token: CancellationToken,
    ) -> Result<
        (
            mpsc::UnboundedSender<InternalRefresh>,
            mpsc::UnboundedReceiver<InternalRefresh>,
        ),
        Error,
    > {
        let (internal_command_tx, internal_command_rx) =
            mpsc::unbounded_channel::<InternalRefresh>();

        info!("Setting up PulseAudio event subscription");
        events::start_event_processor(
            &mut self.context,
            self.state.devices.clone(),
            self.state.streams.clone(),
            event_tx,
            internal_command_tx.clone(),
            cancellation_token,
        )?;

        info!("Triggering initial device and stream discovery");
        let _ = internal_command_tx.send(InternalRefresh::Devices);
        let _ = internal_command_tx.send(InternalRefresh::Streams);
        let _ = internal_command_tx.send(InternalRefresh::ServerInfo);

        Ok((internal_command_tx, internal_command_rx))
    }

    fn spawn_command_processor(
        &self,
        mut command_rx: CommandReceiver,
        external_tx: mpsc::UnboundedSender<ExternalCommand>,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<()> {
        let devices = self.state.devices.clone();
        let streams = self.state.streams.clone();

        spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        info!("PulseBackend command handler cancelled");
                        return;
                    }
                    command = command_rx.recv() => {
                        let Some(command) = command else {
                            info!("Command channel closed");
                            return;
                        };

                        Self::handle_command(command, &devices, &streams, &external_tx);
                    }
                }
            }
        })
    }

    #[allow(clippy::too_many_lines)]
    fn handle_command(
        command: Command,
        devices: &DeviceStore,
        streams: &StreamStore,
        external_tx: &mpsc::UnboundedSender<ExternalCommand>,
    ) {
        match command {
            Command::GetDevice {
                device_key,
                responder,
            } => {
                let result = if let Ok(devices_guard) = devices.read() {
                    devices_guard
                        .values()
                        .find(|d| d.key() == device_key)
                        .cloned()
                        .ok_or(Error::DeviceNotFound {
                            index: device_key.index,
                            device_type: device_key.device_type,
                        })
                } else {
                    Err(Error::LockPoisoned)
                };
                let _ = responder.send(result);
            }
            Command::GetStream {
                stream_key,
                responder,
            } => {
                let result = if let Ok(streams_guard) = streams.read() {
                    streams_guard
                        .values()
                        .find(|s| s.key() == stream_key)
                        .cloned()
                        .ok_or(Error::StreamNotFound {
                            index: stream_key.index,
                            stream_type: stream_key.stream_type,
                        })
                } else {
                    Err(Error::LockPoisoned)
                };
                let _ = responder.send(result);
            }
            Command::SetVolume {
                device_key,
                volume,
                responder,
            } => {
                let pulse_volume = convert_volume_to_pulse(&volume);
                let _ = external_tx.send(ExternalCommand::SetDeviceVolume {
                    device_key,
                    volume: pulse_volume,
                });
                let _ = responder.send(Ok(()));
            }
            Command::SetMute {
                device_key,
                muted,
                responder,
            } => {
                let _ = external_tx.send(ExternalCommand::SetDeviceMute { device_key, muted });
                let _ = responder.send(Ok(()));
            }
            Command::SetStreamVolume {
                stream_key,
                volume,
                responder,
            } => {
                let pulse_volume = convert_volume_to_pulse(&volume);
                let _ = external_tx.send(ExternalCommand::SetStreamVolume {
                    stream_key,
                    volume: pulse_volume,
                });
                let _ = responder.send(Ok(()));
            }
            Command::SetStreamMute {
                stream_key,
                muted,
                responder,
            } => {
                let _ = external_tx.send(ExternalCommand::SetStreamMute { stream_key, muted });
                let _ = responder.send(Ok(()));
            }
            Command::SetDefaultInput {
                device_key,
                responder,
            } => {
                let _ = external_tx.send(ExternalCommand::SetDefaultInput { device_key });
                let _ = responder.send(Ok(()));
            }
            Command::SetDefaultOutput {
                device_key,
                responder,
            } => {
                let _ = external_tx.send(ExternalCommand::SetDefaultOutput { device_key });
                let _ = responder.send(Ok(()));
            }
            Command::MoveStream {
                stream_key,
                device_key,
                responder,
            } => {
                let _ = external_tx.send(ExternalCommand::MoveStream {
                    stream_key,
                    device_key,
                });
                let _ = responder.send(Ok(()));
            }
            Command::SetPort {
                device_key,
                port,
                responder,
            } => {
                let _ = external_tx.send(ExternalCommand::SetPort { device_key, port });
                let _ = responder.send(Ok(()));
            }
        }
    }

    #[allow(clippy::cognitive_complexity)]
    async fn run(
        mut self,
        command_rx: CommandReceiver,
        event_tx: EventSender,
        cancellation_token: CancellationToken,
    ) -> Result<(), Error> {
        let event_token = cancellation_token.child_token();
        let command_token = cancellation_token.child_token();

        let (_, mut internal_rx) = self.setup_event_monitoring(event_tx.clone(), event_token)?;

        let (external_tx, mut external_rx) = mpsc::unbounded_channel::<ExternalCommand>();

        info!("PulseAudio backend fully initialized and monitoring");

        let command_handle =
            self.spawn_command_processor(command_rx, external_tx, command_token.clone());

        loop {
            tokio::select! {
                biased;

                _ = cancellation_token.cancelled() => {
                    info!("PulseAudio backend cancelled");
                    break;
                }

                result = poll_fn(|cx| self.mainloop.tick(cx)) => {
                    if result.is_some() {
                        info!("PulseAudio mainloop quit requested");
                        break;
                    }
                }

                Some(cmd) = internal_rx.recv() => {
                    handle_internal_command(
                        &mut self.context,
                        cmd,
                        &self.state.devices,
                        &self.state.streams,
                        &event_tx,
                        &self.state.default_input,
                        &self.state.default_output,
                    );
                }

                Some(cmd) = external_rx.recv() => {
                    handle_external_command(
                        &mut self.context,
                        cmd,
                        &self.state.devices,
                        &self.state.streams,
                    );
                }
            }
        }

        self.context.disconnect();
        command_token.cancel();
        let _ = command_handle.await;

        info!("PulseAudio backend stopped");
        Ok(())
    }
}
