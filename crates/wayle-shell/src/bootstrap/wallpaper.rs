use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use tracing::{debug, warn};
use wayle_config::schemas::wallpaper::MonitorWallpaperConfig;
use wayle_output::OutputService;
use wayle_wallpaper::{
    ColorExtractorConfig, MonitorState, TransitionConfig, TransitionDuration, TransitionFps,
    WallpaperService,
};

use crate::wallpaper_map;

pub(super) async fn build_wallpaper_service(
    cfg: &wayle_config::schemas::wallpaper::WallpaperConfig,
    theming_monitor: Option<String>,
    color_extractor: ColorExtractorConfig,
    outputs: Option<Arc<OutputService>>,
) -> Result<Arc<WallpaperService>, wayle_wallpaper::Error> {
    let transition = TransitionConfig {
        transition_type: wallpaper_map::transition_type(cfg.transition_type.get()),
        duration: TransitionDuration(cfg.transition_duration.get().value()),
        fps: TransitionFps(cfg.transition_fps.get().value()),
        step: None,
    };

    let t = Instant::now();
    let mut builder = WallpaperService::builder();
    if let Some(outputs) = outputs {
        builder = builder.outputs(outputs);
    }

    let service = builder
        .transition(transition)
        .theming_monitor(theming_monitor)
        .color_extractor(color_extractor)
        .shared_cycle(cfg.cycling_same_image.get())
        .engine_active(cfg.engine_enabled.get())
        .build()
        .await?;
    debug!(elapsed_ms = t.elapsed().as_millis(), "Service built");

    let has_monitor_wallpapers = apply_monitor_config(&service, cfg);
    debug!(
        elapsed_ms = t.elapsed().as_millis(),
        "Monitor config applied"
    );

    let cycling_started = start_cycling_from_config(&service, cfg);

    if has_monitor_wallpapers && !cycling_started {
        service.render_all_background();
    }

    Ok(service)
}

fn start_cycling_from_config(
    service: &Arc<WallpaperService>,
    cfg: &wayle_config::schemas::wallpaper::WallpaperConfig,
) -> bool {
    if !cfg.cycling_enabled.get() {
        return false;
    }

    let directory = cfg.cycling_directory.get();
    if directory.is_empty() {
        return false;
    }

    let mode = wallpaper_map::cycling_mode(cfg.cycling_mode.get());
    let interval = Duration::from_secs(cfg.cycling_interval_mins.get().value() * 60);

    if let Err(e) = service.start_cycling(PathBuf::from(directory), interval, mode) {
        warn!(error = %e, "could not start wallpaper cycling from config");
        return false;
    }

    true
}

fn apply_monitor_config(
    service: &Arc<WallpaperService>,
    cfg: &wayle_config::schemas::wallpaper::WallpaperConfig,
) -> bool {
    let monitor_configs = cfg.monitors.get();
    if monitor_configs.is_empty() {
        return false;
    }

    let mut monitors = service.monitors.get();
    let mut has_wallpapers = false;

    for (connector, state) in &mut monitors {
        if let Some(monitor_cfg) = wallpaper_map::config_for(service, &monitor_configs, connector) {
            has_wallpapers |= apply_single_monitor(state, connector, monitor_cfg);
        }
    }

    service.monitors.set(monitors);

    has_wallpapers
}

fn apply_single_monitor(
    state: &mut MonitorState,
    connector: &str,
    monitor_cfg: &MonitorWallpaperConfig,
) -> bool {
    state.fit_mode = wallpaper_map::fit_mode(monitor_cfg.fit_mode);

    if monitor_cfg.wallpaper.is_empty() {
        return false;
    }

    let path = PathBuf::from(&monitor_cfg.wallpaper);
    if !path.exists() {
        warn!(
            monitor = %connector,
            path = %monitor_cfg.wallpaper,
            "wallpaper path not found"
        );
        return false;
    }

    state.wallpaper = Some(path);
    true
}
