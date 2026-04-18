//! Window class icon mappings for the window title module.

/// Built-in window class-to-icon mappings.
///
/// Patterns use glob syntax and match against window class names.
/// Order matters - first match wins.
pub const BUILTIN_MAPPINGS: &[(&str, &str)] = &[
    // Browsers
    ("*brave*", "si-brave-symbolic"),
    ("*chromium*", "tb-brand-chrome-symbolic"),
    ("*firefox*", "si-firefox-symbolic"),
    ("*floorp*", "si-floorp-symbolic"),
    ("*google-chrome*", "si-googlechrome-symbolic"),
    ("*librewolf*", "si-librewolf-symbolic"),
    ("*microsoft-edge*", "tb-brand-edge-symbolic"),
    ("*opera*", "si-opera-symbolic"),
    ("*thorium*", "si-googlechrome-symbolic"),
    ("*tor*", "si-torbrowser-symbolic"),
    ("*vivaldi*", "si-vivaldi-symbolic"),
    ("*waterfox*", "si-firefox-symbolic"),
    ("*zen*", "si-zenbrowser-symbolic"),
    // Terminals
    ("*alacritty*", "si-alacritty-symbolic"),
    ("*foot*", "ld-terminal-symbolic"),
    ("*ghostty*", "si-ghostty-symbolic"),
    ("*gnome-terminal*", "ld-terminal-symbolic"),
    ("*kitty*", "ld-terminal-symbolic"),
    ("*konsole*", "ld-terminal-symbolic"),
    ("*wezterm*", "si-wezterm-symbolic"),
    ("*xterm*", "ld-terminal-symbolic"),
    // Code editors
    ("*code*", "tb-brand-vscode-symbolic"),
    ("*emacs*", "si-gnuemacs-symbolic"),
    ("*jetbrains*", "si-jetbrains-symbolic"),
    ("*neovide*", "si-neovim-symbolic"),
    ("*sublime*", "si-sublimetext-symbolic"),
    ("*vim*", "si-vim-symbolic"),
    ("*zed*", "si-zedindustries-symbolic"),
    // Communication
    ("*discord*", "si-discord-symbolic"),
    ("*legcord*", "si-discord-symbolic"),
    ("*vesktop*", "si-discord-symbolic"),
    ("*webcord*", "si-discord-symbolic"),
    ("*element*", "si-element-symbolic"),
    ("*signal*", "si-signal-symbolic"),
    ("*slack*", "ld-slack-symbolic"),
    ("*telegram*", "si-telegram-symbolic"),
    ("*thunderbird*", "si-thunderbird-symbolic"),
    // File managers
    ("*dolphin*", "ld-folder-symbolic"),
    ("*nautilus*", "ld-folder-symbolic"),
    ("*nemo*", "ld-folder-symbolic"),
    ("*pcmanfm*", "ld-folder-symbolic"),
    ("*thunar*", "ld-folder-symbolic"),
    // Media
    ("*mpv*", "si-mpv-symbolic"),
    ("*spotify*", "si-spotify-symbolic"),
    ("*vlc*", "si-vlcmediaplayer-symbolic"),
    // Graphics
    ("*blender*", "si-blender-symbolic"),
    ("*gimp*", "si-gimp-symbolic"),
    ("*inkscape*", "si-inkscape-symbolic"),
    ("*krita*", "si-krita-symbolic"),
    // Office
    ("libreoffice-base", "si-libreofficebase-symbolic"),
    ("libreoffice-calc", "si-libreofficecalc-symbolic"),
    ("libreoffice-draw", "si-libreofficedraw-symbolic"),
    ("libreoffice-math", "si-libreofficemath-symbolic"),
    ("libreoffice-writer", "si-libreofficewriter-symbolic"),
    ("libreoffice-impress", "si-libreofficeimpress-symbolic"),
    ("libreoffice*", "si-libreoffice-symbolic"),
    ("*obsidian*", "si-obsidian-symbolic"),
    // Games
    ("*steam*", "si-steam-symbolic"),
    ("*lutris*", "si-lutris-symbolic"),
    // Misc
    ("*qbittorrent*", "si-qbittorrent-symbolic"),
];
