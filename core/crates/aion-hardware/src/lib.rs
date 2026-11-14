//! Hardware detection for AION.
//!
//! Reads directly from procfs/sysfs — no external tools, no root required.
//! Never assumes particular hardware: every probe degrades gracefully to an
//! empty result. All estimates and capabilities must be verified by the
//! caller before advertising support for a workload.

mod cpu;
mod gpu;
mod memory;
mod storage;
mod thermals;

pub use cpu::detect_cpu;
pub use gpu::{detect_accelerators, detect_gpus};
pub use memory::detect_memory;
pub use storage::detect_storage;
pub use thermals::detect_thermals;

use aion_types::HardwareReport;

/// Build a full hardware report. Safe to call on any Linux system; missing
/// information is simply absent from the report rather than an error.
pub fn detect_all() -> HardwareReport {
    HardwareReport {
        cpu: detect_cpu(),
        memory: detect_memory(),
        gpus: detect_gpus(),
        npus: detect_accelerators(),
        storage: detect_storage(),
        thermals: detect_thermals(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_is_self_consistent() {
        let report = detect_all();
        assert!(!report.cpu.architecture.is_empty());
        assert!(report.cpu.threads > 0);
        assert!(report.memory.total_bytes > 0);
        // GPU list may be empty in containers — that is valid.
    }
}
