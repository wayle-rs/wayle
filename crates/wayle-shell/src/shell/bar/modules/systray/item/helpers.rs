use std::{collections::hash_map::DefaultHasher, hash::Hasher, path::Path};

use gtk4::{gdk, glib, prelude::Cast};
use wayle_systray::types::item::IconPixmap;

const TARGET_ICON_SIZE: i32 = 24;
const ICON_EXTENSIONS: [&str; 3] = ["png", "svg", "xpm"];

pub(super) fn select_best_pixmap(pixmaps: &[IconPixmap]) -> Option<&IconPixmap> {
    pixmaps
        .iter()
        .min_by_key(|p| (p.width - TARGET_ICON_SIZE).abs() + (p.height - TARGET_ICON_SIZE).abs())
}

pub(super) fn create_texture_from_pixmap(pixmap: &IconPixmap) -> Option<gdk::Texture> {
    let rgba_data = argb_to_rgba(&pixmap.data);
    let bytes = glib::Bytes::from_owned(rgba_data);

    gdk::MemoryTexture::new(
        pixmap.width,
        pixmap.height,
        gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (pixmap.width * 4) as usize,
    )
    .upcast::<gdk::Texture>()
    .into()
}

pub(super) fn hash_pixmaps(pixmaps: &[IconPixmap]) -> u64 {
    let mut hasher = DefaultHasher::new();

    for pixmap in pixmaps {
        hasher.write_i32(pixmap.width);
        hasher.write_i32(pixmap.height);
        hasher.write_usize(pixmap.data.len());
        hasher.write(&pixmap.data);
    }

    hasher.finish()
}

pub(super) fn load_scaled_texture_from_file(path: &str) -> Option<gdk::Texture> {
    let pixbuf =
        gdk_pixbuf::Pixbuf::from_file_at_scale(path, TARGET_ICON_SIZE, TARGET_ICON_SIZE, true)
            .ok()?;
    Some(gdk::Texture::for_pixbuf(&pixbuf))
}

pub(super) fn find_icon_in_theme_path(theme_path: &str, icon_name: &str) -> Option<String> {
    if theme_path.is_empty() {
        return None;
    }

    for ext in ICON_EXTENSIONS {
        let file_path = format!("{theme_path}/{icon_name}.{ext}");
        if Path::new(&file_path).is_file() {
            return Some(file_path);
        }
    }

    None
}

pub(super) fn load_icon_from_theme_path(theme_path: &str, icon_name: &str) -> Option<gdk::Texture> {
    find_icon_in_theme_path(theme_path, icon_name)
        .as_deref()
        .and_then(load_scaled_texture_from_file)
}

fn argb_to_rgba(argb: &[u8]) -> Vec<u8> {
    argb.chunks_exact(4)
        .flat_map(|chunk| {
            let a = chunk[0];
            let r = chunk[1];
            let g = chunk[2];
            let b = chunk[3];
            [r, g, b, a]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argb_to_rgba_single_pixel() {
        let argb = [0xFF, 0x11, 0x22, 0x33];
        let rgba = argb_to_rgba(&argb);
        assert_eq!(rgba, vec![0x11, 0x22, 0x33, 0xFF]);
    }

    #[test]
    fn argb_to_rgba_multiple_pixels() {
        let argb = [0xFF, 0x11, 0x22, 0x33, 0x80, 0xAA, 0xBB, 0xCC];
        let rgba = argb_to_rgba(&argb);
        assert_eq!(rgba, vec![0x11, 0x22, 0x33, 0xFF, 0xAA, 0xBB, 0xCC, 0x80,]);
    }

    #[test]
    fn argb_to_rgba_empty() {
        let argb: [u8; 0] = [];
        let rgba = argb_to_rgba(&argb);
        assert!(rgba.is_empty());
    }

    #[test]
    fn argb_to_rgba_ignores_trailing_bytes() {
        let argb = [0xFF, 0x11, 0x22, 0x33, 0xAA, 0xBB];
        let rgba = argb_to_rgba(&argb);
        assert_eq!(rgba, vec![0x11, 0x22, 0x33, 0xFF]);
    }

    #[test]
    fn select_best_pixmap_exact_match() {
        let pixmaps = vec![
            IconPixmap {
                width: 16,
                height: 16,
                data: vec![],
            },
            IconPixmap {
                width: 24,
                height: 24,
                data: vec![],
            },
            IconPixmap {
                width: 48,
                height: 48,
                data: vec![],
            },
        ];
        let best = select_best_pixmap(&pixmaps);
        assert!(best.is_some());
        let best = best.unwrap();
        assert_eq!(best.width, 24);
        assert_eq!(best.height, 24);
    }

    #[test]
    fn select_best_pixmap_prefers_larger_when_available() {
        let pixmaps = vec![
            IconPixmap {
                width: 16,
                height: 16,
                data: vec![],
            },
            IconPixmap {
                width: 28,
                height: 28,
                data: vec![],
            },
            IconPixmap {
                width: 64,
                height: 64,
                data: vec![],
            },
        ];
        let best = select_best_pixmap(&pixmaps);
        assert!(best.is_some());
        let best = best.unwrap();
        assert_eq!(best.width, 28);
    }

    #[test]
    fn select_best_pixmap_closest_smaller() {
        let pixmaps = vec![
            IconPixmap {
                width: 8,
                height: 8,
                data: vec![],
            },
            IconPixmap {
                width: 20,
                height: 20,
                data: vec![],
            },
        ];
        let best = select_best_pixmap(&pixmaps);
        assert!(best.is_some());
        let best = best.unwrap();
        assert_eq!(best.width, 20);
    }

    #[test]
    fn select_best_pixmap_empty() {
        let pixmaps: Vec<IconPixmap> = vec![];
        let best = select_best_pixmap(&pixmaps);
        assert!(best.is_none());
    }

    #[test]
    fn select_best_pixmap_single() {
        let pixmaps = vec![IconPixmap {
            width: 128,
            height: 128,
            data: vec![],
        }];
        let best = select_best_pixmap(&pixmaps);
        assert!(best.is_some());
        assert_eq!(best.unwrap().width, 128);
    }

    #[test]
    fn select_best_pixmap_non_square() {
        let pixmaps = vec![
            IconPixmap {
                width: 32,
                height: 16,
                data: vec![],
            },
            IconPixmap {
                width: 24,
                height: 24,
                data: vec![],
            },
            IconPixmap {
                width: 16,
                height: 32,
                data: vec![],
            },
        ];
        let best = select_best_pixmap(&pixmaps);
        assert!(best.is_some());
        let best = best.unwrap();
        assert_eq!(best.width, 24);
        assert_eq!(best.height, 24);
    }
}
