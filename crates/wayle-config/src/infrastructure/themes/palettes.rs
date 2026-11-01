use super::Palette;
use crate::schemas::styling::ThemeEntry;

/// Wayle default theme color constants.
#[allow(missing_docs)]
pub mod wayle_theme {
    pub const BG: &str = "#141420";
    pub const SURFACE: &str = "#1c1c2c";
    pub const ELEVATED: &str = "#262638";
    pub const FG: &str = "#d4d6e8";
    pub const FG_MUTED: &str = "#8a8ca4";
    pub const PRIMARY: &str = "#e0947a";
    pub const RED: &str = "#e46870";
    pub const YELLOW: &str = "#e0b870";
    pub const GREEN: &str = "#68c898";
    pub const BLUE: &str = "#78a0e0";
}

type PaletteFn = fn() -> Palette;

/// All built-in theme entries.
pub fn builtins() -> Vec<ThemeEntry> {
    const ENTRIES: &[(&str, PaletteFn)] = &[
        ("wayle", wayle),
        ("catppuccin-mocha", catppuccin),
        ("catppuccin-macchiato", catppuccin_macchiato),
        ("catppuccin-frappe", catppuccin_frappe),
        ("catppuccin-latte", catppuccin_latte),
        ("dracula", dracula),
        ("everforest-dark", everforest),
        ("everforest-dark-hard", everforest_dark_hard),
        ("everforest-dark-soft", everforest_dark_soft),
        ("everforest-light", everforest_light),
        ("everforest-light-hard", everforest_light_hard),
        ("everforest-light-soft", everforest_light_soft),
        ("gruvbox-dark", gruvbox),
        ("gruvbox-dark-hard", gruvbox_dark_hard),
        ("gruvbox-dark-soft", gruvbox_dark_soft),
        ("gruvbox-light", gruvbox_light),
        ("gruvbox-light-hard", gruvbox_light_hard),
        ("gruvbox-light-soft", gruvbox_light_soft),
        ("kanagawa-wave", kanagawa_wave),
        ("kanagawa-dragon", kanagawa_dragon),
        ("kanagawa-lotus", kanagawa_lotus),
        ("monokai", monokai),
        ("monokai-pro-classic", monokai_pro_classic),
        ("monokai-pro-octagon", monokai_pro_octagon),
        ("monokai-pro-machine", monokai_pro_machine),
        ("monokai-pro-ristretto", monokai_pro_ristretto),
        ("monokai-pro-spectrum", monokai_pro_spectrum),
        ("nightfox-carbonfox", nightfox_carbonfox),
        ("nightfox-nightfox", nightfox_nightfox),
        ("nightfox-duskfox", nightfox_duskfox),
        ("nightfox-nordfox", nightfox_nordfox),
        ("nightfox-terafox", nightfox_terafox),
        ("nightfox-dayfox", nightfox_dayfox),
        ("nord", nord),
        ("one-dark", one_dark),
        ("one-light", one_light),
        ("rose-pine-main", rose_pine),
        ("rose-pine-moon", rose_pine_moon),
        ("rose-pine-dawn", rose_pine_dawn),
        ("solarized-dark", solarized_dark),
        ("solarized-light", solarized_light),
        ("tokyo-night-night", tokyo_night),
        ("tokyo-night-storm", tokyo_night_storm),
        ("tokyo-night-moon", tokyo_night_moon),
        ("tokyo-night-day", tokyo_night_day),
    ];

    ENTRIES
        .iter()
        .map(|(name, palette_fn)| ThemeEntry {
            name: String::from(*name),
            palette: palette_fn(),
            builtin: true,
        })
        .collect()
}

/// Default palette
pub fn wayle() -> Palette {
    use wayle_theme::*;
    Palette {
        bg: BG.to_owned(),
        surface: SURFACE.to_owned(),
        elevated: ELEVATED.to_owned(),
        fg: FG.to_owned(),
        fg_muted: FG_MUTED.to_owned(),
        primary: PRIMARY.to_owned(),
        red: RED.to_owned(),
        yellow: YELLOW.to_owned(),
        green: GREEN.to_owned(),
        blue: BLUE.to_owned(),
    }
}

