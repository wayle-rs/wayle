//! Application icon mappings shared across bar components.
//!
//! Patterns use glob syntax and match case-insensitively.
//! Order matters - first match wins.

use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use glob::Pattern;
use gtk4::{gdk, glib};
use wayle_widgets::icons::icon_exists;

struct CompiledEntry {
    pattern: Pattern,
    icon: &'static str,
}

static COMPILED_MAP: OnceLock<Vec<CompiledEntry>> = OnceLock::new();

fn compiled_map() -> &'static [CompiledEntry] {
    COMPILED_MAP.get_or_init(|| {
        DEFAULT_APP_ICON_MAP
            .iter()
            .filter_map(|(glob_str, icon)| {
                Pattern::new(&glob_str.to_lowercase())
                    .ok()
                    .map(|pattern| CompiledEntry { pattern, icon })
            })
            .collect()
    })
}

pub(crate) const DEFAULT_APP_ICON_MAP: &[(&str, &str)] = &[
    // Browsers
    ("*brave*", "si-brave-symbolic"),
    ("*chromium*", "tb-brand-chrome-symbolic"),
    ("*epiphany*", "si-gnome-symbolic"),
    ("*firefox*", "si-firefox-symbolic"),
    ("*floorp*", "si-floorp-symbolic"),
    ("*google-chrome*", "si-googlechrome-symbolic"),
    ("*librewolf*", "si-librewolf-symbolic"),
    ("*microsoft-edge*", "tb-brand-edge-symbolic"),
    ("*min*", "ld-globe-symbolic"),
    ("*nyxt*", "ld-globe-symbolic"),
    ("*opera*", "si-opera-symbolic"),
    ("*qutebrowser*", "ld-globe-symbolic"),
    ("*thorium*", "si-googlechrome-symbolic"),
    ("*tor*", "si-torbrowser-symbolic"),
    ("*ungoogled*", "tb-brand-chrome-symbolic"),
    ("*vivaldi*", "si-vivaldi-symbolic"),
    ("*waterfox*", "si-firefox-symbolic"),
    ("*zen*", "si-zenbrowser-symbolic"),
    ("*helium*", "si-heliumbrowser-symbolic"),
    // Terminals
    ("*alacritty*", "si-alacritty-symbolic"),
    ("*black-box*", "ld-terminal-symbolic"),
    ("*contour*", "ld-terminal-symbolic"),
    ("*cool-retro-term*", "ld-terminal-symbolic"),
    ("*extraterm*", "ld-terminal-symbolic"),
    ("*foot*", "ld-terminal-symbolic"),
    ("*ghostty*", "si-ghostty-symbolic"),
    ("*gnome-terminal*", "ld-terminal-symbolic"),
    ("*guake*", "ld-terminal-symbolic"),
    ("*hyper*", "si-hyper-symbolic"),
    ("*kitty*", "tb-cat-symbolic"),
    ("*konsole*", "ld-terminal-symbolic"),
    ("*rio*", "ld-terminal-symbolic"),
    ("*st", "ld-terminal-symbolic"),
    ("*st-*", "ld-terminal-symbolic"),
    ("*tabby*", "ld-terminal-symbolic"),
    ("*terminator*", "ld-terminal-symbolic"),
    ("*terminology*", "ld-terminal-symbolic"),
    ("*termite*", "ld-terminal-symbolic"),
    ("*tilix*", "ld-terminal-symbolic"),
    ("*urxvt*", "ld-terminal-symbolic"),
    ("*warp*", "si-warp-symbolic"),
    ("*wezterm*", "si-wezterm-symbolic"),
    ("*xfce4-terminal*", "ld-terminal-symbolic"),
    ("*xterm*", "ld-terminal-symbolic"),
    ("*yakuake*", "ld-terminal-symbolic"),
    ("*zellij*", "ld-terminal-symbolic"),
    // Code Editors & IDEs
    ("*android-studio*", "si-androidstudio-symbolic"),
    ("*atom*", "ld-code-symbolic"),
    ("*clion*", "si-clion-symbolic"),
    ("*code*", "tb-brand-vscode-symbolic"),
    ("*cursor*", "si-cursor-symbolic"),
    ("*datagrip*", "si-datagrip-symbolic"),
    ("*emacs*", "si-gnuemacs-symbolic"),
    ("*fleet*", "si-jetbrains-symbolic"),
    ("*geany*", "ld-code-symbolic"),
    ("*goland*", "si-goland-symbolic"),
    ("*helix*", "si-helix-symbolic"),
    ("*idea*", "si-intellijidea-symbolic"),
    ("*intellij*", "si-intellijidea-symbolic"),
    ("*kate*", "ld-code-symbolic"),
    ("*lapce*", "si-lapce-symbolic"),
    ("*neovide*", "si-neovim-symbolic"),
    ("*nvim*", "si-neovim-symbolic"),
    ("*phpstorm*", "si-phpstorm-symbolic"),
    ("*pycharm*", "si-pycharm-symbolic"),
    ("*rider*", "si-rider-symbolic"),
    ("*rubymine*", "si-rubymine-symbolic"),
    ("*rstudio*", "si-rstudioide-symbolic"),
    ("*sublime*", "si-sublimetext-symbolic"),
    ("*vim*", "si-vim-symbolic"),
    ("*webstorm*", "si-webstorm-symbolic"),
    ("*zed*", "si-zedindustries-symbolic"),
    // Communication
    ("*beeper*", "ld-message-circle-symbolic"),
    ("*caprine*", "si-messenger-symbolic"),
    ("*discord*", "si-discord-symbolic"),
    ("*element*", "si-element-symbolic"),
    ("*ferdium*", "ld-message-circle-symbolic"),
    ("*ferdi*", "ld-message-circle-symbolic"),
    ("*franz*", "ld-message-circle-symbolic"),
    ("*geary*", "ld-mail-symbolic"),
    ("*legcord*", "si-discord-symbolic"),
    ("*mailspring*", "ld-mail-symbolic"),
    ("*mattermost*", "si-mattermost-symbolic"),
    ("*rambox*", "ld-message-circle-symbolic"),
    ("*session*", "si-session-symbolic"),
    ("*signal*", "si-signal-symbolic"),
    ("*skype*", "ld-message-circle-symbolic"),
    ("*slack*", "ld-slack-symbolic"),
    ("*teams*", "ld-message-circle-symbolic"),
    ("*telegram*", "si-telegram-symbolic"),
    ("*thunderbird*", "si-thunderbird-symbolic"),
    ("*vesktop*", "si-discord-symbolic"),
    ("equibop", "si-discord-symbolic"),
    ("*webcord*", "si-discord-symbolic"),
    ("*whatsapp*", "si-whatsapp-symbolic"),
    ("*wire*", "si-wire-symbolic"),
    ("*zoom*", "si-zoom-symbolic"),
    ("*zulip*", "si-zulip-symbolic"),
    // File Managers
    ("*caja*", "ld-folder-symbolic"),
    ("*doublecmd*", "ld-folder-symbolic"),
    ("*dolphin*", "ld-folder-symbolic"),
    ("*krusader*", "ld-folder-symbolic"),
    ("*nautilus*", "ld-folder-symbolic"),
    ("*nemo*", "ld-folder-symbolic"),
    ("*pcmanfm*", "ld-folder-symbolic"),
    ("*ranger*", "ld-folder-symbolic"),
    ("*spacefm*", "ld-folder-symbolic"),
    ("*thunar*", "ld-folder-symbolic"),
    ("*yazi*", "ld-folder-symbolic"),
    // Media Players
    ("*amberol*", "ld-music-symbolic"),
    ("*audacious*", "si-audacity-symbolic"),
    ("*celluloid*", "ld-play-symbolic"),
    ("*cider*", "si-applemusic-symbolic"),
    ("*clementine*", "ld-music-symbolic"),
    ("*elisa*", "ld-music-symbolic"),
    ("*feishin*", "ld-music-symbolic"),
    ("*haruna*", "ld-play-symbolic"),
    ("*jellyfin*", "si-jellyfin-symbolic"),
    ("*lollypop*", "ld-music-symbolic"),
    ("*mpv*", "si-mpv-symbolic"),
    ("*nuclear*", "ld-music-symbolic"),
    ("*plex*", "si-plex-symbolic"),
    ("*rhythmbox*", "ld-music-symbolic"),
    ("*spotify*", "si-spotify-symbolic"),
    ("*strawberry*", "ld-music-symbolic"),
    ("*tidal*", "si-tidal-symbolic"),
    ("*vlc*", "si-vlcmediaplayer-symbolic"),
    // Video Editors
    ("*blender*", "si-blender-symbolic"),
    ("*davinci*", "si-davinciresolve-symbolic"),
    ("*kdenlive*", "si-kdenlive-symbolic"),
    ("*olive*", "ld-film-symbolic"),
    ("*openshot*", "ld-film-symbolic"),
    ("*pitivi*", "ld-film-symbolic"),
    ("*shotcut*", "ld-film-symbolic"),
    // Graphics & Design
    ("*darktable*", "ld-camera-symbolic"),
    ("*figma*", "si-figma-symbolic"),
    ("*gimp*", "si-gimp-symbolic"),
    ("*inkscape*", "si-inkscape-symbolic"),
    ("*krita*", "si-krita-symbolic"),
    ("*penpot*", "si-penpot-symbolic"),
    ("*rawtherapee*", "ld-camera-symbolic"),
    // Games & Gaming
    ("*bottles*", "ld-wine-symbolic"),
    ("*gamescope*", "si-steam-symbolic"),
    ("*heroic*", "si-heroicgameslauncher-symbolic"),
    ("*lutris*", "si-lutris-symbolic"),
    ("*minecraft*", "ld-gamepad-2-symbolic"),
    ("*polymc*", "ld-gamepad-2-symbolic"),
    ("*prism*", "ld-gamepad-2-symbolic"),
    ("*retroarch*", "si-retroarch-symbolic"),
    ("*steam*", "si-steam-symbolic"),
    // Office & Productivity
    ("*calibre*", "si-calibreweb-symbolic"),
    ("*evince*", "ld-file-text-symbolic"),
    ("*joplin*", "si-joplin-symbolic"),
    ("libreoffice-base", "si-libreofficebase-symbolic"),
    ("libreoffice-calc", "si-libreofficecalc-symbolic"),
    ("libreoffice-draw", "si-libreofficedraw-symbolic"),
    ("libreoffice-math", "si-libreofficemath-symbolic"),
    ("libreoffice-writer", "si-libreofficewriter-symbolic"),
    ("libreoffice-impress", "si-libreofficeimpress-symbolic"),
    ("libreoffice*", "si-libreoffice-symbolic"),
    ("*logseq*", "si-logseq-symbolic"),
    ("*marktext*", "ld-file-text-symbolic"),
    ("*notion*", "si-notion-symbolic"),
    ("*obsidian*", "si-obsidian-symbolic"),
    ("*okular*", "ld-file-text-symbolic"),
    ("*onlyoffice*", "si-onlyoffice-symbolic"),
    ("*sioyek*", "ld-file-text-symbolic"),
    ("*typora*", "ld-file-text-symbolic"),
    ("*zathura*", "ld-file-text-symbolic"),
    ("*zettlr*", "ld-file-text-symbolic"),
    // Cloud & Sync
    ("*dropbox*", "si-dropbox-symbolic"),
    ("*mega*", "si-mega-symbolic"),
    ("*nextcloud*", "si-nextcloud-symbolic"),
    ("*owncloud*", "si-owncloud-symbolic"),
    ("*syncthing*", "si-syncthing-symbolic"),
    // Password Managers
    ("*1password*", "si-1password-symbolic"),
    ("*bitwarden*", "si-bitwarden-symbolic"),
    ("*enpass*", "si-enpass-symbolic"),
    ("*keepassxc*", "si-keepassxc-symbolic"),
    ("*lastpass*", "si-lastpass-symbolic"),
    // System & Settings
    ("*dconf*", "ld-settings-symbolic"),
    ("*gnome-control-center*", "ld-settings-symbolic"),
    ("*gnome-tweaks*", "ld-settings-symbolic"),
    ("*systemsettings*", "ld-settings-symbolic"),
    // Utilities
    ("*baobab*", "ld-pie-chart-symbolic"),
    ("*gnome-disks*", "ld-hard-drive-symbolic"),
    ("*gparted*", "ld-hard-drive-symbolic"),
    ("*htop*", "ld-activity-symbolic"),
    ("*mission-center*", "ld-activity-symbolic"),
    ("*resources*", "ld-activity-symbolic"),
    ("*stacer*", "ld-activity-symbolic"),
    ("*timeshift*", "ld-clock-symbolic"),
    ("*virt-manager*", "si-qemu-symbolic"),
    // Misc
    ("*anydesk*", "si-anydesk-symbolic"),
    ("*filezilla*", "si-filezilla-symbolic"),
    ("*gitkraken*", "si-gitkraken-symbolic"),
    ("*insomnia*", "si-insomnia-symbolic"),
    ("*obs*", "si-obsstudio-symbolic"),
    ("*parsec*", "ld-monitor-symbolic"),
    ("*postman*", "si-postman-symbolic"),
    ("*qbittorrent*", "si-qbittorrent-symbolic"),
    ("*remmina*", "ld-monitor-symbolic"),
    ("*rustdesk*", "si-rustdesk-symbolic"),
    ("*sunshine*", "ld-sun-symbolic"),
    ("*transmission*", "si-transmission-symbolic"),
    ("*waydroid*", "si-android-symbolic"),
    ("*wireshark*", "si-wireshark-symbolic"),
];

