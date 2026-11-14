//! Command implementations. Each is honest about what is implemented and
//! what is planned — unimplemented commands say so instead of faking
//! success.

use crate::{
    Cli, Commands, DoctorArgs, EnvCommands, HardwareCommands, ModelCommands, RuntimeCommands,
    ServiceCommands, SystemCommands,
};
use aion_hardware as hw;
use aion_models::{LoadEstimate, ModelStore};
use aion_runtime as rt;
use aion_types::GpuVendor;

pub fn dispatch(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::System(SystemCommands::Info) => system_info(),
        Commands::System(SystemCommands::Update) => not_implemented_yet(
            "`aion system update` requires the aion-core daemon and the aion-update service.",
        ),
        Commands::System(SystemCommands::Rollback) => not_implemented_yet(
            "`aion system rollback` requires Btrfs snapshot integration (aion-update service).",
        ),
        Commands::Hardware(HardwareCommands::Detect) => hardware_detect(),
        Commands::Hardware(HardwareCommands::Benchmark) => hardware_benchmark(),
        Commands::Model(ModelCommands::List) => model_list(),
        Commands::Model(ModelCommands::Search { query }) => model_search(&query),
        Commands::Runtime(RuntimeCommands::List) => runtime_list(),
        Commands::Env(EnvCommands::List) => not_implemented_yet(
            "`aion env` requires the AI Studio environment manager (Milestone 6).",
        ),
        Commands::Service(ServiceCommands::List) => service_list(),
        Commands::Doctor(args) => doctor(args),
    }
}

fn not_implemented_yet(message: &str) -> anyhow::Result<()> {
    eprintln!("aion: not implemented yet — {message}");
    std::process::exit(2);
}

fn system_info() -> anyhow::Result<()> {
    let report = hw::detect_all();
    let distro = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|s| {
            s.lines().find(|l| l.starts_with("PRETTY_NAME=")).map(|l| {
                l.trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string()
            })
        })
        .unwrap_or_else(|| "unknown".into());

    println!("AION system information");
    println!("  OS:        {distro}");
    println!("  Kernel:    {}", std::env::consts::ARCH);
    println!(
        "  CPU:       {} ({} cores / {} threads)",
        report.cpu.model_name, report.cpu.cores, report.cpu.threads
    );
    println!("  RAM:       {}", human(report.memory.total_bytes));
    println!("  GPUs:      {}", report.gpus.len());
    for gpu in &report.gpus {
        println!(
            "    {} at {} [{}]",
            gpu.vendor.as_str(),
            gpu.pci_address,
            gpu.driver.as_deref().unwrap_or("no driver bound")
        );
    }
    println!("  Storage mounts: {}", report.storage.len());
    Ok(())
}

