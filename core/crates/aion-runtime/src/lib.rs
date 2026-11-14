//! AI runtime registry.
//!
//! Every runtime ships a machine-readable capability descriptor. The
//! compatibility matrix is data, not code: a runtime is only advertised
//! for a combination when the descriptor says so. Unsupported
//! combinations are reported as unsupported — never guessed.

use aion_types::{CpuInfo, GpuVendor};
use serde::{Deserialize, Serialize};

/// A runtime's supported hardware/backend matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    /// Executable to probe for availability.
    pub executable: &'static str,
    /// Workload categories this runtime can serve.
    pub workloads: Vec<Workload>,
    /// Backend availability rules.
    pub backends: Vec<BackendRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Workload {
    LlmInference,
    ImageGeneration,
    SpeechRecognition,
    TextToSpeech,
    Embeddings,
    FineTuning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendRule {
    /// e.g. "cuda", "rocm", "vulkan", "cpu", "metal", "onnx"
    pub backend: String,
    /// CPU architectures this backend supports on this runtime.
    pub architectures: Vec<String>,
    /// GPU vendors this backend applies to (empty = vendor-agnostic).
    pub gpu_vendors: Vec<GpuVendor>,
    /// Experimental rules are surfaced as experimental, never as supported.
    pub experimental: bool,
}

/// The built-in registry. Mirrors data/compatibility/runtimes.yaml — the
/// YAML file is the source of truth for tooling; this is the in-process
/// copy used by the CLI. They are kept in sync by CI.
pub fn registry() -> Vec<RuntimeDescriptor> {
    use GpuVendor::*;
    vec![
        RuntimeDescriptor {
            id: "llama.cpp",
            display_name: "llama.cpp",
            executable: "llama-server",
            workloads: vec![Workload::LlmInference, Workload::Embeddings],
            backends: vec![
                BackendRule {
                    backend: "cpu".into(),
                    architectures: vec!["x86_64".into(), "aarch64".into()],
                    gpu_vendors: vec![],
                    experimental: false,
                },
                BackendRule {
                    backend: "cuda".into(),
                    architectures: vec!["x86_64".into(), "aarch64".into()],
                    gpu_vendors: vec![Nvidia],
                    experimental: false,
                },
                BackendRule {
                    backend: "vulkan".into(),
                    architectures: vec!["x86_64".into(), "aarch64".into()],
                    gpu_vendors: vec![Amd, Intel, Nvidia],
                    experimental: false,
                },
                // ROCm exists on x86-64 only. See ADR-002.
                BackendRule {
                    backend: "rocm".into(),
                    architectures: vec!["x86_64".into()],
                    gpu_vendors: vec![Amd],
                    experimental: false,
                },
            ],
        },
        RuntimeDescriptor {
            id: "ollama",
            display_name: "Ollama",
            executable: "ollama",
            workloads: vec![Workload::LlmInference, Workload::Embeddings],
            backends: vec![
                BackendRule {
                    backend: "cpu".into(),
                    architectures: vec!["x86_64".into(), "aarch64".into()],
                    gpu_vendors: vec![],
                    experimental: false,
                },
                BackendRule {
                    backend: "cuda".into(),
                    architectures: vec!["x86_64".into(), "aarch64".into()],
                    gpu_vendors: vec![Nvidia],
                    experimental: false,
                },
            ],
        },
        RuntimeDescriptor {
            id: "vllm",
            display_name: "vLLM",
            executable: "vllm",
            workloads: vec![Workload::LlmInference],
            backends: vec![
                // AArch64 CPU backend with NEON; prebuilt wheels since 0.11.2.
                BackendRule {
                    backend: "cpu".into(),
                    architectures: vec!["x86_64".into(), "aarch64".into()],
                    gpu_vendors: vec![],
                    experimental: false,
                },
                BackendRule {
                    backend: "cuda".into(),
                    architectures: vec!["x86_64".into(), "aarch64".into()],
                    gpu_vendors: vec![Nvidia],
                    experimental: false,
                },
            ],
        },
        RuntimeDescriptor {
            id: "onnxruntime",
            display_name: "ONNX Runtime",
            executable: "onnxruntime",
            workloads: vec![
                Workload::LlmInference,
                Workload::SpeechRecognition,
                Workload::Embeddings,
            ],
            backends: vec![BackendRule {
                backend: "cpu".into(),
                architectures: vec!["x86_64".into(), "aarch64".into()],
                gpu_vendors: vec![],
                experimental: false,
            }],
        },
    ]
}

/// Live status of a runtime on this machine.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStatus {
    pub id: String,
    pub display_name: String,
    pub installed: bool,
    pub version: Option<String>,
    /// Backends usable on THIS machine, per the descriptor + hardware.
    pub available_backends: Vec<String>,
    /// Backends that exist for the runtime but not on this machine.
    pub unavailable_backends: Vec<String>,
}

/// Probe a runtime's availability on this machine and intersect its
/// descriptor with the actual hardware.
pub fn probe(desc: &RuntimeDescriptor, cpu: &CpuInfo, gpu_vendors: &[GpuVendor]) -> RuntimeStatus {
    let installed = which(desc.executable);
    let mut available = Vec::new();
    let mut unavailable = Vec::new();

    for rule in &desc.backends {
        let arch_ok = rule.architectures.iter().any(|a| a == &cpu.architecture);
        let vendor_ok =
            rule.gpu_vendors.is_empty() || rule.gpu_vendors.iter().any(|v| gpu_vendors.contains(v));
        let ok = arch_ok && vendor_ok && (rule.gpu_vendors.is_empty() || !gpu_vendors.is_empty());
        if ok {
            available.push(rule.backend.clone());
        } else {
            unavailable.push(rule.backend.clone());
        }
    }

    RuntimeStatus {
        id: desc.id.to_string(),
        display_name: desc.display_name.to_string(),
        installed,
        version: None,
        available_backends: available,
        unavailable_backends: unavailable,
    }
}

fn which(exe: &str) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path).any(|dir| dir.join(exe).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocm_never_offered_on_aarch64() {
        let regs = registry();
        let llama = regs.iter().find(|r| r.id == "llama.cpp").unwrap();
        let rocm = llama.backends.iter().find(|b| b.backend == "rocm").unwrap();
        assert!(rocm.architectures.contains(&"x86_64".to_string()));
        assert!(!rocm.architectures.contains(&"aarch64".to_string()));
    }

    #[test]
    fn probe_matches_architecture() {
        let regs = registry();
        let llama = regs.iter().find(|r| r.id == "llama.cpp").unwrap();
        let cpu = CpuInfo {
            architecture: "aarch64".into(),
            model_name: "test".into(),
            threads: 8,
            cores: 8,
            features: vec![],
        };
        let status = probe(llama, &cpu, &[]);
        assert!(status.available_backends.contains(&"cpu".to_string()));
        assert!(!status.available_backends.contains(&"rocm".to_string()));
    }
}