/// Matches text against a glob pattern (case-insensitive).
pub(crate) fn matches_glob(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();

    if text_lower == pattern {
        return true;
    }

    Pattern::new(pattern)
        .map(|compiled| compiled.matches(&text_lower))
        .unwrap_or(false)
}

/// Looks up an icon from the default app icon map by matching against the given name.
pub(crate) fn lookup_app_icon(name: &str) -> Option<&'static str> {
    let name_lower = name.to_lowercase();

    compiled_map()
        .iter()
        .find(|entry| entry.pattern.matches(&name_lower))
        .map(|entry| entry.icon)
}

thread_local! {
    /// Caches `identifier -> symbolic icon name` resolutions. App→icon mappings
    /// are stable, so this avoids repeated desktop-file and theme lookups on
    /// every redraw. `None` (no symbolic variant) is cached too.
    static SYMBOLIC_DESKTOP_CACHE: RefCell<HashMap<String, Option<String>>> =
        RefCell::new(HashMap::new());
}

/// Resolves the `-symbolic` variant of an app's desktop-entry icon.
///
/// `identifier` is a desktop-entry id or window class (e.g. `org.gnome.Calendar`
/// or `firefox`). Returns the symbolic icon name only if such a variant exists
/// in the current icon theme — never a full-colour icon — so callers fall
/// through to their own generic fallback otherwise. No-op (returns `None`) when
/// there is no GDK display, so it is safe to call off-screen (e.g. in tests).
pub(crate) fn symbolic_desktop_icon(identifier: &str) -> Option<String> {
    if identifier.is_empty() {
        return None;
    }

    SYMBOLIC_DESKTOP_CACHE.with(|cache| {
        if let Some(cached) = cache.borrow().get(identifier) {
            return cached.clone();
        }
        let resolved = resolve_symbolic_desktop_icon(identifier);
        cache
            .borrow_mut()
            .insert(identifier.to_owned(), resolved.clone());
        resolved
    })
}

