use std::path::Path;

use bytesize::ByteSize;
use serde_json::json;
use wayle_config::schemas::modules::StorageMountPoint;
use wayle_sysinfo::types::DiskData;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct StorageSnapshot {
    pub usage_percent: f32,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub filesystem: String,
}

pub(super) fn aggregate_storage(
    disks: &[DiskData],
    mount_points: &StorageMountPoint,
) -> Option<StorageSnapshot> {
    let targets = mount_points.paths();
    if targets.is_empty() {
        return None;
    }

    let matched: Vec<&DiskData> = targets
        .iter()
        .filter_map(|target| {
            let target_path = Path::new(target);
            disks.iter().find(|disk| disk.mount_point == target_path)
        })
        .collect();

    if matched.is_empty() {
        return None;
    }

    let usage_percent =
        matched.iter().map(|disk| disk.usage_percent).sum::<f32>() / matched.len() as f32;
    let used_bytes = matched.iter().map(|disk| disk.used_bytes).sum::<u64>();
    let total_bytes = matched.iter().map(|disk| disk.total_bytes).sum::<u64>();
    let available_bytes = matched.iter().map(|disk| disk.available_bytes).sum::<u64>();
    let filesystem = if matched.len() == 1 {
        matched[0].filesystem.clone()
    } else {
        String::from("multiple")
    };

    Some(StorageSnapshot {
        usage_percent,
        used_bytes,
        total_bytes,
        available_bytes,
        filesystem,
    })
}

pub(super) fn format_label(format: &str, snapshot: &StorageSnapshot) -> String {
    let ctx = json!({
        "percent": format!("{:02.0}", snapshot.usage_percent),
        "used_tib": tib(snapshot.used_bytes),
        "used_gib": gib(snapshot.used_bytes),
        "used_mib": mib(snapshot.used_bytes),
        "used_auto": auto(snapshot.used_bytes),
        "total_tib": tib(snapshot.total_bytes),
        "total_gib": gib(snapshot.total_bytes),
        "total_mib": mib(snapshot.total_bytes),
        "total_auto": auto(snapshot.total_bytes),
        "free_tib": tib(snapshot.available_bytes),
        "free_gib": gib(snapshot.available_bytes),
        "free_mib": mib(snapshot.available_bytes),
        "free_auto": auto(snapshot.available_bytes),
        "filesystem": &snapshot.filesystem,
    });
    crate::template::render(format, ctx).unwrap_or_default()
}

fn tib(bytes: u64) -> String {
    format!("{:.2}", ByteSize::b(bytes).as_tib())
}

fn gib(bytes: u64) -> String {
    format!("{:.1}", ByteSize::b(bytes).as_gib())
}

fn mib(bytes: u64) -> String {
    format!("{:.0}", ByteSize::b(bytes).as_mib())
}

