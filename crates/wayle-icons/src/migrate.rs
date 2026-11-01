//! In-place migration of on-disk icons to the current SVG format.
//!
//! Older wayle releases shipped icons that used GTK's Grappa stroke
//! attributes (`gpa:stroke='foreground'`). GTK 4.21–4.23 mis-renders those
//! files (see <https://gitlab.gnome.org/GNOME/gtk/-/issues/8147>), so the
//! transform now emits filled-polygon geometry only. Existing icons on
//! upgraded user machines need to be rewritten.
//!
//! [`run`] walks the icons directory, rewrites stale files atomically, and
//! drops a sentinel so subsequent launches short-circuit without scanning.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use tracing::{error, info, instrument, warn};

use crate::transform::{self, FORMAT_VERSION};

const SENTINEL_FILENAME: &str = ".format-version";
const LEGACY_STROKE_MARKER: &str = "gpa:stroke=";
const VERSION_ATTR_PREFIX: &str = "gpa:version='";
const TEMP_NAME_INFIX: &str = ".svg.tmp.";
const WRITE_PROBE_INFIX: &str = ".write-probe.";

/// Aggregate counters returned from a migration pass.
#[derive(Debug, Default, Clone, Copy)]
pub struct MigrationReport {
    /// Number of files rewritten to the current format.
    pub migrated: usize,
    /// Number of files already in the current format.
    pub skipped: usize,
    /// Number of files that errored during scan, transform, or write.
    pub failed: usize,
}

/// Migrates every legacy-format SVG in `icons_dir` to the current format.
///
/// No-op when the directory does not exist, is not writable, or when the
/// sentinel already records a version at or above [`FORMAT_VERSION`].
/// Per-file errors are logged but never propagated — migration is best-effort
/// and must never block shell startup.
#[instrument(skip_all, fields(dir = %icons_dir.display()))]
pub fn run(icons_dir: &Path) -> MigrationReport {
    if !should_migrate(icons_dir) {
        return MigrationReport::default();
    }

    sweep_orphan_tempfiles(icons_dir);

    let report = scan_and_migrate(icons_dir);
    fsync_dir(icons_dir);
    finalize(icons_dir, &report);

    info!(
        migrated = report.migrated,
        skipped = report.skipped,
        failed = report.failed,
        "icon migration complete"
    );
    report
}

fn should_migrate(icons_dir: &Path) -> bool {
    if !icons_dir.exists() {
        info!("no icons directory; nothing to migrate");
        return false;
    }
    if up_to_date(icons_dir) {
        return false;
    }
    if !is_writable(icons_dir) {
        info!("icons directory not writable; skipping migration");
        return false;
    }
    true
}

fn finalize(icons_dir: &Path, report: &MigrationReport) {
    if report.failed > 0 {
        return;
    }
    if let Err(err) = write_sentinel(icons_dir) {
        warn!(error = %err, "cannot write migration sentinel; will retry next launch");
    }
}

/// Returns `true` when the sentinel records a version at or above the current.
///
/// The `>=` comparison preserves files written by a hypothetical future wayle
/// after a user downgrades: those files keep whatever format the future
/// version produced rather than being rewritten back to the current format.
/// The contract is that newer wayle versions never emit files that cannot be
/// rendered by older renderers; if that invariant ever changes, the future
/// version must introduce a separate minimum-supported-version stamp.
fn up_to_date(icons_dir: &Path) -> bool {
    let Ok(content) = fs::read_to_string(icons_dir.join(SENTINEL_FILENAME)) else {
        return false;
    };
    matches!(content.trim().parse::<u32>(), Ok(version) if version >= FORMAT_VERSION)
}

fn is_writable(icons_dir: &Path) -> bool {
    let probe_path = icons_dir.join(format!(
        "{WRITE_PROBE_INFIX}{}.{}",
        process::id(),
        next_nonce()
    ));
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)
    {
        Ok(_) => {
            let _ = fs::remove_file(&probe_path);
            true
        }
        Err(_) => false,
    }
}

fn write_sentinel(icons_dir: &Path) -> io::Result<()> {
    let sentinel = icons_dir.join(SENTINEL_FILENAME);
    write_atomically(&sentinel, format!("{FORMAT_VERSION}\n").as_bytes())
}

fn sweep_orphan_tempfiles(icons_dir: &Path) {
    let Ok(entries) = fs::read_dir(icons_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !filename.contains(TEMP_NAME_INFIX) && !filename.contains(WRITE_PROBE_INFIX) {
            continue;
        }
        if let Err(err) = fs::remove_file(&path) {
            warn!(path = %path.display(), error = %err, "cannot remove orphan tempfile");
        }
    }
}

fn scan_and_migrate(icons_dir: &Path) -> MigrationReport {
    let mut report = MigrationReport::default();

    let entries = match fs::read_dir(icons_dir) {
        Ok(entries) => entries,
        Err(err) => {
            error!(error = %err, "cannot read icons directory");
            report.failed = 1;
            return report;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_target_svg(&path) {
            continue;
        }
        match migrate_one(&path) {
            FileOutcome::Migrated => report.migrated += 1,
            FileOutcome::AlreadyCurrent => report.skipped += 1,
            FileOutcome::Failed => report.failed += 1,
        }
    }
    report
}

fn is_target_svg(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "svg")
}

enum FileOutcome {
    Migrated,
    AlreadyCurrent,
    Failed,
}

