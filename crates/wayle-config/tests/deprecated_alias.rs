//! Verifies `#[wayle(deprecated_alias)]` emits a runtime warning when matched.

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use tracing_subscriber::fmt::MakeWriter;
use wayle_config::{ApplyConfigLayer, CommitConfigReload, Config};

#[derive(Clone, Default)]
struct CapturedBuffer(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedBuffer {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut guard = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for CapturedBuffer {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[test]
fn deprecated_alias_emits_warning_with_canonical_replacement()
-> Result<(), Box<dyn std::error::Error>> {
    let writer = CapturedBuffer::default();
    let captured = Arc::clone(&writer.0);

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(tracing::Level::WARN)
        .with_ansi(false)
        .without_time()
        .finish();

    let toml_input: toml::Value = toml::from_str(
        r#"
        [modules.notification]
        icon-name = "x"
        "#,
    )?;

    tracing::subscriber::with_default(subscriber, || {
        let config = Config::default();
        config.apply_config_layer(&toml_input, "");
        config.commit_config_reload();
    });

    let bytes = match captured.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let log = String::from_utf8(bytes)?;

    assert!(
        log.contains("deprecated config key"),
        "expected deprecation message, got: {log}"
    );
    assert!(
        log.contains("deprecated=modules.notification "),
        "expected deprecated=modules.notification field, got: {log}"
    );
    assert!(
        log.contains("canonical=modules.notifications"),
        "expected canonical=modules.notifications field, got: {log}"
    );

    Ok(())
}
