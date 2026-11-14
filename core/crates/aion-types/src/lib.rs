//! Shared types for the AION core platform.
//!
//! Every subsystem (hardware, models, runtimes, monitor, ...) speaks these
//! types over JSON, so they must stay backward-compatible: additive changes
//! only, removals require a major version bump of `aion-api`.

use serde::{Deserialize, Serialize};

/// Overall hardware report produced by `aion-hardware`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareReport {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
    pub npus: Vec<AcceleratorInfo>,
    pub storage: Vec<StorageInfo>,
    /// Thermal zone readings in millidegrees Celsius (may be empty).
    pub thermals: Vec<ThermalZone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// e.g. "x86_64" or "aarch64"
    pub architecture: String,
    /// Hardware model name from /proc/cpuinfo (e.g. "AMD Ryzen 9 7900X").
    pub model_name: String,
    /// Logical processors.
    pub threads: u32,
    /// Physical cores (best effort).
    pub cores: u32,
    /// Notable CPU instruction features (subset, for compatibility checks).
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total system RAM in bytes.
    pub total_bytes: u64,
    /// Currently available RAM in bytes.
    pub available_bytes: u64,
    /// Swap total in bytes (0 when absent).
    pub swap_total_bytes: u64,
}

/// A detected PCI device class that can accelerate AI workloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// PCI vendor id, e.g. "0x10de"
    pub vendor_id: String,
    pub vendor: GpuVendor,
    /// PCI device id, e.g. "0x2684"
    pub device_id: String,
    /// PCI bus address, e.g. "0000:0a:00.0"
    pub pci_address: String,
    /// Kernel driver currently bound, if any.
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

/// Any non-GPU accelerator (NPU, TPU, etc.). Detection is best-effort.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceleratorInfo {
    pub kind: String,
    pub name: String,
    pub pci_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Device node, e.g. "/dev/nvme0n1" or mount source.
    pub device: String,
    /// Mount point when mounted.
    pub mount_point: Option<String>,
    /// Filesystem type, e.g. "btrfs".
    pub fs_type: Option<String>,
    /// Total size in bytes when known.
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalZone {
    pub label: String,
    /// Millidegrees Celsius, as reported by the kernel.
    pub temp_millidegrees: i64,
}

impl GpuVendor {
    pub fn as_str(&self) -> &'static str {
        match self {
            GpuVendor::Nvidia => "nvidia",
            GpuVendor::Amd => "amd",
            GpuVendor::Intel => "intel",
            GpuVendor::Unknown => "unknown",
        }
    }

    pub fn from_vendor_id(id: &str) -> GpuVendor {
        match id {
            "0x10de" => GpuVendor::Nvidia,
            "0x1002" => GpuVendor::Amd,
            "0x8086" => GpuVendor::Intel,
            _ => GpuVendor::Unknown,
        }
    }
}
