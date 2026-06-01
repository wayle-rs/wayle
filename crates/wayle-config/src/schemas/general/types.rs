use wayle_derive::wayle_enum;

/// Layer-shell layer a window is placed on, from furthest back to furthest front.
#[wayle_enum(default)]
pub enum Layer {
    /// Below everything else, used for wallpapers and ambient surfaces.
    Background,
    /// Behind regular application windows.
    Bottom,
    /// Above regular application windows.
    #[default]
    Top,
    /// Above everything, including fullscreen application windows.
    Overlay,
}