fn catppuccin() -> Palette {
    Palette {
        bg: String::from("#11111b"),
        surface: String::from("#181825"),
        elevated: String::from("#1e1e2e"),
        fg: String::from("#cdd6f4"),
        fg_muted: String::from("#bac2de"),
        primary: String::from("#b4befe"),
        red: String::from("#f38ba8"),
        yellow: String::from("#f9e2af"),
        green: String::from("#a6e3a1"),
        blue: String::from("#74c7ec"),
    }
}

fn catppuccin_latte() -> Palette {
    Palette {
        bg: String::from("#eff1f5"),
        surface: String::from("#e6e9ef"),
        elevated: String::from("#dce0e8"),
        fg: String::from("#4c4f69"),
        fg_muted: String::from("#5c5f77"),
        primary: String::from("#7287fd"),
        red: String::from("#d20f39"),
        yellow: String::from("#df8e1d"),
        green: String::from("#40a02b"),
        blue: String::from("#1e66f5"),
    }
}

fn gruvbox() -> Palette {
    Palette {
        bg: String::from("#282828"),
        surface: String::from("#3c3836"),
        elevated: String::from("#504945"),
        fg: String::from("#ebdbb2"),
        fg_muted: String::from("#d5c4a1"),
        primary: String::from("#83a598"),
        red: String::from("#fb4934"),
        yellow: String::from("#fabd2f"),
        green: String::from("#b8bb26"),
        blue: String::from("#8ec07c"),
    }
}

fn tokyo_night() -> Palette {
    Palette {
        bg: String::from("#16161e"),
        surface: String::from("#1a1b26"),
        elevated: String::from("#292e42"),
        fg: String::from("#c0caf5"),
        fg_muted: String::from("#a9b1d6"),
        primary: String::from("#7aa2f7"),
        red: String::from("#f7768e"),
        yellow: String::from("#e0af68"),
        green: String::from("#9ece6a"),
        blue: String::from("#7dcfff"),
    }
}

fn rose_pine() -> Palette {
    Palette {
        bg: String::from("#191724"),
        surface: String::from("#1f1d2e"),
        elevated: String::from("#26233a"),
        fg: String::from("#e0def4"),
        fg_muted: String::from("#908caa"),
        primary: String::from("#c4a7e7"),
        red: String::from("#eb6f92"),
        yellow: String::from("#f6c177"),
        green: String::from("#31748f"),
        blue: String::from("#9ccfd8"),
    }
}

fn dracula() -> Palette {
    Palette {
        bg: String::from("#282a36"),
        surface: String::from("#343746"),
        elevated: String::from("#44475a"),
        fg: String::from("#f8f8f2"),
        fg_muted: String::from("#6272a4"),
        primary: String::from("#bd93f9"),
        red: String::from("#ff5555"),
        yellow: String::from("#f1fa8c"),
        green: String::from("#50fa7b"),
        blue: String::from("#8be9fd"),
    }
}

fn nord() -> Palette {
    Palette {
        bg: String::from("#2e3440"),
        surface: String::from("#3b4252"),
        elevated: String::from("#434c5e"),
        fg: String::from("#eceff4"),
        fg_muted: String::from("#d8dee9"),
        primary: String::from("#88c0d0"),
        red: String::from("#bf616a"),
        yellow: String::from("#ebcb8b"),
        green: String::from("#a3be8c"),
        blue: String::from("#81a1c1"),
    }
}

fn everforest() -> Palette {
    Palette {
        bg: String::from("#2d353b"),
        surface: String::from("#343f44"),
        elevated: String::from("#3d484d"),
        fg: String::from("#d3c6aa"),
        fg_muted: String::from("#859289"),
        primary: String::from("#7fbbb3"),
        red: String::from("#e67e80"),
        yellow: String::from("#dbbc7f"),
        green: String::from("#a7c080"),
        blue: String::from("#83c092"),
    }
}

fn catppuccin_frappe() -> Palette {
    Palette {
        bg: String::from("#232634"),
        surface: String::from("#292c3c"),
        elevated: String::from("#303446"),
        fg: String::from("#c6d0f5"),
        fg_muted: String::from("#b5bfe2"),
        primary: String::from("#babbf1"),
        red: String::from("#e78284"),
        yellow: String::from("#e5c890"),
        green: String::from("#a6d189"),
        blue: String::from("#8caaee"),
    }
}

