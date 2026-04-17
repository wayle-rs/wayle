//! Window class icon mappings for the window title module.

/// Built-in window class-to-icon mappings.
///
/// Patterns use glob syntax and match against window class names.
/// Order matters - first match wins.
///
/// TODO: Replace all generic glob patterns with specific classes (*code* -> code).
/// This requires launching each application individually and looking at its
/// window class. If you use one of these applications - please contribute a patch.
///
/// This is a problem, because `*tor*` (instead of `Tor Browser`) also matched something like
/// `qbit*tor*rent`. The same thing with VSCode's `code`.
pub const BUILTIN_MAPPINGS: &[(&str, &str)] = &[
    // Browsers
    ("brave-browser", "si-brave-symbolic"),
    ("chromium-browser", "tb-brand-chrome-symbolic"),
    ("firefox", "si-firefox-symbolic"),
    ("floorp", "si-floorp-symbolic"),
    ("google-chrome", "si-googlechrome-symbolic"),
    ("librewolf", "si-librewolf-symbolic"),
    ("*microsoft-edge*", "tb-brand-edge-symbolic"),
    ("*opera*", "si-opera-symbolic"),
    ("*thorium*", "si-googlechrome-symbolic"),
    ("Tor Browser", "si-torbrowser-symbolic"),
    ("vivaldi-*", "si-vivaldi-symbolic"),
    ("waterfox", "si-firefox-symbolic"),
    ("zen-beta", "si-zenbrowser-symbolic"),
    ("zen-twilight", "si-zenbrowser-symbolic"),
    // Terminals
    ("Alacritty", "si-alacritty-symbolic"),
    ("foot", "ld-terminal-symbolic"),
    ("com.mitchellh.ghostty", "si-ghostty-symbolic"),
    ("*gnome-terminal*", "ld-terminal-symbolic"),
    ("kitty", "ld-terminal-symbolic"),
    ("org.kde.konsole", "ld-terminal-symbolic"),
    ("org.wezfurlong.wezterm", "si-wezterm-symbolic"),
    ("XTerm", "ld-terminal-symbolic"),
    // Code editors
    ("code", "tb-brand-vscode-symbolic"),
    ("codium", "tb-brand-vscode-symbolic"),
    ("Emacs", "si-gnuemacs-symbolic"),
    ("*jetbrains*", "si-jetbrains-symbolic"),
    ("neovide", "si-neovim-symbolic"),
    ("sublime_text", "si-sublimetext-symbolic"),
    ("dev.zed.Zed", "si-zedindustries-symbolic"),
    // Communication
    ("discord", "si-discord-symbolic"),
    ("*legcord*", "si-discord-symbolic"),
    ("vesktop", "si-discord-symbolic"),
    ("*webcord*", "si-discord-symbolic"),
    ("*element*", "si-element-symbolic"),
    ("signal", "si-signal-symbolic"),
    ("Slack", "ld-slack-symbolic"),
    ("org.telegram.desktop", "si-telegram-symbolic"),
    ("*thunderbird*", "si-thunderbird-symbolic"),
    // File managers
    ("org.kde.dolphin", "ld-folder-symbolic"),
    ("com.system76.CosmicFiles", "ld-folder-symbolic"),
    ("org.gnome.Nautilus", "ld-folder-symbolic"),
    ("nemo", "ld-folder-symbolic"),
    ("pcmanfm", "ld-folder-symbolic"),
    ("thunar", "ld-folder-symbolic"),
    // Media
    ("mpv", "si-mpv-symbolic"),
    ("spotify", "si-spotify-symbolic"),
    ("vlc", "si-vlcmediaplayer-symbolic"),
    // Graphics
    ("blender", "si-blender-symbolic"),
    ("gimp", "si-gimp-symbolic"),
    ("org.inkscape.Inkscape", "si-inkscape-symbolic"),
    ("krita", "si-krita-symbolic"),
    // Office
    ("libreoffice-base", "si-libreofficebase-symbolic"),
    ("libreoffice-calc", "si-libreofficecalc-symbolic"),
    ("libreoffice-draw", "si-libreofficedraw-symbolic"),
    ("libreoffice-math", "si-libreofficemath-symbolic"),
    ("libreoffice-writer", "si-libreofficewriter-symbolic"),
    ("libreoffice-impress", "si-libreofficeimpress-symbolic"),
    ("libreoffice*", "si-libreoffice-symbolic"),
    ("*onlyoffice*", "si-onlyoffice-symbolic"),
    ("*obsidian*", "si-obsidian-symbolic"),
    // Games
    ("steam", "si-steam-symbolic"),
    ("net.lutris.Lutris", "si-lutris-symbolic"),
    // Misc
    ("org.qbittorrent.qBittorrent", "si-qbittorrent-symbolic"),
];
