use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use claude_zhcn_core::{evaluate_installation, InstallationInventory, SupportedReleaseManifest};

/// Read-only bootstrap CLI for validating pinned manifests and inventory snapshots.
///
/// Real system discovery, download, installation, localization, backup, and rollback
/// are intentionally not implemented in this scaffold.
#[derive(Debug, Parser)]
#[command(name = "claude-zhcn", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate one exact, version-pinned supported-release manifest.
    ValidateManifest {
        /// JSON manifest to validate.
        file: PathBuf,
    },
    /// Evaluate a caller-supplied read-only inventory snapshot.
    Status {
        /// Inventory JSON produced by a future platform discovery adapter.
        #[arg(long)]
        input: PathBuf,
        /// Exact supported-release JSON to validate before evaluation.
        #[arg(long)]
        manifest: PathBuf,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::ValidateManifest { file } => {
            read_manifest(&file)?;
            println!("{}", serde_json::json!({ "valid": true }));
        }
        Command::Status { input, manifest } => {
            let manifest = read_manifest(&manifest)?;
            let input = read_file(&input, "inventory")?;
            let inventory: InstallationInventory = serde_json::from_str(&input)
                .map_err(|error| format!("inventory JSON is invalid: {error}"))?;
            let evaluation = evaluate_installation(&inventory, &manifest);
            let output = serde_json::to_string_pretty(&evaluation)
                .map_err(|error| format!("could not serialize status: {error}"))?;
            println!("{output}");
        }
    }
    Ok(())
}

fn read_manifest(path: &PathBuf) -> Result<SupportedReleaseManifest, String> {
    let contents = read_file(path, "manifest")?;
    SupportedReleaseManifest::parse_and_validate(&contents)
        .map_err(|error| format!("supported-release manifest was rejected: {error}"))
}

fn read_file(path: &PathBuf, kind: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("could not read {kind} {}: {error}", path.display()))
}
