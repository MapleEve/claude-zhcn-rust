//! Shared, side-effect-free domain rules for `claude-zhcn-rust`.
//!
//! This crate deliberately contains no Tauri integration, system discovery,
//! installer, downloader, or patch implementation. Entry points must provide
//! observed inventory data and call these fail-closed rules.

pub mod domain;
pub mod manifest;
pub mod state;

pub use domain::{
    IdentityState, InstallKind, InstallState, InstallationCandidate, InstallationInventory,
    PatchState, Platform, Product,
};
pub use manifest::{
    ArchiveFormat, ArtifactKind, ArtifactManifest, ClaudeCliRelease, ClaudeDesktopRelease,
    DesktopArtifactManifest, DesktopArtifactPlatform, MacosArchitecture, ManifestError,
    SupportedReleaseManifest,
};
pub use state::{evaluate_installation, StateEvaluation};
