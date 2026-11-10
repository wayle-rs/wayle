//! Build script that concatenates Fluent i18n partial files (`_*.ftl`) into
//! a single `wayle-shell.ftl` per locale directory, bundles the country-flag
//! SVGs into a GResource, and enforces link order for gtk4-layer-shell.

#![allow(clippy::expect_used, clippy::panic)]

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=assets/flags");

    // wayle-idle-inhibit can pull in libwayland-client early due to linker behavior.
    // Which then prevents the gtk4 layer shell from interposing since it's gotta be
    // first in the link/load order used for symbol resolution.
    // Easier to just enforce the linking order in our shell, so here we are...
    println!("cargo:rustc-link-lib=gtk4-layer-shell");
    println!("cargo:rustc-link-lib=wayland-client");

    let locales_dir = Path::new("locales");

    for entry in fs::read_dir(locales_dir).expect("locales/ directory must exist") {
        let locale_dir = entry.expect("readable directory entry").path();
        if locale_dir.is_dir() {
            concatenate_partials(&locale_dir);
        }
    }

    build_flag_icons();
}

/// Bakes rounded corners into each vendored flag SVG and compiles them into a
/// `flags.gresource` in `OUT_DIR`, laid out as an icon-theme resource path
/// (`scalable/actions/flag-<code>.svg`). The shell registers this resource and
/// adds `/dev/wayle/icons` to GTK's `IconTheme`, so `flag-<code>` names resolve
/// to full-color, rounded country flags (bundled in the binary — no CDN fetch,
/// and outside wayle-icons' monochrome symbolic pipeline).
fn build_flag_icons() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR set by cargo"));
    let src_dir = Path::new("assets/flags");
    let staging = out_dir.join("flags/scalable/actions");
    fs::create_dir_all(&staging).expect("create flag staging dir");

    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(src_dir).expect("assets/flags/ must exist") {
        let path = entry.expect("readable dir entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
            continue;
        }
        let Some(code) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let svg = fs::read_to_string(&path).expect("read flag svg");
        let rounded = squareify_and_round(&svg)
            .unwrap_or_else(|| panic!("unexpected SVG structure in {}", path.display()));
        let out_name = format!("flag-{code}.svg");
        fs::write(staging.join(&out_name), rounded).expect("write rounded flag svg");
        names.push(out_name);
    }
    names.sort();

    let mut manifest = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <gresources>\n  <gresource prefix=\"/dev/wayle/icons\">\n",
    );
    for name in &names {
        let _ = writeln!(manifest, "    <file>scalable/actions/{name}</file>");
    }
    manifest.push_str("  </gresource>\n</gresources>\n");
    let manifest_path = out_dir.join("flags.gresource.xml");
    fs::write(&manifest_path, manifest).expect("write gresource manifest");

    let status = Command::new("glib-compile-resources")
        .arg(format!("--sourcedir={}", out_dir.join("flags").display()))
        .arg(format!(
            "--target={}",
            out_dir.join("flags.gresource").display()
        ))
        .arg(&manifest_path)
        .status()
        .expect("glib-compile-resources must be on PATH (provided by glib)");
    assert!(status.success(), "glib-compile-resources failed");
}

