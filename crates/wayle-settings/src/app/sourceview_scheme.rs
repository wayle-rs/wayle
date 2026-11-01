//! Sourceview style scheme generation. Writes an XML scheme from the active
//! palette and registers its directory with the global `StyleSchemeManager`.

use std::{env, fs, path::PathBuf, sync::Mutex};

use tracing::warn;
use wayle_config::schemas::styling::PaletteConfig;

pub(crate) const SCHEME_ID: &str = "wayle";
const SCHEME_FILENAME: &str = "wayle.xml";

static SCHEME_DIR_REGISTERED: Mutex<bool> = Mutex::new(false);

fn scheme_dir() -> PathBuf {
    if let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir).join("wayle-sourceview");
    }

    let user = env::var("USER").unwrap_or_else(|_| String::from("unknown"));
    env::temp_dir().join(format!("wayle-sourceview-{user}"))
}

pub(crate) fn update_wayle_scheme(palette: &PaletteConfig) {
    let dir = scheme_dir();

    if let Err(err) = fs::create_dir_all(&dir) {
        warn!(error = %err, "failed to create scheme directory");
        return;
    }

    if let Err(err) = fs::write(dir.join(SCHEME_FILENAME), build_scheme_xml(palette)) {
        warn!(error = %err, "failed to write scheme file");
        return;
    }

    let manager = sourceview5::StyleSchemeManager::default();

    let mut registered = SCHEME_DIR_REGISTERED
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    if !*registered {
        let Some(dir_str) = dir.to_str() else {
            warn!(path = %dir.display(), "scheme directory path is not valid UTF-8");
            return;
        };
        manager.append_search_path(dir_str);
        *registered = true;
    }

    manager.force_rescan();
}

fn build_scheme_xml(palette: &PaletteConfig) -> String {
    let bg = palette.bg.get();
    let surface = palette.surface.get();
    let fg = palette.fg.get();
    let fg_muted = palette.fg_muted.get();
    let primary = palette.primary.get();
    let red = palette.red.get();
    let green = palette.green.get();
    let yellow = palette.yellow.get();
    let blue = palette.blue.get();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<style-scheme id="{SCHEME_ID}" name="Wayle" version="1.0">
  <style name="text" foreground="{fg}" background="{bg}"/>
  <style name="cursor" foreground="{primary}"/>
  <style name="selection" foreground="{fg}" background="{surface}"/>
  <style name="current-line" background="{surface}"/>
  <style name="line-numbers" foreground="{fg_muted}" background="{bg}"/>
  <style name="bracket-match" foreground="{primary}" bold="true"/>

  <style name="def:keyword" foreground="{blue}" bold="true"/>
  <style name="def:string" foreground="{green}"/>
  <style name="def:number" foreground="{primary}"/>
  <style name="def:boolean" foreground="{primary}"/>
  <style name="def:comment" foreground="{fg_muted}" italic="true"/>
  <style name="def:type" foreground="{yellow}"/>
  <style name="def:constant" foreground="{primary}"/>
  <style name="def:identifier" foreground="{fg}"/>
  <style name="def:special-char" foreground="{red}"/>
  <style name="def:heading" foreground="{blue}" bold="true"/>
</style-scheme>"#
    )
}