fn resolve_symbolic_desktop_icon(identifier: &str) -> Option<String> {
    // Icon-theme lookups require a GDK display; bail out (rather than panic in
    // `icon_exists`) when there isn't one.
    gdk::Display::default()?;

    let icon = desktop_entry_icon(identifier)?;
    let base = icon.strip_suffix("-symbolic").unwrap_or(&icon);
    let symbolic = format!("{base}-symbolic");

    icon_exists(&symbolic).then_some(symbolic)
}

thread_local! {
    /// Caches `identifier -> colour icon name` resolutions, mirroring
    /// [`SYMBOLIC_DESKTOP_CACHE`]. `None` (no colour variant) is cached too.
    static COLOR_DESKTOP_CACHE: RefCell<HashMap<String, Option<String>>> =
        RefCell::new(HashMap::new());
}

/// Resolves the full-colour variant of an app's desktop-entry icon.
///
/// The counterpart to [`symbolic_desktop_icon`]: returns the desktop entry's `Icon=`
/// name only when it is non-symbolic and exists in the current icon theme (never a
/// `-symbolic` name and never a bare filesystem path), so callers fall through to a
/// symbolic icon otherwise. No-op (returns `None`) when there is no GDK display.
pub(crate) fn color_desktop_icon(identifier: &str) -> Option<String> {
    if identifier.is_empty() {
        return None;
    }

    COLOR_DESKTOP_CACHE.with(|cache| {
        if let Some(cached) = cache.borrow().get(identifier) {
            return cached.clone();
        }
        let resolved = resolve_color_desktop_icon(identifier);
        cache
            .borrow_mut()
            .insert(identifier.to_owned(), resolved.clone());
        resolved
    })
}