fn migrate_one(path: &Path) -> FileOutcome {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            warn!(path = %path.display(), error = %err, "cannot read icon; skipping");
            return FileOutcome::Failed;
        }
    };

    if !needs_migration(&content) {
        return FileOutcome::AlreadyCurrent;
    }

    let Some(transformed) = transform::to_symbolic(&content) else {
        warn!(
            path = %path.display(),
            "icon produced no extractable paths; original kept untouched"
        );
        return FileOutcome::Failed;
    };

    if let Err(err) = write_atomically(path, transformed.as_bytes()) {
        error!(path = %path.display(), error = %err, "cannot write migrated icon");
        return FileOutcome::Failed;
    }

    FileOutcome::Migrated
}

fn needs_migration(content: &str) -> bool {
    if content.contains(LEGACY_STROKE_MARKER) {
        return true;
    }
    file_format_version(content).is_none_or(|version| version < FORMAT_VERSION)
}

fn file_format_version(content: &str) -> Option<u32> {
    let start = content.find(VERSION_ATTR_PREFIX)? + VERSION_ATTR_PREFIX.len();
    let rest = &content[start..];
    let end = rest.find('\'')?;
    rest[..end].parse::<u32>().ok()
}

fn write_atomically(target: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| io::Error::other("target path has no parent directory"))?;
    let base = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("icon");
    let pid = process::id();
    let nonce = next_nonce();
    let tmp = parent.join(format!(".{base}{TEMP_NAME_INFIX}{pid}.{nonce}"));

    let write_result = (|| -> io::Result<()> {
        {
            let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
            file.write_all(bytes)?;
            file.sync_data()?;
        }
        fs::rename(&tmp, target)
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    write_result
}

fn next_nonce() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn fsync_dir(dir: &Path) {
    let Ok(file) = File::open(dir) else { return };
    let _ = file.sync_all();
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::{
        env,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let path = env::temp_dir().join(format!(
                "wayle-icons-test-{}-{}",
                process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("create test dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    const LEGACY_LUCIDE_WIFI: &str = "<svg width='16' height='16' \
        xmlns:gpa='https://www.gtk.org/grappa' gpa:version='1'>\
        <path d='M2 8L14 8' stroke-width='1.33' stroke-linecap='round' \
            stroke='rgb(0,0,0)' fill='none' gpa:stroke='foreground'/></svg>";

    #[test]
    fn older_sentinel_triggers_remigration() {
        let dir = TestDir::new();
        fs::write(dir.path().join(SENTINEL_FILENAME), "1\n").expect("write sentinel");
        fs::write(dir.path().join("ld-wifi-symbolic.svg"), LEGACY_LUCIDE_WIFI).expect("write svg");

        let report = run(dir.path());

        assert_eq!(report.migrated, 1);
        assert_eq!(report.failed, 0);
        let sentinel = fs::read_to_string(dir.path().join(SENTINEL_FILENAME))
            .expect("read sentinel after run");
        assert_eq!(sentinel.trim().parse::<u32>().ok(), Some(FORMAT_VERSION));
    }

    #[test]
    fn current_sentinel_short_circuits_scan() {
        let dir = TestDir::new();
        fs::write(
            dir.path().join(SENTINEL_FILENAME),
            format!("{FORMAT_VERSION}\n"),
        )
        .expect("write sentinel");
        let legacy_path = dir.path().join("ld-wifi-symbolic.svg");
        fs::write(&legacy_path, LEGACY_LUCIDE_WIFI).expect("write svg");

        let report = run(dir.path());

        assert_eq!(report.migrated, 0);
        assert_eq!(report.skipped, 0);
        assert_eq!(report.failed, 0);
        let after = fs::read_to_string(&legacy_path).expect("read svg after run");
        assert_eq!(after, LEGACY_LUCIDE_WIFI);
    }

    #[test]
    fn unparseable_legacy_file_blocks_sentinel_and_preserves_original() {
        let dir = TestDir::new();
        let unparseable = "<svg viewBox='0 0 16 16'><!-- gpa:stroke=foreground --></svg>";
        let target = dir.path().join("ld-broken-symbolic.svg");
        fs::write(&target, unparseable).expect("write svg");

        let report = run(dir.path());

        assert_eq!(report.failed, 1);
        assert!(!dir.path().join(SENTINEL_FILENAME).exists());
        let after = fs::read_to_string(&target).expect("read svg after run");
        assert_eq!(after, unparseable);
    }

    #[test]
    fn write_atomically_leaves_no_orphan_tempfiles() {
        let dir = TestDir::new();
        let target = dir.path().join("ld-wifi-symbolic.svg");

        write_atomically(&target, b"<svg></svg>").expect("atomic write");

        assert_eq!(
            fs::read_to_string(&target).expect("read target"),
            "<svg></svg>"
        );
        let orphans: Vec<_> = fs::read_dir(dir.path())
            .expect("read dir")
            .flatten()
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.contains(TEMP_NAME_INFIX))
            })
            .collect();
        assert!(orphans.is_empty(), "found orphan tempfiles");
    }

    #[test]
    fn run_is_idempotent() {
        let dir = TestDir::new();
        fs::write(dir.path().join("ld-wifi-symbolic.svg"), LEGACY_LUCIDE_WIFI).expect("write svg");

        let first = run(dir.path());
        assert_eq!(first.migrated, 1);

        let second = run(dir.path());
        assert_eq!(second.migrated, 0);
        assert_eq!(second.skipped, 0);
        assert_eq!(second.failed, 0);
    }
}
