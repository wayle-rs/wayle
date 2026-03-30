use std::cell::RefCell;
use std::rc::Rc;

use gdk4::{MemoryFormat, MemoryTexture};
use relm4::gtk;
use gtk::prelude::*;
use wayle_capture::CaptureResult;

/// Hit region for click-to-focus on the composite thumbnail.
pub(super) struct ClickRegion {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub address: String,
}

/// Composite all thumbnails into a single image buffer and display on a Picture.
///
/// All positioning uses logical coordinates (monitor-relative). The captured
/// buffer dimensions may differ from logical sizes due to compositor scaling,
/// so we derive per-window buffer-to-logical ratios rather than assuming a
/// single global scale factor.
pub(super) fn apply_capture_result(
    preview: &gtk::Picture,
    result: &CaptureResult,
    click_regions: &Rc<RefCell<Vec<ClickRegion>>>,
    preview_width: f64,
) {
    let mon_w = result.monitor_width as f64;
    let mon_h = result.monitor_height as f64;
    if mon_w <= 0.0 || mon_h <= 0.0 {
        return;
    }

    // Scale factor from logical monitor space → preview pixel space.
    let scale = preview_width / mon_w;
    let pw = preview_width as u32;
    let ph = ((mon_h * scale) as u32).max(1);
    let stride = pw as usize * 4;
    let mut buf = vec![0u8; stride * ph as usize];

    let mut regions = click_regions.borrow_mut();
    regions.clear();

    for thumb in &result.thumbnails {
        // Destination size in preview pixels, based on logical window size.
        let dst_w = ((thumb.win_width as f64 * scale) as u32).max(1);
        let dst_h = ((thumb.win_height as f64 * scale) as u32).max(1);

        // The captured buffer may be larger than the logical size (HiDPI).
        // downscale_nearest maps from captured buffer pixels → preview pixels,
        // which handles any buffer-to-logical ratio correctly.
        let src_w = thumb.width.max(1);
        let src_h = thumb.height.max(1);
        let (scaled, scaled_stride) = downscale_nearest(
            &thumb.data,
            src_w,
            src_h,
            thumb.stride,
            dst_w,
            dst_h,
        );

        // Position in preview pixels, from logical monitor-relative coords.
        let ox = (thumb.x as f64 * scale) as i32;
        let oy = (thumb.y as f64 * scale) as i32;

        // Blit into composite buffer.
        for row in 0..dst_h as i32 {
            let by = oy + row;
            if by < 0 || by >= ph as i32 {
                continue;
            }
            for col in 0..dst_w as i32 {
                let bx = ox + col;
                if bx < 0 || bx >= pw as i32 {
                    continue;
                }
                let src_off = row as usize * scaled_stride + col as usize * 4;
                let dst_off = by as usize * stride + bx as usize * 4;
                if src_off + 4 <= scaled.len() && dst_off + 4 <= buf.len() {
                    buf[dst_off..dst_off + 3].copy_from_slice(&scaled[src_off..src_off + 3]);
                    buf[dst_off + 3] = 0xFF; // force opaque
                }
            }
        }

        regions.push(ClickRegion {
            x: ox.max(0) as f64,
            y: oy.max(0) as f64,
            w: dst_w as f64,
            h: dst_h as f64,
            address: thumb.address.clone(),
        });
    }
    drop(regions);

    let bytes = glib::Bytes::from(&buf);
    let texture = MemoryTexture::new(
        pw as i32,
        ph as i32,
        MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        stride,
    );
    preview.set_paintable(Some(&texture));
    preview.set_size_request(pw as i32, ph as i32);
    preview.set_visible(true);
}

/// Nearest-neighbor downscale of BGRA pixel data.
fn downscale_nearest(
    src: &[u8],
    src_w: u32,
    src_h: u32,
    src_stride: u32,
    dst_w: u32,
    dst_h: u32,
) -> (Vec<u8>, usize) {
    let dst_stride = dst_w as usize * 4;
    let mut dst = vec![0u8; dst_stride * dst_h as usize];

    for dy in 0..dst_h {
        let sy = ((dy as u64 * src_h as u64) / dst_h as u64) as u32;
        for dx in 0..dst_w {
            let sx = ((dx as u64 * src_w as u64) / dst_w as u64) as u32;
            let src_off = (sy * src_stride + sx * 4) as usize;
            let dst_off = dy as usize * dst_stride + dx as usize * 4;
            if src_off + 4 <= src.len() {
                dst[dst_off..dst_off + 4].copy_from_slice(&src[src_off..src_off + 4]);
            }
        }
    }

    (dst, dst_stride)
}