fn catppuccin_macchiato() -> Palette {
    Palette {
        bg: String::from("#181926"),
        surface: String::from("#1e2030"),
        elevated: String::from("#24273a"),
        fg: String::from("#cad3f5"),
        fg_muted: String::from("#b8c0e0"),
        primary: String::from("#b7bdf8"),
        red: String::from("#ed8796"),
        yellow: String::from("#eed49f"),
        green: String::from("#a6da95"),
        blue: String::from("#8aadf4"),
    }
}

fn gruvbox_dark_hard() -> Palette {
    Palette {
        bg: String::from("#1d2021"),
        surface: String::from("#282828"),
        elevated: String::from("#3c3836"),
        fg: String::from("#ebdbb2"),
        fg_muted: String::from("#d5c4a1"),
        primary: String::from("#83a598"),
        red: String::from("#fb4934"),
        yellow: String::from("#fabd2f"),
        green: String::from("#b8bb26"),
        blue: String::from("#8ec07c"),
    }
}

fn gruvbox_dark_soft() -> Palette {
    Palette {
        bg: String::from("#32302f"),
        surface: String::from("#3c3836"),
        elevated: String::from("#504945"),
        fg: String::from("#ebdbb2"),
        fg_muted: String::from("#d5c4a1"),
        primary: String::from("#83a598"),
        red: String::from("#fb4934"),
        yellow: String::from("#fabd2f"),
        green: String::from("#b8bb26"),
        blue: String::from("#8ec07c"),
    }
}

fn gruvbox_light() -> Palette {
    Palette {
        bg: String::from("#fbf1c7"),
        surface: String::from("#ebdbb2"),
        elevated: String::from("#d5c4a1"),
        fg: String::from("#3c3836"),
        fg_muted: String::from("#504945"),
        primary: String::from("#076678"),
        red: String::from("#9d0006"),
        yellow: String::from("#b57614"),
        green: String::from("#79740e"),
        blue: String::from("#427b58"),
    }
}

fn gruvbox_light_hard() -> Palette {
    Palette {
        bg: String::from("#f9f5d7"),
        surface: String::from("#fbf1c7"),
        elevated: String::from("#ebdbb2"),
        fg: String::from("#3c3836"),
        fg_muted: String::from("#504945"),
        primary: String::from("#076678"),
        red: String::from("#9d0006"),
        yellow: String::from("#b57614"),
        green: String::from("#79740e"),
        blue: String::from("#427b58"),
    }
}

fn gruvbox_light_soft() -> Palette {
    Palette {
        bg: String::from("#f2e5bc"),
        surface: String::from("#ebdbb2"),
        elevated: String::from("#d5c4a1"),
        fg: String::from("#3c3836"),
        fg_muted: String::from("#504945"),
        primary: String::from("#076678"),
        red: String::from("#9d0006"),
        yellow: String::from("#b57614"),
        green: String::from("#79740e"),
        blue: String::from("#427b58"),
    }
}

fn tokyo_night_storm() -> Palette {
    Palette {
        bg: String::from("#1f2335"),
        surface: String::from("#24283b"),
        elevated: String::from("#292e42"),
        fg: String::from("#c0caf5"),
        fg_muted: String::from("#a9b1d6"),
        primary: String::from("#7aa2f7"),
        red: String::from("#f7768e"),
        yellow: String::from("#e0af68"),
        green: String::from("#9ece6a"),
        blue: String::from("#7dcfff"),
    }
}

fn tokyo_night_moon() -> Palette {
    Palette {
        bg: String::from("#1e2030"),
        surface: String::from("#222436"),
        elevated: String::from("#2f334d"),
        fg: String::from("#c8d3f5"),
        fg_muted: String::from("#828bb8"),
        primary: String::from("#82aaff"),
        red: String::from("#ff757f"),
        yellow: String::from("#ffc777"),
        green: String::from("#c3e88d"),
        blue: String::from("#86e1fc"),
    }
}

fn tokyo_night_day() -> Palette {
    Palette {
        bg: String::from("#e1e2e7"),
        surface: String::from("#d0d5e3"),
        elevated: String::from("#c4c8da"),
        fg: String::from("#3760bf"),
        fg_muted: String::from("#6172b0"),
        primary: String::from("#2e7de9"),
        red: String::from("#f52a65"),
        yellow: String::from("#8c6c3e"),
        green: String::from("#587539"),
        blue: String::from("#007197"),
    }
}

