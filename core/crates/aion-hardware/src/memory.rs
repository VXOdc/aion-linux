//! Memory detection from /proc/meminfo.

use aion_types::MemoryInfo;

pub fn detect_memory() -> MemoryInfo {
    let content = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let get_kb = |key: &str| -> u64 {
        content
            .lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u64>().ok())
            .map(|kb| kb.saturating_mul(1024))
            .unwrap_or(0)
    };

    MemoryInfo {
        total_bytes: get_kb("MemTotal:"),
        available_bytes: get_kb("MemAvailable:"),
        swap_total_bytes: get_kb("SwapTotal:"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_reported() {
        let mem = detect_memory();
        assert!(mem.total_bytes > 0);
    }
}
