//! GPU and NPU detection via sysfs PCI enumeration.

use aion_types::{AcceleratorInfo, GpuInfo, GpuVendor};

/// PCI base class 0x03 = display controller.
const PCI_CLASS_DISPLAY: &str = "0x03";
/// PCI base class 0x12 (processing accelerator) covers NPUs/DPUs.
const PCI_CLASS_ACCELERATOR: &str = "0x12";

pub fn detect_gpus() -> Vec<GpuInfo> {
    enumerate_pci(Some(PCI_CLASS_DISPLAY))
        .into_iter()
        .map(|d| {
            let vendor = GpuVendor::from_vendor_id(&d.vendor_id);
            GpuInfo {
                vendor_id: d.vendor_id,
                vendor,
                device_id: d.device_id,
                pci_address: d.address,
                driver: d.driver,
            }
        })
        .collect()
}

pub fn detect_accelerators() -> Vec<AcceleratorInfo> {
    enumerate_pci(Some(PCI_CLASS_ACCELERATOR))
        .into_iter()
        .map(|dev| AcceleratorInfo {
            kind: "npu".to_string(),
            name: format!("PCI device {}", dev.device_id),
            pci_address: Some(dev.address),
        })
        .collect()
}

struct PciDevice {
    address: String,
    vendor_id: String,
    device_id: String,
    _class: String,
    driver: Option<String>,
}

fn enumerate_pci(base_class_filter: Option<&str>) -> Vec<PciDevice> {
    let mut devices = Vec::new();

    let entries = match std::fs::read_dir("/sys/bus/pci/devices") {
        Ok(entries) => entries,
        Err(_) => return devices,
    };

    for entry in entries.flatten() {
        let address = entry.file_name().to_string_lossy().into_owned();
        let base = entry.path();
        let vendor_id = match read_pci_hex(&base, "vendor") {
            Some(v) => v,
            None => continue,
        };
        let device_id = read_pci_hex(&base, "device").unwrap_or_else(|| "0x0000".into());
        let class = read_pci_hex(&base, "class").unwrap_or_else(|| "0x000000".into());

        if let Some(filter) = base_class_filter {
            // The kernel reports a 24-bit class; the first byte after 0x
            // is the base class we filter on.
            let base_class = class.get(0..4).unwrap_or("0x00");
            if base_class != filter {
                continue;
            }
        }

        let driver = std::fs::read_link(base.join("driver")).ok().map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });

        devices.push(PciDevice {
            address,
            vendor_id,
            device_id,
            _class: class,
            driver,
        });
    }
    devices
}

fn read_pci_hex(base: &std::path::Path, file: &str) -> Option<String> {
    let raw = std::fs::read_to_string(base.join(file)).ok()?;
    let trimmed = raw.trim();
    let parsed = trimmed.trim_start_matches("0x");
    u64::from_str_radix(parsed, 16).ok()?;
    Some(trimmed.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_detection_never_panics() {
        let gpus = detect_gpus();
        for gpu in &gpus {
            assert!(gpu.pci_address.contains(':'));
        }
    }
}
