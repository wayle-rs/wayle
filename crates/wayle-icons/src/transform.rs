//! SVG transformation for GTK symbolic icon compatibility.
//!
//! Uses `usvg` for robust SVG parsing and path manipulation.
//!
//! # Why This Exists
//!
//! GTK symbolic icons require specific attributes (`gpa:stroke`, `gpa:fill`)
//! from the Grappa namespace for CSS color recoloring. Most icon libraries
//! don't include these, so we transform standard SVGs into GTK-compatible format.
//!
//! # What This Module Does
//!
//! 1. **Parses SVG** using `usvg` for correct handling of all path commands
//! 2. **Scales coordinates** from source size (typically 24x24) to 16x16
//! 3. **Adds GTK Grappa attributes** for CSS color support
//! 4. **Detects stroke vs fill** icons and applies appropriate attributes

use std::fmt::Write;

use usvg::{
    Node, Options, Tree,
    tiny_skia_path::{PathSegment, Transform},
};

const TARGET_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IconStyle {
    Stroke,
    Fill,
}

pub(crate) fn to_symbolic(svg_content: &str) -> String {
    let style = detect_icon_style(svg_content);

    let tree = match Tree::from_str(svg_content, &Options::default()) {
        Ok(tree) => tree,
        Err(_) => return build_fallback_svg(svg_content, style),
    };

    let source_size = tree.size().width().max(tree.size().height());
    let scale = if source_size > 0.0 {
        TARGET_SIZE / source_size
    } else {
        1.0
    };

    let paths = extract_paths(&tree, scale);

    if paths.is_empty() {
        return build_fallback_svg(svg_content, style);
    }

    build_gtk_svg(&paths, style, scale)
}

fn detect_icon_style(content: &str) -> IconStyle {
    if content.contains(r#"stroke="currentColor""#) {
        IconStyle::Stroke
    } else {
        IconStyle::Fill
    }
}

struct ScaledPath {
    d: String,
    stroke_width: Option<f32>,
    is_bounding_box: bool,
}

fn is_bounding_box_path(path: &usvg::tiny_skia_path::Path, target_size: f32) -> bool {
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

    collect_paths_from_group(tree.root(), transform, &mut paths, scale);

    paths
}

fn collect_paths_from_group(
    group: &usvg::Group,
    parent_transform: Transform,
    paths: &mut Vec<ScaledPath>,
    scale: f32,
) {
    for node in group.children() {
        match node {
            Node::Path(path) => {
                let transformed = path.data().clone().transform(parent_transform);

                if let Some(transformed_path) = transformed {
                    let d = path_data_to_string(&transformed_path);

                    if !d.is_empty() {
                        let stroke_width = path.stroke().map(|s| s.width().get() * scale);
                        let has_visible_paint = path.fill().is_some() || path.stroke().is_some();
                        let is_bounding_box = !has_visible_paint
                            && is_bounding_box_path(&transformed_path, TARGET_SIZE);
                        paths.push(ScaledPath {
                            d,
                            stroke_width,
                            is_bounding_box,
                        });
                    }
                }
            }
            Node::Group(child_group) => {
                let combined = parent_transform.pre_concat(child_group.transform());
                collect_paths_from_group(child_group, combined, paths, scale);
            }
            _ => {}
        }
    }
}