fn resolve_color_desktop_icon(identifier: &str) -> Option<String> {
    gdk::Display::default()?;

    let icon = desktop_entry_icon(identifier)?;
    (!icon.ends_with("-symbolic") && icon_exists(&icon)).then_some(icon)
}

/// Reads the `Icon=` value from an app's desktop entry — by desktop id first,
/// then by matching `StartupWMClass`.
///
/// Reads the `.desktop` files directly (via `glib::KeyFile`) instead of going
/// through `gio::DesktopAppInfo` / `AppInfo::all()`, which silently drop any app
/// whose `Exec` program is not found in `$PATH`. That filtering would otherwise
/// hide an app's icon whenever Wayle runs with a restricted `$PATH` (e.g. as a
/// systemd user service whose unit sets a minimal `PATH`) even though the icon is
/// installed — an app's icon does not depend on its binary being runnable.
fn desktop_entry_icon(identifier: &str) -> Option<String> {
    let ids = [
        format!("{identifier}.desktop"),
        format!("{}.desktop", identifier.to_lowercase()),
    ];
    let dirs = application_dirs();

    for id in &ids {
        for dir in &dirs {
            if let Some(icon) = read_desktop_icon(&dir.join(id)) {
                return Some(icon);
            }
        }
    }

    icon_by_startup_wm_class(&dirs, identifier)
}

