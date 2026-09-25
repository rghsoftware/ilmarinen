//! `ilmarinen`: doctor | setup | init | upgrade | off | on.

use ilmarinen::{assets, doctor, init, manifest, setup};

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "ilmarinen",
    version,
    about = "Check a machine, set it up, and stamp repos with the Ilmarinen workflow"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check tools, MCP liveness in both hosts, and personal state. Offers `mise install`.
    Doctor(doctor::Args),
    /// Configure this machine (host MCP registration, instructions, personal memory). Runs doctor first.
    Setup(setup::Args),
    /// Stamp the per-repo skeleton into <repo>. Idempotent. Must succeed
    /// without network; network is used only for opt-in sync features and
    /// never for telemetry.
    Init(init::InitArgs),
    /// Show template changes since <repo> was stamped; apply only with confirmation.
    Upgrade { repo: PathBuf },
    /// Turn Ilmarinen off in an initialized repo: writes .ilmarinen.off, which
    /// every hook and skill respects (gitignored; per checkout).
    Off { repo: Option<PathBuf> },
    /// Turn Ilmarinen back on: removes .ilmarinen.off.
    On { repo: Option<PathBuf> },
}

/// Every child process inherits the per-run telemetry opt-outs from tools.toml.
fn opt_out_env() {
    if let Ok(tools) = manifest::parse(&assets::tools_toml()) {
        for (k, v) in tools.iter().filter_map(|t| t.telemetry.as_ref()).flat_map(|t| &t.env) {
            // SAFETY: called first thing in main, before any thread is spawned.
            unsafe { std::env::set_var(k, v) };
        }
    }
}

fn main() -> ExitCode {
    opt_out_env();
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Doctor(a) => doctor::run(&a).map(|r| r.passed()),
        Cmd::Setup(a) => setup::run(&a).map(|_| true),
        Cmd::Init(a) => init::init(&a).map(|_| true),
        Cmd::Upgrade { repo } => init::upgrade(&repo).map(|_| true),
        Cmd::Off { repo } => init::switch(repo.as_deref(), false).map(|_| true),
        Cmd::On { repo } => init::switch(repo.as_deref(), true).map(|_| true),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
