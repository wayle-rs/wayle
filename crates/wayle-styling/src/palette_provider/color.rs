use palette::{FromColor, Hsl, IntoColor, Srgb};

pub(crate) struct Layers {
    pub(crate) bg: String,
    pub(crate) surface: String,
    pub(crate) elevated: String,
}

const OFFSET: f32 = 0.04;

pub(crate) fn derive_layers(background: &str, is_light: bool) -> Layers {
    if is_light {
        Layers {
            bg: lighten(background, OFFSET),
            surface: background.to_owned(),
            elevated: lighten(background, -OFFSET),
        }
    } else {
        Layers {
            bg: lighten(background, -OFFSET),
            surface: background.to_owned(),
            elevated: lighten(background, OFFSET),
        }
    }
}

pub(crate) fn hex_to_rgba(hex: &str, alpha: f32) -> String {
    let srgb = parse(hex);
    let srgb = srgb.into_format::<u8>();
    format!(
        "rgba({}, {}, {}, {:.2})",
        srgb.red, srgb.green, srgb.blue, alpha
    )
}

fn parse(hex: &str) -> Srgb<f32> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Srgb::new(r, g, b).into_format()
}

fn format(color: Srgb<f32>) -> String {
    let rgb: Srgb<u8> = color.into_format();
    format!("#{:02x}{:02x}{:02x}", rgb.red, rgb.green, rgb.blue)
}

fn lighten(hex: &str, amount: f32) -> String {
    let rgb = parse(hex);
    let mut hsl = Hsl::from_color(rgb);
    hsl.lightness = (hsl.lightness + amount).clamp(0.0, 1.0);
    let rgb: Srgb<f32> = hsl.into_color();
    format(rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lighten_black() {
        let result = lighten("#000000", 0.1);
        assert_ne!(result, "#000000");
    }

    #[test]
    fn lighten_clamps_to_white() {
        let result = lighten("#ffffff", 0.5);
        assert_eq!(result, "#ffffff");
    }

    #[test]
    fn roundtrip_preserves_color() {
        let original = "#b4befe";
        let rgb = parse(original);
        let hsl = Hsl::from_color(rgb);
        let back: Srgb<f32> = hsl.into_color();
        let back_u8: Srgb<u8> = back.into_format();
        let orig_u8: Srgb<u8> = rgb.into_format();
        assert!((orig_u8.red as i16 - back_u8.red as i16).unsigned_abs() <= 1);
        assert!((orig_u8.green as i16 - back_u8.green as i16).unsigned_abs() <= 1);
        assert!((orig_u8.blue as i16 - back_u8.blue as i16).unsigned_abs() <= 1);
    }

    #[test]
    fn surface_ramp_from_dark_bg() {
        let bg = "#11111b";
        let surface = lighten(bg, 0.03);
        let elevated = lighten(bg, 0.06);

        let l_bg = Hsl::from_color(parse(bg)).lightness;
        let l_surface = Hsl::from_color(parse(&surface)).lightness;
        let l_elevated = Hsl::from_color(parse(&elevated)).lightness;

        assert!(l_surface > l_bg);
        assert!(l_elevated > l_surface);
    }

    #[test]
    fn derive_layers_dark_bg_darkens_base() {
        let layers = derive_layers("#11111b", false);

        let l_bg = Hsl::from_color(parse(&layers.bg)).lightness;
        let l_surface = Hsl::from_color(parse(&layers.surface)).lightness;
        let l_elevated = Hsl::from_color(parse(&layers.elevated)).lightness;

        assert!(l_bg < l_surface);
        assert!(l_surface < l_elevated);
    }

    #[test]
    fn derive_layers_light_bg_lightens_base() {
        let layers = derive_layers("#e0e0e0", true);

        let l_bg = Hsl::from_color(parse(&layers.bg)).lightness;
        let l_surface = Hsl::from_color(parse(&layers.surface)).lightness;
        let l_elevated = Hsl::from_color(parse(&layers.elevated)).lightness;

        assert!(l_bg > l_surface);
        assert!(l_surface > l_elevated);
    }
}