fn rose_pine_moon() -> Palette {
    Palette {
        bg: String::from("#232136"),
        surface: String::from("#2a273f"),
        elevated: String::from("#393552"),
        fg: String::from("#e0def4"),
        fg_muted: String::from("#908caa"),
        primary: String::from("#c4a7e7"),
        red: String::from("#eb6f92"),
        yellow: String::from("#f6c177"),
        green: String::from("#3e8fb0"),
        blue: String::from("#9ccfd8"),
    }
}

fn rose_pine_dawn() -> Palette {
    Palette {
        bg: String::from("#fffaf3"),
        surface: String::from("#faf4ed"),
        elevated: String::from("#f2e9e1"),
        fg: String::from("#464261"),
        fg_muted: String::from("#797593"),
        primary: String::from("#907aa9"),
        red: String::from("#b4637a"),
        yellow: String::from("#ea9d34"),
        green: String::from("#286983"),
        blue: String::from("#56949f"),
    }
}

fn everforest_dark_hard() -> Palette {
    Palette {
        bg: String::from("#272e33"),
        surface: String::from("#2d353b"),
        elevated: String::from("#343f44"),
        fg: String::from("#d3c6aa"),
        fg_muted: String::from("#859289"),
        primary: String::from("#7fbbb3"),
        red: String::from("#e67e80"),
        yellow: String::from("#dbbc7f"),
        green: String::from("#a7c080"),
        blue: String::from("#83c092"),
    }
}

fn everforest_dark_soft() -> Palette {
    Palette {
        bg: String::from("#333c43"),
        surface: String::from("#3a464c"),
        elevated: String::from("#4d5960"),
        fg: String::from("#d3c6aa"),
        fg_muted: String::from("#859289"),
        primary: String::from("#7fbbb3"),
        red: String::from("#e67e80"),
        yellow: String::from("#dbbc7f"),
        green: String::from("#a7c080"),
        blue: String::from("#83c092"),
    }
}

fn everforest_light() -> Palette {
    Palette {
        bg: String::from("#fdf6e3"),
        surface: String::from("#f4f0d9"),
        elevated: String::from("#e6e2cc"),
        fg: String::from("#5c6a72"),
        fg_muted: String::from("#939f91"),
        primary: String::from("#3a94c5"),
        red: String::from("#f85552"),
        yellow: String::from("#dfa000"),
        green: String::from("#8da101"),
        blue: String::from("#35a77c"),
    }
}

fn everforest_light_hard() -> Palette {
    Palette {
        bg: String::from("#fffbef"),
        surface: String::from("#fdf6e3"),
        elevated: String::from("#f4f0d9"),
        fg: String::from("#5c6a72"),
        fg_muted: String::from("#939f91"),
        primary: String::from("#3a94c5"),
        red: String::from("#f85552"),
        yellow: String::from("#dfa000"),
        green: String::from("#8da101"),
        blue: String::from("#35a77c"),
    }
}

fn everforest_light_soft() -> Palette {
    Palette {
        bg: String::from("#f3ead3"),
        surface: String::from("#eae4ca"),
        elevated: String::from("#ddd8be"),
        fg: String::from("#5c6a72"),
        fg_muted: String::from("#939f91"),
        primary: String::from("#3a94c5"),
        red: String::from("#f85552"),
        yellow: String::from("#dfa000"),
        green: String::from("#8da101"),
        blue: String::from("#35a77c"),
    }
}

fn kanagawa_wave() -> Palette {
    Palette {
        bg: String::from("#1a1a22"),
        surface: String::from("#1f1f28"),
        elevated: String::from("#2a2a37"),
        fg: String::from("#dcd7ba"),
        fg_muted: String::from("#727169"),
        primary: String::from("#957fb8"),
        red: String::from("#e82424"),
        yellow: String::from("#dca561"),
        green: String::from("#98bb6c"),
        blue: String::from("#7e9cd8"),
    }
}

