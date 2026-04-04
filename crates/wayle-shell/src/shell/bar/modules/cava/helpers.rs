use std::sync::Arc;

use wayle_cava::{CavaService, InputMethod};
use wayle_config::{ConfigService, schemas::modules::CavaInput};

pub(super) use crate::shell::bar::modules::shared::rem_to_px;

pub(super) fn map_input(input: CavaInput) -> InputMethod {
    match input {
        CavaInput::PipeWire => InputMethod::PipeWire,
        CavaInput::Pulse => InputMethod::Pulse,
        CavaInput::Alsa => InputMethod::Alsa,
        CavaInput::Jack => InputMethod::Jack,
        CavaInput::Fifo => InputMethod::Fifo,
        CavaInput::PortAudio => InputMethod::PortAudio,
        CavaInput::Sndio => InputMethod::Sndio,
        CavaInput::Oss => InputMethod::Oss,
        CavaInput::Shmem => InputMethod::Shmem,
        CavaInput::Winscap => InputMethod::Winscap,
    }
}

pub(super) async fn build_cava_service(
    config: &Arc<ConfigService>,
) -> Result<Arc<CavaService>, wayle_cava::Error> {
    let cfg = &config.config().modules.cava;

    let service = CavaService::builder()
        .bars(cfg.bars.get().value())
        .framerate(cfg.framerate.get().value())
        .autosens(true)
        .stereo(cfg.stereo.get())
        .noise_reduction(cfg.noise_reduction.get().value())
        .monstercat(cfg.monstercat.get())
        .waves(cfg.waves.get())
        .low_cutoff(cfg.low_cutoff.get().value())
        .high_cutoff(cfg.high_cutoff.get().value())
        .input(map_input(cfg.input.get()))
        .source(cfg.source.get().clone())
        .build()
        .await?;

    Ok(Arc::new(service))
}