/// Rewrites a (landscape 4:3) flag SVG so it renders undistorted in GTK's
/// **square** icon slots: the original content is centered on a square canvas
/// (by widening the `viewBox` vertically) and clipped to a rounded rect (~15% of
/// the short side). GTK renders the square canvas 1:1 into the square slot, so
/// the flag keeps its true proportions (letterboxed) rather than being stretched
/// — which is what a bare non-square SVG icon would suffer, since
/// `GtkIconPaintable` reports a square intrinsic size. Returns `None` if the
/// input lacks a recognizable `<svg …>…</svg>` structure.
fn squareify_and_round(svg: &str) -> Option<String> {
    let svg_tag = svg.find("<svg")?;
    // The `>` that closes the opening `<svg …>` tag (SVGO output has no `>`
    // inside attribute values, so the first one closes the tag).
    let open_end = svg[svg_tag..].find('>')? + svg_tag;
    let close = svg.rfind("</svg>")?;
    let open_tag = &svg[svg_tag..=open_end];

    let (minx, miny, width, height) = parse_viewbox(open_tag)?;
    let side = width.max(height);
    let canvas_x = minx - (side - width) / 2.0;
    let canvas_y = miny - (side - height) / 2.0;
    let radius = (width.min(height) * 0.15).round() as i64;

    let new_viewbox = format!(
        "viewBox=\"{} {} {} {}\"",
        num(canvas_x),
        num(canvas_y),
        num(side),
        num(side)
    );
    let open_tag = replace_attr(open_tag, "viewBox=\"", &new_viewbox)?;

    let defs = format!(
        "<defs><clipPath id=\"wayle-round\">\
         <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{radius}\" ry=\"{radius}\"/>\
         </clipPath></defs><g clip-path=\"url(#wayle-round)\">",
        num(minx),
        num(miny),
        num(width),
        num(height)
    );

    let mut out = String::with_capacity(svg.len() + open_tag.len() + defs.len() + 8);
    out.push_str(&svg[..svg_tag]); // anything before `<svg` (e.g. an XML decl)
    out.push_str(&open_tag); // "<svg … viewBox=<square>>"
    out.push_str(&defs);
    out.push_str(&svg[open_end + 1..close]); // original inner content
    out.push_str("</g>");
    out.push_str(&svg[close..]); // "</svg>…"
    Some(out)
}

/// Formats a coordinate compactly — as an integer when whole (the flag viewBoxes
/// are), else to 3 decimals.
fn num(value: f64) -> String {
    if value.fract().abs() < 1e-6 {
        format!("{}", value as i64)
    } else {
        format!("{value:.3}")
    }
}

/// Replaces a `name="value"` attribute (matched by the `name="` prefix) in an
/// opening tag with `replacement`.
fn replace_attr(open_tag: &str, prefix: &str, replacement: &str) -> Option<String> {
    let key = open_tag.find(prefix)?;
    let val_start = key + prefix.len();
    let val_end = val_start + open_tag[val_start..].find('"')?;
    let mut out = String::with_capacity(open_tag.len() + replacement.len());
    out.push_str(&open_tag[..key]);
    out.push_str(replacement);
    out.push_str(&open_tag[val_end + 1..]); // past the closing quote
    Some(out)
}

/// Extracts `(minx, miny, width, height)` from a `viewBox="…"` attribute in an
/// opening `<svg>` tag.
fn parse_viewbox(svg_open_tag: &str) -> Option<(f64, f64, f64, f64)> {
    let start = svg_open_tag.find("viewBox=\"")? + "viewBox=\"".len();
    let end = start + svg_open_tag[start..].find('"')?;
    let nums: Vec<f64> = svg_open_tag[start..end]
        .split_whitespace()
        .filter_map(|token| token.parse().ok())
        .collect();
    match nums.as_slice() {
        [minx, miny, width, height] => Some((*minx, *miny, *width, *height)),
        _ => None,
    }
}

fn concatenate_partials(locale_dir: &Path) {
    let partials = collect_partials_recursive(locale_dir);
    let combined = merge_partials(&partials);
    let output = locale_dir.join("wayle-shell.ftl");
    let existing = fs::read_to_string(&output).unwrap_or_default();
    if existing != combined {
        fs::write(&output, combined).expect("failed to write combined ftl");
    }
}

fn collect_partials_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut partials = Vec::new();
    collect_partials_inner(dir, &mut partials);
    partials.sort();
    partials
}

fn collect_partials_inner(dir: &Path, partials: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_dir() {
            collect_partials_inner(&path, partials);
        } else if is_partial(&path) {
            partials.push(path);
        }
    }
}

fn is_partial(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.starts_with('_') && name.ends_with(".ftl"))
}

fn merge_partials(partials: &[PathBuf]) -> String {
    let mut combined = String::new();

    for partial in partials {
        let content = fs::read_to_string(partial).expect("ftl file readable");
        combined.push_str(&content);
        combined.push('\n');
        println!("cargo::rerun-if-changed={}", partial.display());
    }

    combined
}
