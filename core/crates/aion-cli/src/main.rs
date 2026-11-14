//! `aion` — the AION Linux command-line interface.
//!
//! Every command provides useful errors and JSON output (--json) so the
//! desktop applications and scripts can consume the same data.

mod commands;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "aion",
    version,
    about = "AION Linux system management",
    after_help = "Run `aion <command> --help` for details on any command."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// System information, updates, and rollback
    #[command(subcommand)]
    System(SystemCommands),
    /// Hardware detection and benchmarking
    #[command(subcommand)]
    Hardware(HardwareCommands),
    /// Model library management
    #[command(subcommand)]
    Model(ModelCommands),
    /// AI runtime management
    #[command(subcommand)]
    Runtime(RuntimeCommands),
    /// Environment management
    #[command(subcommand)]
    Env(EnvCommands),
    /// Service management
    #[command(subcommand)]
    Service(ServiceCommands),
    /// Diagnose common problems
    Doctor(DoctorArgs),
}

#[derive(clap::Subcommand)]
enum SystemCommands {
    /// Show system summary
    Info,
    /// Apply system updates (requires the aion-core daemon)
    Update,
    /// Roll back to the previous Btrfs snapshot
    Rollback,
}

#[derive(clap::Subcommand)]
enum HardwareCommands {
    /// Detect and report all hardware
    Detect,
    /// Run a quick CPU benchmark
    Benchmark,
}

#[derive(clap::Subcommand)]
enum ModelCommands {
    /// List locally installed models
    List,
    /// Search the offline catalog (remote search arrives with Model Center)
    Search { query: String },
}

#[derive(clap::Subcommand)]
enum RuntimeCommands {
    /// List AI runtimes and their status on this machine
    List,
}

#[derive(clap::Subcommand)]
enum EnvCommands {
    /// List development environments
    List,
}

#[derive(clap::Subcommand)]
enum ServiceCommands {
    /// List AION-managed services
    List,
}

#[derive(clap::Args)]
struct DoctorArgs {
    /// Emit machine-readable JSON for the Model Center support tooling
    #[arg(long)]
    json: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    commands::dispatch(cli)
}