fn kanagawa_dragon() -> Palette {
    Palette {
        bg: String::from("#0d0c0c"),
        surface: String::from("#181616"),
        elevated: String::from("#282727"),
        fg: String::from("#c5c9c5"),
        fg_muted: String::from("#a6a69c"),
        primary: String::from("#8992a7"),
        red: String::from("#c4746e"),
        yellow: String::from("#c4b28a"),
        green: String::from("#87a987"),
        blue: String::from("#8ba4b0"),
    }
}

fn kanagawa_lotus() -> Palette {
    Palette {
        bg: String::from("#f2ecbc"),
        surface: String::from("#e5ddb0"),
        elevated: String::from("#e7dba0"),
        fg: String::from("#545464"),
        fg_muted: String::from("#716e61"),
        primary: String::from("#766b90"),
        red: String::from("#c84053"),
        yellow: String::from("#de9800"),
        green: String::from("#6f894e"),
        blue: String::from("#4d699b"),
    }
}

fn monokai() -> Palette {
    Palette {
        bg: String::from("#1e1f1c"),
        surface: String::from("#272822"),
        elevated: String::from("#3e3d32"),
        fg: String::from("#f8f8f2"),
        fg_muted: String::from("#75715e"),
        primary: String::from("#ae81ff"),
        red: String::from("#f92672"),
        yellow: String::from("#e6db74"),
        green: String::from("#a6e22e"),
        blue: String::from("#66d9ef"),
    }
}

fn monokai_pro_classic() -> Palette {
    Palette {
        bg: String::from("#221f22"),
        surface: String::from("#2d2a2e"),
        elevated: String::from("#403e41"),
        fg: String::from("#fcfcfa"),
        fg_muted: String::from("#727072"),
        primary: String::from("#ab9df2"),
        red: String::from("#ff6188"),
        yellow: String::from("#ffd866"),
        green: String::from("#a9dc76"),
        blue: String::from("#78dce8"),
    }
}

fn monokai_pro_octagon() -> Palette {
    Palette {
        bg: String::from("#1d1f28"),
        surface: String::from("#282a3a"),
        elevated: String::from("#3a3d4b"),
        fg: String::from("#eaf2f1"),
        fg_muted: String::from("#696d77"),
        primary: String::from("#c39ac9"),
        red: String::from("#ff657a"),
        yellow: String::from("#ffd76d"),
        green: String::from("#bad761"),
        blue: String::from("#9cd1bb"),
    }
}

fn monokai_pro_machine() -> Palette {
    Palette {
        bg: String::from("#1d2528"),
        surface: String::from("#273136"),
        elevated: String::from("#363c42"),
        fg: String::from("#f2fffc"),
        fg_muted: String::from("#6b7678"),
        primary: String::from("#baa0f8"),
        red: String::from("#ff6d7e"),
        yellow: String::from("#ffed72"),
        green: String::from("#a2e57b"),
        blue: String::from("#7cd5f1"),
    }
}

fn monokai_pro_ristretto() -> Palette {
    Palette {
        bg: String::from("#211c1c"),
        surface: String::from("#2c2525"),
        elevated: String::from("#403838"),
        fg: String::from("#fff1f3"),
        fg_muted: String::from("#72696a"),
        primary: String::from("#a8a9eb"),
        red: String::from("#fd6883"),
        yellow: String::from("#f9cc6c"),
        green: String::from("#adda78"),
        blue: String::from("#85dacc"),
    }
}

fn monokai_pro_spectrum() -> Palette {
    Palette {
        bg: String::from("#191919"),
        surface: String::from("#222222"),
        elevated: String::from("#363537"),
        fg: String::from("#f7f1ff"),
        fg_muted: String::from("#69676c"),
        primary: String::from("#948ae3"),
        red: String::from("#fc618d"),
        yellow: String::from("#fce566"),
        green: String::from("#7bd88f"),
        blue: String::from("#5ad4e6"),
    }
}

fn nightfox_carbonfox() -> Palette {
    Palette {
        bg: String::from("#0a0a0a"),
        surface: String::from("#161616"),
        elevated: String::from("#1f1f1f"),
        fg: String::from("#f2f4f8"),
        fg_muted: String::from("#a8aab1"),
        primary: String::from("#78a9ff"),
        red: String::from("#ee5396"),
        yellow: String::from("#08bdba"),
        green: String::from("#25be6a"),
        blue: String::from("#33b1ff"),
    }
}

