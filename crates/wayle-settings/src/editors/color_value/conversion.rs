//! Hex-string <-> GdkRGBA conversion.

use relm4::gtk::gdk;

pub(super) fn hex_to_rgba(hex: &str) -> gdk::RGBA {
    gdk::RGBA::parse(hex).unwrap_or(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))
}

pub(super) fn rgba_to_hex(rgba: &gdk::RGBA) -> String {
    let red = (rgba.red() * 255.0).round() as u8;
    let green = (rgba.green() * 255.0).round() as u8;
    let blue = (rgba.blue() * 255.0).round() as u8;
    let alpha = (rgba.alpha() * 255.0).round() as u8;

    if alpha == 255 {
        return format!("#{red:02x}{green:02x}{blue:02x}");
    }

    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}