fn path_data_to_string(path: &usvg::tiny_skia_path::Path) -> String {
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

fn build_gtk_svg(paths: &[ScaledPath], style: IconStyle, scale: f32) -> String {
    let mut output = String::with_capacity(512);

    output.push_str("<svg width='16' height='16'\n");
    output.push_str("     xmlns:gpa='https://www.gtk.org/grappa'\n");
    output.push_str("     gpa:version='1'>\n");

    for path in paths.iter().filter(|p| !p.is_bounding_box) {
        let path_element = build_path_element(&path.d, style, path.stroke_width, scale);
        output.push_str(&path_element);
    }

    output.push_str("</svg>\n");
    output
}

fn build_path_element(d: &str, style: IconStyle, stroke_width: Option<f32>, scale: f32) -> String {
    match style {
        IconStyle::Stroke => {
            let width = stroke_width.unwrap_or(2.0 * scale);
            format!(
                "  <path d='{d}'\n\
                        stroke-width='{width:.2}'\n\
                        stroke-linecap='round'\n\
                        stroke-linejoin='round'\n\
                        stroke='rgb(0,0,0)'\n\
                        fill='none'\n\
                        gpa:stroke='foreground'/>\n"
            )
        }
        IconStyle::Fill => {
            format!(
                "  <path d='{d}'\n\
                        stroke='none'\n\
                        fill='rgb(0,0,0)'\n\
                        gpa:fill='foreground'/>\n"
            )
        }
    }
}

fn build_fallback_svg(original: &str, style: IconStyle) -> String {
    if let Some(d) = extract_path_d_fallback(original) {
        let path = ScaledPath {
            d,
            stroke_width: None,
            is_bounding_box: false,
        };
        build_gtk_svg(&[path], style, 1.0_f32)
    } else {
        String::from("<svg width='16' height='16'/>")
    }
}

fn extract_path_d_fallback(content: &str) -> Option<String> {
    let start = content.find("d=\"")? + 3;
    let end = start + content[start..].find('"')?;
    Some(content[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod detect_icon_style_tests {
        use super::*;

        #[test]
        fn returns_stroke_when_stroke_currentcolor_present() {
            let svg = r#"<svg stroke="currentColor"><path d="M0 0"/></svg>"#;
            assert_eq!(detect_icon_style(svg), IconStyle::Stroke);
        }

        #[test]
        fn returns_fill_when_fill_currentcolor_present() {
            let svg = r#"<svg fill="currentColor"><path d="M0 0"/></svg>"#;
            assert_eq!(detect_icon_style(svg), IconStyle::Fill);
        }

        #[test]
        fn returns_fill_when_neither_stroke_nor_fill_specified() {
            let svg = r#"<svg><path d="M0 0"/></svg>"#;
            assert_eq!(detect_icon_style(svg), IconStyle::Fill);
        }

        #[test]
        fn returns_stroke_when_both_present_stroke_checked_first() {
            let svg = r#"<svg stroke="currentColor" fill="currentColor"><path/></svg>"#;
            assert_eq!(detect_icon_style(svg), IconStyle::Stroke);
        }
    }

    mod to_symbolic_tests {
        use super::*;

        #[test]
        fn outputs_16x16_dimensions() {
            let svg = r#"<svg viewBox="0 0 24 24"><path d="M12 12"/></svg>"#;
            let result = to_symbolic(svg);

            assert!(result.contains("width='16'"));
            assert!(result.contains("height='16'"));
        }

        #[test]
        fn includes_grappa_namespace() {
            let svg = r#"<svg viewBox="0 0 24 24"><path d="M0 0"/></svg>"#;
            let result = to_symbolic(svg);

            assert!(result.contains("xmlns:gpa='https://www.gtk.org/grappa'"));
            assert!(result.contains("gpa:version='1'"));
        }

        #[test]
        fn scales_coordinates_from_24_to_16() {
            let svg = r#"<svg viewBox="0 0 24 24"><path d="M0 0L24 24"/></svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("L16.00 16.00"),
                "Expected L24 24 scaled to L16 16, got: {}",
                result
            );
        }

        #[test]
        fn handles_arc_commands_without_nan() {
            let svg = r#"<svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M20.452 3.445a11.002 11.002 0 00-2.482-1.908"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(!result.contains("NaN"), "Arc conversion produced NaN");
            assert!(!result.contains("nan"), "Arc conversion produced nan");
        }

        #[test]
        fn uses_fallback_for_invalid_svg() {
            let invalid_svg = r#"<svg><not valid xml"#;
            let result = to_symbolic(invalid_svg);

            assert!(result.contains("width='16'"));
        }

        #[test]
        fn returns_empty_svg_for_completely_broken_input() {
            let garbage = "not svg at all";
            let result = to_symbolic(garbage);

            assert_eq!(result, "<svg width='16' height='16'/>");
        }

        #[test]
        fn accumulates_nested_group_transforms() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <g transform="translate(6, 6)">
                    <g transform="scale(2)">
                        <path d="M0 0L3 3"/>
                    </g>
                </g>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("d='M"),
                "Expected path in output, got: {}",
                result
            );
            assert!(
                !result.contains("M0.00 0.00L3.00 3.00"),
                "Transforms should have been applied, got raw coords: {}",
                result
            );
        }

        #[test]
        fn extracts_multiple_paths_from_svg() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <path d="M0 0L12 0"/>
                <path d="M0 12L12 12"/>
            </svg>"#;
            let result = to_symbolic(svg);

            let path_count = result.matches("<path d='").count();
            assert_eq!(
                path_count, 2,
                "Expected 2 paths, got {}: {}",
                path_count, result
            );
        }

        #[test]
        fn stroke_icon_gets_gpa_stroke_attribute() {
            let svg = r#"<svg viewBox="0 0 24 24" stroke="currentColor">
                <path d="M0 0L24 24"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("gpa:stroke='foreground'"),
                "Stroke icon should have gpa:stroke attribute, got: {}",
                result
            );
            assert!(
                result.contains("stroke-linecap='round'"),
                "Stroke icon should have linecap, got: {}",
                result
            );
        }

        #[test]
        fn fill_icon_gets_gpa_fill_attribute() {
            let svg = r#"<svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M0 0L24 24"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("gpa:fill='foreground'"),
                "Fill icon should have gpa:fill attribute, got: {}",
                result
            );
            assert!(
                result.contains("stroke='none'"),
                "Fill icon should have stroke='none', got: {}",
                result
            );
        }

        #[test]
        fn serializes_closed_paths_with_z_command() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <path d="M0 0L24 0L24 24Z"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("Z"),
                "Closed path should contain Z command, got: {}",
                result
            );
        }

        #[test]
        fn serializes_quadratic_bezier_curves() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <path d="M0 0Q12 24 24 0"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("Q"),
                "Quadratic bezier should contain Q command, got: {}",
                result
            );
        }

        #[test]
        fn preserves_stroke_width_from_source() {
            let svg = r#"<svg viewBox="0 0 24 24" stroke="currentColor">
                <path d="M0 0L24 24" stroke-width="3"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("stroke-width='2.00'"),
                "Stroke width 3 scaled by 16/24 should be 2.00, got: {}",
                result
            );
        }

        #[test]
        fn converts_rect_to_path() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <rect x="4" y="4" width="16" height="16"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("<path d='M"),
                "Rect should be converted to path, got: {}",
                result
            );
        }

        #[test]
        fn strips_inkscape_cruft_from_inline_style_attributes() {
            let svg = r##"<svg viewBox="0 0 16 16"
                xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
                xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd">
                <path style="display:inline;stop-color:#000000;stop-opacity:1"
                      d="M 2 2 L 14 2 L 14 14 L 2 14 Z"/>
            </svg>"##;
            let result = to_symbolic(svg);

            assert!(
                result.contains("gpa:fill='foreground'"),
                "Inkscape-exported SVG should produce recolor-capable output, got: {result}"
            );
            assert!(
                !result.contains("sodipodi") && !result.contains("inkscape:"),
                "Editor namespaces must be stripped, got: {result}"
            );
            assert!(
                !result.contains("style=\""),
                "Inline style attributes must be stripped, got: {result}"
            );
        }

        #[test]
        fn converts_circle_to_path() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="8"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("<path d='M"),
                "Circle should be converted to path, got: {}",
                result
            );
        }
    }

    mod build_path_element_tests {
        use super::*;

        #[test]
        fn stroke_style_includes_stroke_attributes() {
            let result = build_path_element("M0 0", IconStyle::Stroke, None, 0.667);

            assert!(result.contains("stroke-linecap='round'"));
            assert!(result.contains("stroke-linejoin='round'"));
            assert!(result.contains("fill='none'"));
            assert!(result.contains("gpa:stroke='foreground'"));
        }

        #[test]
        fn fill_style_includes_fill_attributes() {
            let result = build_path_element("M0 0", IconStyle::Fill, None, 0.667);

            assert!(result.contains("stroke='none'"));
            assert!(result.contains("fill='rgb(0,0,0)'"));
            assert!(result.contains("gpa:fill='foreground'"));
        }

        #[test]
        fn stroke_width_uses_provided_value_when_present() {
            let result = build_path_element("M0 0", IconStyle::Stroke, Some(1.5), 0.667);

            assert!(
                result.contains("stroke-width='1.50'"),
                "Expected stroke-width='1.50', got: {}",
                result
            );
        }

        #[test]
        fn stroke_width_uses_default_scaled_when_none() {
            let scale = 0.5;
            let result = build_path_element("M0 0", IconStyle::Stroke, None, scale);

            let expected_width = 2.0 * scale;
            let expected = format!("stroke-width='{:.2}'", expected_width);
            assert!(
                result.contains(&expected),
                "Expected {}, got: {}",
                expected,
                result
            );
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
            let svg = r#"<svg><path d="M5 5"/></svg>"#;
            let result = build_fallback_svg(svg, IconStyle::Fill);

            assert!(result.contains("xmlns:gpa="));
            assert!(result.contains("M5 5"));
            assert!(result.contains("gpa:fill='foreground'"));
        }

        #[test]
        fn returns_empty_svg_when_no_path_extractable() {
            let svg = r#"<svg><rect/></svg>"#;
            let result = build_fallback_svg(svg, IconStyle::Fill);

            assert_eq!(result, "<svg width='16' height='16'/>");
        }
    }

    mod bounding_box_tests {
        use super::*;

        #[test]
        fn filters_out_bounding_box_rectangle() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <rect x="0" y="0" width="24" height="24" fill="none" stroke="none"/>
                <circle cx="12" cy="12" r="8" stroke="currentColor"/>
            </svg>"#;
            let result = to_symbolic(svg);

            let path_count = result.matches("<path d='").count();
            assert_eq!(
                path_count, 1,
                "Bounding box rect should be filtered, leaving only the circle: {}",
                result
            );
        }

        #[test]
        fn preserves_non_bounding_box_rectangles() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <rect x="4" y="4" width="16" height="16"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("<path d='"),
                "Smaller rectangle should be preserved: {}",
                result
            );
        }

        #[test]
        fn preserves_partial_bounding_box() {
            let svg = r#"<svg viewBox="0 0 24 24">
                <rect x="0" y="0" width="24" height="12"/>
            </svg>"#;
            let result = to_symbolic(svg);

            assert!(
                result.contains("<path d='"),
                "Partial bounding rect should be preserved: {}",
                result
            );
        }

        #[test]
        fn preserves_visible_full_size_rectangle() {
            let svg = r#"<svg viewBox="0 0 24 24" stroke="currentColor">
                <rect x="0" y="0" width="24" height="24"/>
                <circle cx="6" cy="6" r="2"/>
            </svg>"#;
            let result = to_symbolic(svg);

            let path_count = result.matches("<path d='").count();
            assert_eq!(
                path_count, 2,
                "Visible full-size rect (like dice outline) should be preserved: {}",
                result
            );
        }
    }
}
