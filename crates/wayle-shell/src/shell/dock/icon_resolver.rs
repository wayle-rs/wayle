//! Application icon resolution for dock items.
//!
//! Resolves an application icon name, falling back to desktop entry lookup
//! and icon theme search when the hardcoded map has no match.

use std::collections::HashMap;
use std::sync::OnceLock;

pub(crate) fn resolve_app_icon(name: &str) -> std::borrow::Cow<'static, str> {
    if let Some(icon) = crate::shell::bar::icons::lookup_app_icon(name) {
        return std::borrow::Cow::Borrowed(icon);
    }

    let name_lower = name.to_lowercase();

    let cache = DESKTOP_APP_CACHE.get_or_init(|| std::sync::RwLock::new(HashMap::new()));
    if let Ok(cache_read) = cache.read() {
        if let Some(cached) = cache_read.get(&name_lower) {
            return std::borrow::Cow::Owned(cached.clone());
        }
    }

    let result = find_desktop_entry_icon(&name_lower).map(|s| s.into_owned());
    if let Some(ref icon) = result {
        if let Ok(mut cache_write) = cache.write() {
            cache_write.insert(name_lower.clone(), icon.clone());
        }
    }

    result.map(std::borrow::Cow::Owned).unwrap_or_else(|| {
        let app_id = name.to_string();
        if app_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
            && app_id.starts_with(|c: char| c.is_alphabetic())
        {
            std::borrow::Cow::Owned(app_id)
        } else if let Some(last) = app_id.split('.').last() {
            std::borrow::Cow::Owned(last.to_string())
        } else {
            std::borrow::Cow::Borrowed("application-x-executable")
        }
    })
}

static DESKTOP_APP_CACHE: OnceLock<std::sync::RwLock<HashMap<String, String>>> = OnceLock::new();

fn find_desktop_entry_icon(name_lower: &str) -> Option<std::borrow::Cow<'static, str>> {
    use gio_unix::prelude::{AppInfoExt, IconExt};

    let desktop_ids = [
        format!("{name_lower}.desktop"),
        format!("{}-launcher.desktop", name_lower),
    ];
    for desktop_id in &desktop_ids {
        if let Some(app_info) = gio_unix::DesktopAppInfo::new(desktop_id) {
            if let Some(icon_str) = app_info.icon()?.to_string() {
                return Some(std::borrow::Cow::Owned(icon_str.into()));
            }
        }
    }
    icon_theme_lookup(name_lower)
}

fn icon_theme_lookup(name_lower: &str) -> Option<std::borrow::Cow<'static, str>> {
    let icons = [
        name_lower.to_string(),
        format!("{}-symbolic", name_lower),
        format!("{}-dark", name_lower),
        format!("{}-light", name_lower),
    ];

    let display = gdk4::Display::default()?;
    let theme = gtk4::IconTheme::for_display(&display);
    for icon_name in &icons {
        if theme.has_icon(icon_name) {
            return Some(std::borrow::Cow::Owned(icon_name.clone()));
        }
    }

    None
}
