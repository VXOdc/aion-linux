//! CPU detection from procfs.

use aion_types::CpuInfo;

/// Instruction features we surface for runtime compatibility checks.
const INTERESTING_FEATURES: &[&str] = &[
    // x86-64
    "avx2", "avx512f", "fma3", "sse4_2", // AArch64
    "asimd", "bf16", "i8mm", "sve",
];

pub fn detect_cpu() -> CpuInfo {
    let architecture = std::env::consts::ARCH.to_string();
    let (model_name, threads, cores) = read_proc_cpuinfo();
    let features = read_cpu_features();

    CpuInfo {
        architecture,
        model_name,
        threads,
        cores,
        features,
    }
}

/// Parse /proc/cpuinfo. Format differs between architectures, so we scan
/// for known keys instead of assuming a fixed layout.
fn read_proc_cpuinfo() -> (String, u32, u32) {
    let content = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut model_name = String::new();
    let mut processors = std::collections::HashSet::new();
    let mut physical_ids = std::collections::HashSet::new();
    let mut cores_per_package: u32 = 0;

    for line in content.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        match key {
            "model name" => {
                if model_name.is_empty() {
                    model_name = value.to_string();
                }
            }
            "Hardware" => {
                // AArch64 kernels expose the SoC name here.
                if model_name.is_empty() {
                    model_name = value.to_string();
                }
            }
            "processor" => {
                processors.insert(value.to_string());
            }
            "physical id" => {
                physical_ids.insert(value.to_string());
            }
            "cpu cores" => {
                if let Ok(n) = value.parse::<u32>() {
                    cores_per_package = cores_per_package.max(n);
                }
            }
            _ => {}
        }
    }

    let threads = processors.len().max(1) as u32;
    let cores = if cores_per_package > 0 {
        cores_per_package * physical_ids.len().max(1) as u32
    } else {
        threads
    };

    (model_name, threads, cores)
}

/// Read CPU features: /proc/cpuinfo `flags` on x86-64, `Features` on AArch64.
fn read_cpu_features() -> Vec<String> {
    let content = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut flags: Vec<String> = Vec::new();

    for line in content.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        if key == "flags" || key == "Features" {
            for tok in value.split_whitespace() {
                let tok = tok.trim_matches(',');
                if INTERESTING_FEATURES.contains(&tok) && !flags.iter().any(|f| f == tok) {
                    flags.push(tok.to_string());
                }
            }
        }
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_at_least_one_thread() {
        let cpu = detect_cpu();
        assert!(cpu.threads >= 1);
        assert!(cpu.cores >= 1);
        assert!(!cpu.architecture.is_empty());
    }
}
