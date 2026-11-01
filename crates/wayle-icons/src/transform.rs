//! SVG transformation for GTK symbolic icon compatibility.
//!
//! Parses an arbitrary symbolic-style SVG with `usvg` and re-emits it as a
//! filled-path-only GTK Grappa symbolic icon. Stroked source paths are
//! outlined into filled polygons via `tiny_skia_path::Path::stroke`, so the
//! output never depends on GTK4's symbolic stroke renderer (which is broken
//! across 4.21-4.23 and only repaired in 4.24).
//!
//! See [GNOME/gtk#8147](https://gitlab.gnome.org/GNOME/gtk/-/issues/8147).

use std::fmt::Write;

use usvg::{
    Node, Options, Path as SvgPath, Stroke, Tree,
    tiny_skia_path::{Path, PathSegment, PathStroker, Transform},
};

const TARGET_SIZE: f32 = 16.0;

/// Stroke-outliner precision target.
///
/// Passed to [`tiny_skia_path::Path::stroke`] and [`tiny_skia_path::Path::dash`]
/// as the resolution scale. Higher values produce more polygon segments on
/// curved caps and joins, eliminating visible faceting at the 16x16 output.
const RESOLUTION_SCALE: f32 = 4.0;

/// On-disk format revision stamped onto every emitted SVG via `gpa:version`.
///
/// Bumped any time the geometric output shape changes so the migration code
/// can detect stale files written by older revisions.
pub(crate) const FORMAT_VERSION: u32 = 2;

pub(crate) fn to_symbolic(svg_content: &str) -> Option<String> {
    let Ok(tree) = Tree::from_str(svg_content, &Options::default()) else {
        return build_fallback_svg(svg_content);
    };

    let source_size = tree.size().width().max(tree.size().height());
    let scale = if source_size > 0.0 {
        TARGET_SIZE / source_size
    } else {
        1.0
    };

    let paths = extract_paths(&tree, scale);
    if paths.is_empty() {
        return build_fallback_svg(svg_content);
    }

    Some(build_gtk_svg(&paths))
}

struct ScaledPath {
    d: String,
    is_bounding_box: bool,
}

fn is_bounding_box_path(path: &Path, target_size: f32) -> bool {
    let segments: Vec<_> = path.segments().collect();

    if segments.len() != 5 {
        return false;
    }

    let epsilon = 0.5;

    let corners: Vec<(f32, f32)> = segments
        .iter()
        .filter_map(|seg| match seg {
            PathSegment::MoveTo(p) | PathSegment::LineTo(p) => Some((p.x, p.y)),
            _ => None,
        })
        .collect();

    if corners.len() != 4 {
        return false;
    }

    let mut has_origin = false;
    let mut has_top_right = false;
    let mut has_bottom_right = false;
    let mut has_bottom_left = false;

    for (x, y) in &corners {
        let at_left = x.abs() < epsilon;
        let at_right = (*x - target_size).abs() < epsilon;
        let at_top = y.abs() < epsilon;
        let at_bottom = (*y - target_size).abs() < epsilon;

        if at_left && at_top {
            has_origin = true;
        } else if at_right && at_top {
            has_top_right = true;
        } else if at_right && at_bottom {
            has_bottom_right = true;
        } else if at_left && at_bottom {
            has_bottom_left = true;
        }
    }

    has_origin && has_top_right && has_bottom_right && has_bottom_left
}

fn extract_paths(tree: &Tree, scale: f32) -> Vec<ScaledPath> {
    let mut paths = Vec::new();
    let transform = Transform::from_scale(scale, scale);

    collect_paths_from_group(tree.root(), transform, &mut paths);

    paths
}

fn collect_paths_from_group(
    group: &usvg::Group,
    parent_transform: Transform,
    paths: &mut Vec<ScaledPath>,
) {
    for node in group.children() {
        match node {
            Node::Path(path) => process_path(path, parent_transform, paths),
            Node::Group(child_group) => {
                let combined = parent_transform.pre_concat(child_group.transform());
                collect_paths_from_group(child_group, combined, paths);
            }
            _ => {}
        }
    }
}

fn process_path(source: &SvgPath, parent_transform: Transform, paths: &mut Vec<ScaledPath>) {
    let Some(transformed) = source.data().clone().transform(parent_transform) else {
        return;
    };

    let has_fill = source.fill().is_some();
    let has_stroke = source.stroke().is_some();

    if has_fill {
        push_fill_geometry(&transformed, paths);
    }

    if has_stroke {
        push_stroke_outline(&transformed, source.stroke(), &parent_transform, paths);
    }

    if !has_fill && !has_stroke {
        push_unpainted_geometry(&transformed, paths);
    }
}

