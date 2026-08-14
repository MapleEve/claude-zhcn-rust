use serde::{Deserialize, Serialize};

/// A Claude product managed by the localizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Product {
    Cli,
    Desktop,
}

/// The three host targets included in the first-release contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "macos-aarch64")]
    MacosArm64,
    #[serde(rename = "macos-x86_64")]
    MacosX86_64,
    #[serde(rename = "windows-x86_64")]
    WindowsX86_64,
}

/// Observed installation layout. This describes the installed result, not the
/// file extension of a downloaded installer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallKind {
    CliNative,
    CliNpm,
    DesktopMacosApp,
    DesktopMsix,
    DesktopSquirrel,
}

/// Identity evidence collected by a platform adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdentityState {
    Verified,
    Unknown,
    Corrupt,
}

/// Patch evidence collected without changing the target installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PatchState {
    Clean,
    Verified,
    Unknown,
    Drift,
}

/// One read-only installation observation supplied by a platform adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallationCandidate {
    pub canonical_path: String,
    pub install_kind: InstallKind,
    pub version: Option<String>,
    pub identity: IdentityState,
    pub patch: PatchState,
}

/// A snapshot consumed by the pure state evaluator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallationInventory {
    pub product: Product,
    pub platform: Platform,
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub candidates: Vec<InstallationCandidate>,
}

/// Public state surfaced consistently by CLI and desktop entry points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum InstallState {
    NotInstalled,
    Older,
    ExactClean,
    ExactPatched,
    Newer,
    Unknown,
    Corrupt,
    Running,
}
