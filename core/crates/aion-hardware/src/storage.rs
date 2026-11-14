//! Storage detection from /proc/self/mounts.
//!
//! Model storage should live on a dedicated subvolume or mount point,
//! kept separate from the OS root — the CLI surfaces this distinction.

use aion_types::StorageInfo;

pub fn detect_storage() -> Vec<StorageInfo> {
    let content = std::fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();

    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 3 {
            continue;
        }
        let (device, mount_point, fs_type) = (fields[0], fields[1], fields[2]);
        if !device.starts_with("/dev/") {
            continue;
        }
        if !seen.insert(device.to_string()) {
            continue;
        }
        out.push(StorageInfo {
            device: device.to_string(),
            mount_point: Some(mount_point.to_string()),
            fs_type: Some(fs_type.to_string()),
            total_bytes: None,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_detection_never_panics() {
        let storage = detect_storage();
        for s in &storage {
            assert!(s.device.starts_with("/dev/"));
        }
    }
}
