//! Model library management for AION.
//!
//! Responsibilities:
//! - Parse GGUF headers (name, architecture, quantization, context length).
//! - Scan the local model store and report what is installed.
//! - Estimate RAM/VRAM needed to load a model. Estimates are labeled as
//!   estimates: quantization overheads vary and are never exact.

mod gguf;
mod store;

pub use gguf::{parse_gguf_header, GgufHeader, GgufMetadata};
pub use store::{LocalModel, ModelStore};

use aion_types::MemoryInfo;

/// Where models live by default. Deliberately NOT under /usr — the model
/// store must survive OS rollbacks and is excluded from system snapshots.
pub const DEFAULT_MODEL_STORE: &str = "/var/lib/aion/models";

/// Estimated resource requirements for loading a model.
///
/// Estimate = weights + a context/KV-cache allowance. The KV cache term is
/// rough by nature; callers must present it as an estimate, never a promise.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoadEstimate {
    /// Model weights on disk.
    pub file_size_bytes: u64,
    /// Estimated RAM needed for weights plus activation overhead.
    pub estimated_ram_bytes: u64,
    /// KV-cache allowance for a modest context, included in the RAM figure.
    pub kv_cache_allowance_bytes: u64,
}

impl LoadEstimate {
    pub fn for_model(file_size_bytes: u64) -> Self {
        // Empirical overhead factor for GGUF inference (activations,
        // fragmentation, runtime structures). Conservative on purpose.
        const OVERHEAD_FACTOR: f64 = 1.15;
        // 1 GiB allowance for KV cache at default context sizes.
        const KV_ALLOWANCE: u64 = 1 << 30;

        let est = (file_size_bytes as f64 * OVERHEAD_FACTOR) as u64 + KV_ALLOWANCE;
        Self {
            file_size_bytes,
            estimated_ram_bytes: est,
            kv_cache_allowance_bytes: KV_ALLOWANCE,
        }
    }

    /// Honest comparison against available memory. Never treats swap as
    /// equivalent to RAM: swap is reported separately.
    pub fn fits(&self, mem: &MemoryInfo) -> FitsResult {
        if self.estimated_ram_bytes <= mem.available_bytes {
            FitsResult::Fits
        } else if self.estimated_ram_bytes <= mem.total_bytes {
            FitsResult::FitsWithPressure
        } else {
            FitsResult::TooLarge
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitsResult {
    Fits,
    /// Loadable but will push the system into memory pressure / swap.
    FitsWithPressure,
    TooLarge,
}