fn nightfox_nightfox() -> Palette {
    Palette {
        bg: String::from("#131a24"),
        surface: String::from("#192330"),
        elevated: String::from("#212e3f"),
        fg: String::from("#cdcecf"),
        fg_muted: String::from("#71839b"),
        primary: String::from("#719cd6"),
        red: String::from("#c94f6d"),
        yellow: String::from("#dbc074"),
        green: String::from("#81b29a"),
        blue: String::from("#63cdcf"),
    }
}

fn nightfox_duskfox() -> Palette {
    Palette {
        bg: String::from("#191726"),
        surface: String::from("#232136"),
        elevated: String::from("#2d2a45"),
        fg: String::from("#e0def4"),
        fg_muted: String::from("#6e6a86"),
        primary: String::from("#c4a7e7"),
        red: String::from("#eb6f92"),
        yellow: String::from("#f6c177"),
        green: String::from("#a3be8c"),
        blue: String::from("#9ccfd8"),
    }
}

fn nightfox_nordfox() -> Palette {
    Palette {
        bg: String::from("#232831"),
        surface: String::from("#2e3440"),
        elevated: String::from("#39404f"),
        fg: String::from("#cdcecf"),
        fg_muted: String::from("#7e8188"),
        primary: String::from("#88c0d0"),
        red: String::from("#bf616a"),
        yellow: String::from("#ebcb8b"),
        green: String::from("#a3be8c"),
        blue: String::from("#81a1c1"),
    }
}

fn nightfox_terafox() -> Palette {
    Palette {
        bg: String::from("#0f1c1e"),
        surface: String::from("#152528"),
        elevated: String::from("#1d3337"),
        fg: String::from("#e6eaea"),
        fg_muted: String::from("#587b7b"),
        primary: String::from("#5a93aa"),
        red: String::from("#e85c51"),
        yellow: String::from("#fda47f"),
        green: String::from("#7aa4a1"),
        blue: String::from("#a1cdd8"),
    }
}

fn nightfox_dayfox() -> Palette {
    Palette {
        bg: String::from("#f6f2ee"),
        surface: String::from("#e4dcd4"),
        elevated: String::from("#d3c7bb"),
        fg: String::from("#3d2b5a"),
        fg_muted: String::from("#824d5b"),
        primary: String::from("#2848a9"),
        red: String::from("#a5222f"),
        yellow: String::from("#ac5402"),
        green: String::from("#396847"),
        blue: String::from("#287980"),
    }
}

fn one_dark() -> Palette {
    Palette {
        bg: String::from("#21252b"),
        surface: String::from("#282c34"),
        elevated: String::from("#3e4451"),
        fg: String::from("#abb2bf"),
        fg_muted: String::from("#5c6370"),
        primary: String::from("#61afef"),
        red: String::from("#e06c75"),
        yellow: String::from("#e5c07b"),
        green: String::from("#98c379"),
        blue: String::from("#56b6c2"),
    }
}

fn one_light() -> Palette {
    Palette {
        bg: String::from("#fafafa"),
        surface: String::from("#ebebec"),
        elevated: String::from("#e5e5e6"),
        fg: String::from("#383a42"),
        fg_muted: String::from("#a0a1a7"),
        primary: String::from("#4078f2"),
        red: String::from("#e45649"),
        yellow: String::from("#c18401"),
        green: String::from("#50a14f"),
        blue: String::from("#0184bc"),
    }
}

fn solarized_dark() -> Palette {
    Palette {
        bg: String::from("#002b36"),
        surface: String::from("#073642"),
        elevated: String::from("#094553"),
        fg: String::from("#839496"),
        fg_muted: String::from("#586e75"),
        primary: String::from("#268bd2"),
        red: String::from("#dc322f"),
        yellow: String::from("#b58900"),
        green: String::from("#859900"),
        blue: String::from("#2aa198"),
    }
}

fn solarized_light() -> Palette {
    Palette {
        bg: String::from("#fdf6e3"),
        surface: String::from("#eee8d5"),
        elevated: String::from("#e0d9c4"),
        fg: String::from("#657b83"),
        fg_muted: String::from("#93a1a1"),
        primary: String::from("#268bd2"),
        red: String::from("#dc322f"),
        yellow: String::from("#b58900"),
        green: String::from("#859900"),
        blue: String::from("#2aa198"),
    }
}