/// The `applications` subdirectory of every XDG data directory
/// (`$XDG_DATA_HOME` then each entry of `$XDG_DATA_DIRS`).
fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![glib::user_data_dir()];
    dirs.extend(glib::system_data_dirs());
    dirs.into_iter()
        .map(|dir| dir.join("applications"))
        .collect()
}

/// Reads `[Desktop Entry] Icon=` from a `.desktop` file, if present and non-empty.
fn read_desktop_icon(path: &Path) -> Option<String> {
    let keyfile = glib::KeyFile::new();
    keyfile.load_from_file(path, glib::KeyFileFlags::NONE).ok()?;
    desktop_icon_key(&keyfile)
}

fn desktop_icon_key(keyfile: &glib::KeyFile) -> Option<String> {
    keyfile
        .string("Desktop Entry", "Icon")
        .ok()
        .map(|icon| icon.to_string())
        .filter(|icon| !icon.is_empty())
}

/// Scans `.desktop` files for one whose `StartupWMClass` matches `wm_class`
/// (case-insensitively) and returns its `Icon=`.
fn icon_by_startup_wm_class(dirs: &[PathBuf], wm_class: &str) -> Option<String> {
    let wm_class_lower = wm_class.to_lowercase();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
                continue;
            }
            let keyfile = glib::KeyFile::new();
            if keyfile.load_from_file(&path, glib::KeyFileFlags::NONE).is_err() {
                continue;
            }
            let matches = keyfile
                .string("Desktop Entry", "StartupWMClass")
                .ok()
                .is_some_and(|class| class.to_lowercase() == wm_class_lower);
            if matches
                && let Some(icon) = desktop_icon_key(&keyfile)
            {
                return Some(icon);
            }
        }
    }
    None
}