fn auto(bytes: u64) -> String {
    ByteSize::b(bytes).to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    const TIB: u64 = 1024 * 1024 * 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;
    const MIB: u64 = 1024 * 1024;

    fn disk_data(
        mount_point: &str,
        used: u64,
        total: u64,
        available: u64,
        usage_percent: f32,
        fs: &str,
    ) -> DiskData {
        DiskData {
            mount_point: PathBuf::from(mount_point),
            filesystem: fs.to_string(),
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percent,
        }
    }

    fn storage_snapshot(
        used: u64,
        total: u64,
        available: u64,
        usage_percent: f32,
        fs: &str,
    ) -> StorageSnapshot {
        StorageSnapshot {
            usage_percent,
            used_bytes: used,
            total_bytes: total,
            available_bytes: available,
            filesystem: fs.to_string(),
        }
    }

    #[test]
    fn aggregate_storage_single_mount_point_returns_that_disk() {
        let disks = vec![disk_data(
            "/",
            500 * GIB,
            1000 * GIB,
            500 * GIB,
            50.0,
            "ext4",
        )];

        let snapshot = aggregate_storage(&disks, &StorageMountPoint::Single(String::from("/")))
            .expect("expected matching disk");

        assert_eq!(snapshot.usage_percent, 50.0);
        assert_eq!(snapshot.used_bytes, 500 * GIB);
        assert_eq!(snapshot.total_bytes, 1000 * GIB);
        assert_eq!(snapshot.available_bytes, 500 * GIB);
        assert_eq!(snapshot.filesystem, "ext4");
    }

    #[test]
    fn aggregate_storage_multiple_mount_points_uses_average_percent() {
        let disks = vec![
            disk_data("/", 500 * GIB, 1000 * GIB, 500 * GIB, 50.0, "ext4"),
            disk_data("/mnt/drive1", 400 * GIB, 800 * GIB, 400 * GIB, 80.0, "btrfs"),
        ];

        let snapshot = aggregate_storage(
            &disks,
            &StorageMountPoint::Multiple(vec![String::from("/"), String::from("/mnt/drive1")]),
        )
        .expect("expected matching disks");

        assert_eq!(snapshot.usage_percent, 65.0);
        assert_eq!(snapshot.used_bytes, 900 * GIB);
        assert_eq!(snapshot.total_bytes, 1800 * GIB);
        assert_eq!(snapshot.available_bytes, 900 * GIB);
        assert_eq!(snapshot.filesystem, "multiple");
    }

    #[test]
    fn aggregate_storage_ignores_unknown_mount_points() {
        let disks = vec![disk_data(
            "/",
            500 * GIB,
            1000 * GIB,
            500 * GIB,
            50.0,
            "ext4",
        )];

        let snapshot = aggregate_storage(
            &disks,
            &StorageMountPoint::Multiple(vec![String::from("/unknown"), String::from("/")]),
        )
        .expect("expected one matching disk");

        assert_eq!(snapshot.usage_percent, 50.0);
        assert_eq!(snapshot.filesystem, "ext4");
    }

    #[test]
    fn format_label_replaces_percent_placeholder() {
        let snapshot = storage_snapshot(500 * GIB, 1000 * GIB, 500 * GIB, 50.0, "ext4");
        let result = format_label("{{ percent }}%", &snapshot);
        assert_eq!(result, "50%");
    }

    #[test]
    fn format_label_percent_pads_single_digits() {
        let snapshot = storage_snapshot(50 * GIB, 1000 * GIB, 950 * GIB, 5.0, "ext4");
        let result = format_label("{{ percent }}", &snapshot);
        assert_eq!(result, "05");
    }

    #[test]
    fn format_label_replaces_used_gib_placeholder() {
        let snapshot = storage_snapshot(256 * GIB, 512 * GIB, 256 * GIB, 50.0, "ext4");
        let result = format_label("{{ used_gib }}G", &snapshot);
        assert_eq!(result, "256.0G");
    }

    #[test]
    fn format_label_replaces_used_mib_placeholder() {
        let snapshot = storage_snapshot(1536 * MIB, 4096 * MIB, 2560 * MIB, 37.5, "ext4");
        let result = format_label("{{ used_mib }}M", &snapshot);
        assert_eq!(result, "1536M");
    }

    #[test]
    fn format_label_replaces_used_auto_placeholder_gib() {
        let snapshot = storage_snapshot(100 * GIB, 500 * GIB, 400 * GIB, 20.0, "ext4");
        let result = format_label("{{ used_auto }}", &snapshot);
        assert_eq!(result, "100.0 GiB");
    }

    #[test]
    fn format_label_replaces_used_auto_placeholder_mib() {
        let snapshot = storage_snapshot(512 * MIB, 2 * GIB, GIB + 512 * MIB, 25.0, "ext4");
        let result = format_label("{{ used_auto }}", &snapshot);
        assert_eq!(result, "512.0 MiB");
    }

    #[test]
    fn format_label_replaces_total_gib_placeholder() {
        let snapshot = storage_snapshot(256 * GIB, 512 * GIB, 256 * GIB, 50.0, "ext4");
        let result = format_label("{{ total_gib }}G", &snapshot);
        assert_eq!(result, "512.0G");
    }

    #[test]
    fn format_label_replaces_total_mib_placeholder() {
        let snapshot = storage_snapshot(1024 * MIB, 4096 * MIB, 3072 * MIB, 25.0, "ext4");
        let result = format_label("{{ total_mib }}M", &snapshot);
        assert_eq!(result, "4096M");
    }

    #[test]
    fn format_label_replaces_free_gib_placeholder() {
        let snapshot = storage_snapshot(300 * GIB, 500 * GIB, 200 * GIB, 60.0, "ext4");
        let result = format_label("{{ free_gib }}G free", &snapshot);
        assert_eq!(result, "200.0G free");
    }

    #[test]
    fn format_label_replaces_free_mib_placeholder() {
        let snapshot = storage_snapshot(2048 * MIB, 4096 * MIB, 2048 * MIB, 50.0, "ext4");
        let result = format_label("{{ free_mib }}M", &snapshot);
        assert_eq!(result, "2048M");
    }

    #[test]
    fn format_label_replaces_filesystem_placeholder() {
        let snapshot = storage_snapshot(100 * GIB, 500 * GIB, 400 * GIB, 20.0, "btrfs");
        let result = format_label("[{{ filesystem }}]", &snapshot);
        assert_eq!(result, "[btrfs]");
    }

    #[test]
    fn format_label_with_multiple_placeholders() {
        let snapshot = storage_snapshot(250 * GIB, 500 * GIB, 250 * GIB, 50.0, "ext4");
        let result = format_label(
            "{{ used_gib }}/{{ total_gib }}G ({{ percent }}%)",
            &snapshot,
        );
        assert_eq!(result, "250.0/500.0G (50%)");
    }

    #[test]
    fn format_label_with_zero_bytes() {
        let snapshot = storage_snapshot(0, 500 * GIB, 500 * GIB, 0.0, "ext4");
        let result = format_label("{{ used_gib }}", &snapshot);
        assert_eq!(result, "0.0");
    }

    #[test]
    fn format_label_with_no_placeholders_returns_unchanged() {
        let snapshot = storage_snapshot(100 * GIB, 500 * GIB, 400 * GIB, 20.0, "ext4");
        let result = format_label("Disk", &snapshot);
        assert_eq!(result, "Disk");
    }

    #[test]
    fn format_label_replaces_tib_placeholders() {
        let snapshot = storage_snapshot(2 * TIB, 4 * TIB, 2 * TIB, 50.0, "ext4");
        let result = format_label("{{ used_tib }}/{{ total_tib }} TiB", &snapshot);
        assert_eq!(result, "2.00/4.00 TiB");
    }
}