fn push_fill_geometry(geometry: &Path, paths: &mut Vec<ScaledPath>) {
    let d = path_data_to_string(geometry);
    if d.is_empty() {
        return;
    }
    paths.push(ScaledPath {
        d,
        is_bounding_box: false,
    });
}

fn push_stroke_outline(
    geometry: &Path,
    stroke: Option<&Stroke>,
    parent_transform: &Transform,
    paths: &mut Vec<ScaledPath>,
) {
    let Some(outlined) = outline_stroke(geometry, stroke, parent_transform) else {
        return;
    };

    let d = path_data_to_string(&outlined);
    if d.is_empty() {
        return;
    }

    paths.push(ScaledPath {
        d,
        is_bounding_box: false,
    });
}

fn push_unpainted_geometry(geometry: &Path, paths: &mut Vec<ScaledPath>) {
    let d = path_data_to_string(geometry);
    if d.is_empty() {
        return;
    }

    let is_bounding_box = is_bounding_box_path(geometry, TARGET_SIZE);
    paths.push(ScaledPath { d, is_bounding_box });
}

fn outline_stroke(
    geometry: &Path,
    stroke: Option<&Stroke>,
    parent_transform: &Transform,
) -> Option<Path> {
    let source_stroke = stroke?;
    let mut tiny_stroke = source_stroke.to_tiny_skia();
    let effective_scale = PathStroker::compute_resolution_scale(parent_transform);
    tiny_stroke.width *= effective_scale;

    if let Some(dash) = tiny_stroke.dash.take() {
        let dashed = geometry.dash(&dash, RESOLUTION_SCALE)?;
        return dashed.stroke(&tiny_stroke, RESOLUTION_SCALE);
    }

    geometry.stroke(&tiny_stroke, RESOLUTION_SCALE)
}

fn path_data_to_string(path: &Path) -> String {
    let mut result = String::with_capacity(256);

    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(point) => {
                write_command(&mut result, 'M', &[point.x, point.y]);
            }
            PathSegment::LineTo(point) => {
                write_command(&mut result, 'L', &[point.x, point.y]);
            }
            PathSegment::QuadTo(ctrl, end) => {
                write_command(&mut result, 'Q', &[ctrl.x, ctrl.y, end.x, end.y]);
            }
            PathSegment::CubicTo(ctrl1, ctrl2, end) => {
                write_command(
                    &mut result,
                    'C',
                    &[ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, end.x, end.y],
                );
            }
            PathSegment::Close => {
                result.push('Z');
            }
        }
    }

    result
}

fn write_command(out: &mut String, cmd: char, coords: &[f32]) {
    out.push(cmd);
    for (i, coord) in coords.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let _ = write!(out, "{:.2}", coord);
    }
}

fn build_gtk_svg(paths: &[ScaledPath]) -> String {
    let mut output = String::with_capacity(512);

    output.push_str("<svg width='16' height='16'\n");
    output.push_str("     xmlns:gpa='https://www.gtk.org/grappa'\n");
    let _ = writeln!(output, "     gpa:version='{FORMAT_VERSION}'>");

    for scaled_path in paths
        .iter()
        .filter(|scaled_path| !scaled_path.is_bounding_box)
    {
        output.push_str(&build_fill_path_element(&scaled_path.d));
    }

    output.push_str("</svg>\n");
    output
}

fn build_fill_path_element(d: &str) -> String {
    format!(
        "  <path d='{d}'\n\
                stroke='none'\n\
                fill='rgb(0,0,0)'\n\
                gpa:fill='foreground'/>\n"
    )
}

fn build_fallback_svg(original: &str) -> Option<String> {
    let d = extract_path_d_fallback(original)?;
    let path = ScaledPath {
        d,
        is_bounding_box: false,
    };
    Some(build_gtk_svg(&[path]))
}

fn extract_path_d_fallback(content: &str) -> Option<String> {
    extract_quoted_attr(content, "d=\"", '"').or_else(|| extract_quoted_attr(content, "d='", '\''))
}