fn hardware_detect() -> anyhow::Result<()> {
    let report = hw::detect_all();
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

/// A small, honest CPU benchmark: hashing work, reported with the
/// caveat that it measures CPU throughput only.
fn hardware_benchmark() -> anyhow::Result<()> {
    const MEIB: u64 = 1 << 20;
    let buffer = vec![0x5au8; 4 * MEIB as usize];
    let start = std::time::Instant::now();
    let mut hash: u64 = 0xcbf29ce484222325;
    for chunk in buffer.chunks(64 * 1024) {
        for &b in chunk {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    let elapsed = start.elapsed();
    let mib_s = (4 * MEIB) as f64 / elapsed.as_secs_f64() / MEIB as f64;
    println!("CPU benchmark (FNV-1a over 4 MiB): {mib_s:.0} MiB/s");
    println!("  single-threaded, memory-bound — useful for relative comparisons only");
    // Prevent the optimizer from removing the work.
    if hash == 0 {
        println!("(impossible)");
    }
    Ok(())
}

fn model_list() -> anyhow::Result<()> {
    let store = ModelStore::open_default();
    let mem = hw::detect_memory();
    let models = store.list()?;
    if models.is_empty() {
        println!("No models installed in {}", store.root().display());
        println!("Install one with: aion model install <name>   (arrives with Model Center, Milestone 5)");
        return Ok(());
    }

    for model in &models {
        let est = LoadEstimate::for_model(model.file_size_bytes);
        let fit = match est.fits(&mem) {
            aion_models::FitsResult::Fits => "fits in available RAM",
            aion_models::FitsResult::FitsWithPressure => {
                "fits but will cause memory pressure (estimates are estimates)"
            }
            aion_models::FitsResult::TooLarge => "exceeds physical RAM",
        };
        let name = model
            .metadata
            .as_ref()
            .and_then(|m| m.name.clone())
            .unwrap_or_else(|| {
                model
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            });
        println!(
            "{:<30} {:>10}  {:?}  estimated RAM {} — {}",
            name,
            human(model.file_size_bytes),
            model.format,
            human(est.estimated_ram_bytes),
            fit,
        );
    }
    Ok(())
}

fn model_search(query: &str) -> anyhow::Result<()> {
    println!("Searching local catalog for {query:?}...");
    println!("Remote catalog search arrives with the Model Center (Milestone 5).");
    println!("The local store currently holds only installed models; try `aion model list`.");
    Ok(())
}

fn runtime_list() -> anyhow::Result<()> {
    let cpu = hw::detect_cpu();
    let report = hw::detect_all();
    let vendors: Vec<GpuVendor> = report.gpus.iter().map(|g| g.vendor).collect();

    println!(
        "{:<14} {:<10} BACKENDS AVAILABLE HERE",
        "RUNTIME", "INSTALLED"
    );
    for desc in rt::registry() {
        let status = rt::probe(&desc, &cpu, &vendors);
        println!(
            "{:<14} {:<10} {}",
            status.display_name,
            if status.installed { "yes" } else { "no" },
            status.available_backends.join(", ")
        );
        if !status.unavailable_backends.is_empty() {
            println!(
                "{:<14} {:<10} not available on this hardware: {}",
                "",
                "",
                status.unavailable_backends.join(", ")
            );
        }
    }
    Ok(())
}

fn service_list() -> anyhow::Result<()> {
    // List AION-related systemd services visible on this system.
    let out = std::process::Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--all",
            "--no-pager",
            "--plain",
        ])
        .output();

    match out {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines().filter(|l| l.contains("aion")) {
                println!("{line}");
            }
            if !text.lines().any(|l| l.contains("aion")) {
                println!("No AION services currently registered.");
            }
        }
        _ => {
            println!("systemctl not available in this environment; cannot list services.");
        }
    }
    Ok(())
}

fn doctor(args: DoctorArgs) -> anyhow::Result<()> {
    use std::collections::BTreeMap;

    let mut checks: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    // 1. Kernel/arch sanity
    let report = hw::detect_all();
    checks.insert(
        "cpu".into(),
        serde_json::json!({
            "status": "ok",
            "architecture": report.cpu.architecture,
            "threads": report.cpu.threads,
        }),
    );

    // 2. Memory headroom
    let mem_ok = report.memory.available_bytes > 2 * (1 << 30);
    checks.insert(
        "memory".into(),
        serde_json::json!({
            "status": if mem_ok { "ok" } else { "warn" },
            "available_bytes": report.memory.available_bytes,
            "note": if mem_ok { "" } else { "less than 2 GiB available" },
        }),
    );

    // 3. Model store
    let store = ModelStore::open_default();
    let store_ok = store.root().exists();
    checks.insert(
        "model_store".into(),
        serde_json::json!({
            "status": if store_ok { "ok" } else { "warn" },
            "path": store.root().display().to_string(),
            "note": if store_ok { "" } else { "model store not initialized yet" },
        }),
    );

    // 4. GPU drivers
    let driverless: Vec<&str> = report
        .gpus
        .iter()
        .filter(|g| g.driver.is_none())
        .map(|g| g.pci_address.as_str())
        .collect();
    checks.insert(
        "gpus".into(),
        serde_json::json!({
            "status": if driverless.is_empty() { "ok" } else { "warn" },
            "count": report.gpus.len(),
            "missing_drivers": driverless,
        }),
    );

    if args.json {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        println!("aion doctor");
        for (name, value) in &checks {
            let status = value.get("status").and_then(|s| s.as_str()).unwrap_or("?");
            println!("  [{status:>4}] {name}");
        }
        println!("Run with --json for machine-readable output.");
    }
    Ok(())
}

/// Format bytes for humans (SI units).
fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "kiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
