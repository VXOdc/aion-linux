//! Thermal zone detection from sysfs.

use aion_types::ThermalZone;

pub fn detect_thermals() -> Vec<ThermalZone> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/thermal") else {
        return out;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let Ok(temp) = std::fs::read_to_string(entry.path().join("temp")) else {
            continue;
        };
        let Ok(temp_millidegrees) = temp.trim().parse::<i64>() else {
            continue;
        };
        let label = std::fs::read_to_string(entry.path().join("type"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| name.clone());
        out.push(ThermalZone {
            label,
            temp_millidegrees,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn thermal_detection_never_panics() {
        let _ = super::detect_thermals();
    }
}
