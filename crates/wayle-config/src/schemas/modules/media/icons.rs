//! Player icon mappings for the media module.

/// Built-in player-to-icon mappings.
///
/// Patterns use glob syntax and match against MPRIS bus names
/// (e.g., `org.mpris.MediaPlayer2.spotify`). Order matters - first match wins.
///
/// Only includes native Linux applications with MPRIS support.
pub const BUILTIN_MAPPINGS: &[(&str, &str)] = &[
    ("*spotify*", "si-spotify-symbolic"),
    ("*tidal*", "si-tidal-symbolic"),
    ("*vlc*", "si-vlcmediaplayer-symbolic"),
    ("*mpv*", "si-mpv-symbolic"),
    ("*kodi*", "si-kodi-symbolic"),
    ("*celluloid*", "si-mpv-symbolic"),
    ("*jellyfin*", "si-jellyfin-symbolic"),
    ("*firefox*", "si-firefox-symbolic"),
    ("*librewolf*", "si-librewolf-symbolic"),
    ("*floorp*", "si-floorp-symbolic"),
    ("*zen*", "si-zenbrowser-symbolic"),
    ("*chrom*", "si-googlechrome-symbolic"),
    ("*brave*", "si-brave-symbolic"),
    ("*vivaldi*", "si-vivaldi-symbolic"),
    ("*opera*", "si-opera-symbolic"),
    ("*edge*", "tb-brand-edge-symbolic"),
    ("*tor*", "si-torbrowser-symbolic"),
    ("*helium*", "si-heliumbrowser-symbolic"),
];
