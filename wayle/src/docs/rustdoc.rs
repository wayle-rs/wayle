//! Shared helpers for rehosting Rust doc comments as VitePress markdown.
//!
//! A schema's rustdoc often includes its own headings, tables, and fenced code
//! blocks. When the generator embeds that description under another heading,
//! the embedded headings need to shift deeper so they slot correctly under the
//! host page's outline.

/// Shifts ATX headings in `description` so they render below `enclosing_depth`.
/// Headings inside fenced code blocks pass through unchanged.
///
/// The caller is responsible for wrapping the returned content in `<div v-pre>`
/// or `::: v-pre` so Jinja-style mustaches in prose (`{{ output }}`) and
/// HTML-like prose tokens don't trip VitePress's Vue template compiler.
pub fn rehost_rustdoc(description: &str, enclosing_depth: usize) -> String {
    const MAX_DEPTH: usize = 6;
    let shift = enclosing_depth.saturating_sub(1);
    let min_depth = (enclosing_depth + 1).min(MAX_DEPTH);

    let mut rehosted = String::with_capacity(description.len());
    let mut inside_code_fence = false;

    for line in description.lines() {
        if is_fence_line(line) {
            inside_code_fence = !inside_code_fence;
            rehosted.push_str(line);
            rehosted.push('\n');
            continue;
        }

        if inside_code_fence {
            rehosted.push_str(line);
            rehosted.push('\n');
            continue;
        }

        match parse_atx_heading(line) {
            Some((heading_depth, body)) => {
                let new_depth = (heading_depth + shift).max(min_depth).min(MAX_DEPTH);
                for _ in 0..new_depth {
                    rehosted.push('#');
                }
                rehosted.push_str(body);
                rehosted.push('\n');
            }
            None => {
                rehosted.push_str(line);
                rehosted.push('\n');
            }
        }
    }

    rehosted
}

/// `true` when `line` opens or closes a fenced code block (backtick or tilde).
pub fn is_fence_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// Splits an ATX heading line into `(depth, body)` where `body` starts with
/// the space that followed the hashes. Rejects bare hashes, 7+ hashes, and
/// hashes not followed by a space.
pub fn parse_atx_heading(line: &str) -> Option<(usize, &str)> {
    let trimmed_start = line.trim_start();
    let hash_count = trimmed_start
        .bytes()
        .take_while(|&byte| byte == b'#')
        .count();

    if hash_count == 0 || hash_count > 6 {
        return None;
    }

    let after_hashes = &trimmed_start[hash_count..];
    if !after_hashes.starts_with(' ') {
        return None;
    }

    Some((hash_count, after_hashes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUBSECTION_DEPTH: usize = 3;

    #[test]
    fn rehost_shifts_headings_into_h3_subsection() {
        let source = "summary\n\n## Common\n\n- item\n\n## Examples\n\n- `x`";
        let rehosted = rehost_rustdoc(source, SUBSECTION_DEPTH);

        assert!(rehosted.contains("\n#### Common\n"));
        assert!(rehosted.contains("\n#### Examples\n"));
        assert!(!rehosted.contains("\n## "));
    }

    #[test]
    fn rehost_clamps_shallow_headings_below_enclosing_depth() {
        let rehosted = rehost_rustdoc("# Top", SUBSECTION_DEPTH);
        assert!(rehosted.starts_with("#### Top"));
    }

    #[test]
    fn rehost_caps_depth_at_six() {
        let rehosted = rehost_rustdoc("###### Already at six", SUBSECTION_DEPTH);
        assert!(rehosted.contains("###### Already at six"));
        assert!(!rehosted.contains("####### "));
    }

    #[test]
    fn rehost_leaves_fenced_code_alone() {
        let source = "summary\n\n```toml\n## not a heading\n```";
        let rehosted = rehost_rustdoc(source, SUBSECTION_DEPTH);
        assert!(rehosted.contains("\n## not a heading\n"));
    }

    #[test]
    fn parse_atx_heading_rejects_no_space() {
        assert!(parse_atx_heading("##Foo").is_none());
    }

    #[test]
    fn parse_atx_heading_rejects_seven_hashes() {
        assert!(parse_atx_heading("####### too deep").is_none());
    }

    #[test]
    fn parse_atx_heading_accepts_indented() {
        assert_eq!(parse_atx_heading("   ## Indented"), Some((2, " Indented")));
    }
}