fn extract_quoted_attr(content: &str, prefix: &str, quote: char) -> Option<String> {
    let start = content.find(prefix)? + prefix.len();
    let end = start + content[start..].find(quote)?;
    Some(content[start..end].to_string())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    mod to_symbolic_tests {
        use super::*;

        fn transform(svg: &str) -> String {
            to_symbolic(svg).expect("to_symbolic returned None for input that should produce paths")
        }

        #[test]
        fn outputs_16x16_dimensions() {
            let result =
                transform(r#"<svg viewBox="0 0 24 24"><path d="M12 12" fill="black"/></svg>"#);

            assert!(result.contains("width='16'"));
            assert!(result.contains("height='16'"));
        }

        #[test]
        fn stamps_current_format_version() {
            let result =
                transform(r#"<svg viewBox="0 0 24 24"><path d="M0 0" fill="black"/></svg>"#);

            assert!(result.contains("xmlns:gpa='https://www.gtk.org/grappa'"));
            assert!(
                result.contains(&format!("gpa:version='{FORMAT_VERSION}'")),
                "output must stamp current version: {result}"
            );
        }

        #[test]
        fn scales_coordinates_from_24_to_16() {
            let result =
                transform(r#"<svg viewBox="0 0 24 24"><path d="M0 0L24 24" fill="black"/></svg>"#);

            assert!(
                result.contains("L16.00 16.00"),
                "Expected L24 24 scaled to L16 16, got: {result}"
            );
        }

        #[test]
        fn handles_arc_commands_without_nan() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20.452 3.445a11.002 11.002 0 00-2.482-1.908"/>
            </svg>"#,
            );

            assert!(!result.contains("NaN"), "Arc conversion produced NaN");
            assert!(!result.contains("nan"), "Arc conversion produced nan");
        }

        #[test]
        fn returns_none_when_xml_unparseable_and_no_path_d() {
            assert_eq!(to_symbolic(r#"<svg><not valid xml"#), None);
        }

        #[test]
        fn returns_none_for_completely_broken_input() {
            assert_eq!(to_symbolic("not svg at all"), None);
        }

        #[test]
        fn accumulates_nested_group_transforms() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24">
                <g transform="translate(6, 6)">
                    <g transform="scale(2)">
                        <path d="M0 0L3 3" fill="black"/>
                    </g>
                </g>
            </svg>"#,
            );

            assert!(
                result.contains("d='M"),
                "Expected path in output, got: {result}"
            );
            assert!(
                !result.contains("M0.00 0.00L3.00 3.00"),
                "Transforms should have been applied, got raw coords: {result}"
            );
        }

        #[test]
        fn extracts_multiple_paths_from_svg() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="black">
                <path d="M0 0L12 0"/>
                <path d="M0 12L12 12"/>
            </svg>"#,
            );

            let path_count = result.matches("<path d='").count();
            assert_eq!(
                path_count, 2,
                "Expected 2 paths, got {path_count}: {result}"
            );
        }

        #[test]
        fn output_is_always_filled_geometry() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" stroke="currentColor" fill="none">
                <path d="M0 0L24 24"/>
            </svg>"#,
            );

            assert!(result.contains("gpa:fill='foreground'"), "{result}");
            assert!(!result.contains("gpa:stroke="), "{result}");
            assert!(!result.contains("stroke-linecap"), "{result}");
            assert!(result.contains("stroke='none'"), "{result}");
        }

        #[test]
        fn fill_only_source_emits_fill_output() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M0 0L24 24"/>
            </svg>"#,
            );

            assert!(result.contains("gpa:fill='foreground'"));
            assert!(result.contains("stroke='none'"));
        }

        #[test]
        fn stroked_open_path_becomes_closed_outline() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" stroke="currentColor" fill="none"
                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 12L22 12"/>
            </svg>"#,
            );

            assert!(
                result.matches("Z").count() >= 1,
                "Stroke outline must produce at least one closed subpath: {result}"
            );
        }

        #[test]
        fn serializes_closed_paths_with_z_command() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="black">
                <path d="M0 0L24 0L24 24Z"/>
            </svg>"#,
            );

            assert!(result.contains("Z"), "{result}");
        }

        #[test]
        fn converts_rect_to_path() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="black">
                <rect x="4" y="4" width="16" height="16"/>
            </svg>"#,
            );

            assert!(result.contains("<path d='M"), "{result}");
        }

        #[test]
        fn strips_inkscape_cruft_from_inline_style_attributes() {
            let result = transform(
                r##"<svg viewBox="0 0 16 16"
                xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
                xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd">
                <path style="display:inline;stop-color:#000000;stop-opacity:1"
                      d="M 2 2 L 14 2 L 14 14 L 2 14 Z"/>
            </svg>"##,
            );

            assert!(result.contains("gpa:fill='foreground'"), "{result}");
            assert!(
                !result.contains("sodipodi") && !result.contains("inkscape:"),
                "{result}"
            );
            assert!(!result.contains("style=\""), "{result}");
        }

        #[test]
        fn real_lucide_wifi_produces_only_filled_paths() {
            let lucide_wifi = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-wifi"><path d="M12 20h.01"/><path d="M2 8.82a15 15 0 0 1 20 0"/><path d="M5 12.859a10 10 0 0 1 14 0"/><path d="M8.5 16.429a5 5 0 0 1 7 0"/></svg>"#;

            let result = transform(lucide_wifi);

            assert!(!result.contains("gpa:stroke="), "{result}");
            assert!(!result.contains("stroke-linecap"), "{result}");
            assert!(!result.contains("stroke-linejoin"), "{result}");
            assert!(!result.contains("stroke-width"), "{result}");

            let fill_paths = result.matches("gpa:fill='foreground'").count();
            assert_eq!(fill_paths, 4, "one fill path per source stroke: {result}");
        }

        #[test]
        fn converts_circle_to_path() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="black">
                <circle cx="12" cy="12" r="8"/>
            </svg>"#,
            );

            assert!(result.contains("<path d='M"), "{result}");
        }
    }

    mod build_fill_path_element_tests {
        use super::*;

        #[test]
        fn emits_fill_attributes() {
            let result = build_fill_path_element("M0 0");

            assert!(result.contains("stroke='none'"));
            assert!(result.contains("fill='rgb(0,0,0)'"));
            assert!(result.contains("gpa:fill='foreground'"));
        }

        #[test]
        fn never_emits_stroke_attributes() {
            let result = build_fill_path_element("M0 0");

            assert!(!result.contains("stroke-width"));
            assert!(!result.contains("stroke-linecap"));
            assert!(!result.contains("stroke-linejoin"));
            assert!(!result.contains("gpa:stroke="));
        }
    }

    mod extract_path_d_fallback_tests {
        use super::*;

        #[test]
        fn extracts_path_d_attribute() {
            let svg = r#"<svg><path d="M10 20L30 40"/></svg>"#;
            let result = extract_path_d_fallback(svg);

            assert_eq!(result, Some("M10 20L30 40".to_string()));
        }

        #[test]
        fn returns_none_when_no_path_d() {
            let svg = r#"<svg><rect width="10"/></svg>"#;
            let result = extract_path_d_fallback(svg);

            assert_eq!(result, None);
        }

        #[test]
        fn extracts_first_path_when_multiple_exist() {
            let svg = r#"<svg><path d="M1 1"/><path d="M2 2"/></svg>"#;
            let result = extract_path_d_fallback(svg);

            assert_eq!(result, Some("M1 1".to_string()));
        }

        #[test]
        fn handles_complex_path_data() {
            let svg = r#"<svg><path d="M0 0C1 2 3 4 5 6Z"/></svg>"#;
            let result = extract_path_d_fallback(svg);

            assert_eq!(result, Some("M0 0C1 2 3 4 5 6Z".to_string()));
        }
    }

    mod build_fallback_svg_tests {
        use super::*;

        #[test]
        fn wraps_extracted_path_in_gtk_svg() {
            let result = build_fallback_svg(r#"<svg><path d="M5 5"/></svg>"#)
                .expect("extractable d attribute should produce fallback SVG");

            assert!(result.contains("xmlns:gpa="));
            assert!(result.contains("M5 5"));
            assert!(result.contains("gpa:fill='foreground'"));
        }

        #[test]
        fn returns_none_when_no_path_extractable() {
            assert_eq!(build_fallback_svg(r#"<svg><rect/></svg>"#), None);
        }
    }

    mod bounding_box_tests {
        use super::*;

        fn transform(svg: &str) -> String {
            to_symbolic(svg).expect("to_symbolic returned None for input that should produce paths")
        }

        #[test]
        fn filters_out_bounding_box_rectangle() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="none">
                <rect x="0" y="0" width="24" height="24" stroke="none"/>
                <circle cx="12" cy="12" r="8" stroke="currentColor"/>
            </svg>"#,
            );

            let path_count = result.matches("<path d='").count();
            assert_eq!(
                path_count, 1,
                "Bounding box rect should be filtered, leaving only the circle: {result}"
            );
        }

        #[test]
        fn preserves_non_bounding_box_rectangles() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="black">
                <rect x="4" y="4" width="16" height="16"/>
            </svg>"#,
            );

            assert!(result.contains("<path d='"), "{result}");
        }

        #[test]
        fn preserves_partial_bounding_box() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" fill="black">
                <rect x="0" y="0" width="24" height="12"/>
            </svg>"#,
            );

            assert!(result.contains("<path d='"), "{result}");
        }

        #[test]
        fn preserves_visible_full_size_rectangle() {
            let result = transform(
                r#"<svg viewBox="0 0 24 24" stroke="currentColor" fill="none">
                <rect x="0" y="0" width="24" height="24"/>
                <circle cx="6" cy="6" r="2"/>
            </svg>"#,
            );

            let path_count = result.matches("<path d='").count();
            assert_eq!(
                path_count, 2,
                "Visible full-size rect (like dice outline) should be preserved: {result}"
            );
        }
    }
}
