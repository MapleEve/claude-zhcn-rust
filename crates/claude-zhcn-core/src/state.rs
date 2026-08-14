use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{
    IdentityState, InstallKind, InstallState, InstallationCandidate, InstallationInventory,
    PatchState, Platform, Product, SupportedReleaseManifest,
};

/// Pure state-machine result. `underlyingState` preserves the diagnostic state
/// when a supported target is currently running.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateEvaluation {
    pub state: InstallState,
    pub underlying_state: Option<InstallState>,
    pub write_allowed: bool,
    pub reason: String,
}

impl StateEvaluation {
    fn terminal(state: InstallState, write_allowed: bool, reason: impl Into<String>) -> Self {
        Self {
            state,
            underlying_state: None,
            write_allowed,
            reason: reason.into(),
        }
    }
}

/// Evaluate a read-only inventory against one exact supported release.
///
/// This function never selects between multiple candidates. Ambiguous native +
/// npm or MSIX + Squirrel inventories therefore fail closed as `Unknown`.
#[must_use]
pub fn evaluate_installation(
    inventory: &InstallationInventory,
    manifest: &SupportedReleaseManifest,
) -> StateEvaluation {
    if inventory.candidates.is_empty() {
        return StateEvaluation::terminal(
            InstallState::NotInstalled,
            true,
            "no installation candidates were supplied",
        );
    }
    if inventory.candidates.len() != 1 {
        return StateEvaluation::terminal(
            InstallState::Unknown,
            false,
            "multiple installation candidates are ambiguous; no candidate was selected",
        );
    }

    let candidate = &inventory.candidates[0];
    if !kind_matches_product_and_platform(inventory.product, inventory.platform, candidate) {
        return StateEvaluation::terminal(
            InstallState::Unknown,
            false,
            "the observed installation kind is detection-only or unsupported on this platform",
        );
    }

    let base = evaluate_single_candidate(
        candidate,
        manifest.supported_version(inventory.product, inventory.platform),
    );
    if inventory.running && !matches!(base, InstallState::Unknown | InstallState::Corrupt) {
        return StateEvaluation {
            state: InstallState::Running,
            underlying_state: Some(base),
            write_allowed: false,
            reason: "the supported target is running; exit it before any write operation"
                .to_owned(),
        };
    }

    let write_allowed = matches!(
        base,
        InstallState::ExactClean | InstallState::ExactPatched | InstallState::NotInstalled
    );
    StateEvaluation::terminal(base, write_allowed, reason_for(base))
}

fn kind_matches_product_and_platform(
    product: Product,
    platform: Platform,
    candidate: &InstallationCandidate,
) -> bool {
    matches!(
        (product, platform, candidate.install_kind),
        (Product::Cli, _, InstallKind::CliNative)
            | (
                Product::Desktop,
                Platform::MacosArm64 | Platform::MacosX86_64,
                InstallKind::DesktopMacosApp,
            )
            | (
                Product::Desktop,
                Platform::WindowsX86_64,
                InstallKind::DesktopMsix
            )
    )
}

fn evaluate_single_candidate(candidate: &InstallationCandidate, target: &str) -> InstallState {
    if candidate.canonical_path.trim().is_empty()
        || candidate.identity == IdentityState::Corrupt
        || candidate.patch == PatchState::Drift
    {
        return InstallState::Corrupt;
    }
    if candidate.identity == IdentityState::Unknown {
        return InstallState::Unknown;
    }

    let Some(version) = candidate.version.as_deref() else {
        return InstallState::Unknown;
    };
    let Some(ordering) = compare_dotted_versions(version, target) else {
        return InstallState::Unknown;
    };
    match ordering {
        Ordering::Less => InstallState::Older,
        Ordering::Greater => InstallState::Newer,
        Ordering::Equal => match candidate.patch {
            PatchState::Clean => InstallState::ExactClean,
            PatchState::Verified => InstallState::ExactPatched,
            PatchState::Unknown => InstallState::Unknown,
            PatchState::Drift => InstallState::Corrupt,
        },
    }
}

fn compare_dotted_versions(observed: &str, target: &str) -> Option<Ordering> {
    let observed = parse_dotted_version(observed)?;
    let target = parse_dotted_version(target)?;
    (observed.len() == target.len()).then(|| observed.cmp(&target))
}

fn parse_dotted_version(value: &str) -> Option<Vec<u64>> {
    let parts: Vec<_> = value.split('.').collect();
    if !(3..=4).contains(&parts.len()) {
        return None;
    }
    parts
        .into_iter()
        .map(|part| {
            if part.is_empty()
                || !part.bytes().all(|byte| byte.is_ascii_digit())
                || (part != "0" && part.starts_with('0'))
            {
                return None;
            }
            part.parse().ok()
        })
        .collect()
}

fn reason_for(state: InstallState) -> &'static str {
    match state {
        InstallState::NotInstalled => "no installation was found",
        InstallState::Older => "the verified installation is older than the supported version",
        InstallState::ExactClean => "the installation exactly matches and is not localized",
        InstallState::ExactPatched => "the installation and localization receipt exactly match",
        InstallState::Newer => "the installation is newer; update this tool or wait for support",
        InstallState::Unknown => "the installation cannot be proven safe to modify",
        InstallState::Corrupt => "identity or localization evidence is inconsistent",
        InstallState::Running => "the target is running",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ArchiveFormat, ArtifactKind, ArtifactManifest, ClaudeCliRelease, ClaudeDesktopRelease,
        DesktopArtifactManifest, DesktopArtifactPlatform, MacosArchitecture,
        SupportedReleaseManifest,
    };

    fn hash(seed: char) -> String {
        std::iter::repeat_n(seed, 64).collect()
    }

    fn artifact(platform: Platform, kind: ArtifactKind, version: &str) -> ArtifactManifest {
        ArtifactManifest {
            platform,
            kind,
            archive_format: match platform {
                Platform::MacosArm64 | Platform::MacosX86_64 => ArchiveFormat::TarGz,
                Platform::WindowsX86_64 => ArchiveFormat::Zip,
            },
            url: format!("https://downloads.anthropic.test/{version}/artifact"),
            size: 1,
            sha256: hash('a'),
            identity: "official".to_owned(),
            publisher: "publisher".to_owned(),
        }
    }

    fn desktop_artifact(
        platform: DesktopArtifactPlatform,
        kind: ArtifactKind,
        version: &str,
    ) -> DesktopArtifactManifest {
        DesktopArtifactManifest {
            platform,
            kind,
            architectures: (platform == DesktopArtifactPlatform::MacosUniversal)
                .then(|| vec![MacosArchitecture::Aarch64, MacosArchitecture::X86_64]),
            url: format!("https://downloads.anthropic.test/{version}/artifact"),
            size: 1,
            sha256: hash('a'),
            identity: "official".to_owned(),
            publisher: "publisher".to_owned(),
        }
    }

    fn manifest() -> SupportedReleaseManifest {
        SupportedReleaseManifest {
            schema_version: 1,
            localizer_version: "0.1.0".to_owned(),
            claude_cli: ClaudeCliRelease {
                version: "2.1.226".to_owned(),
                resource_sha256: hash('b'),
                patch_profile: "cli-profile".to_owned(),
                artifacts: vec![
                    artifact(Platform::MacosArm64, ArtifactKind::NativeBinary, "2.1.226"),
                    artifact(Platform::MacosX86_64, ArtifactKind::NativeBinary, "2.1.226"),
                    artifact(
                        Platform::WindowsX86_64,
                        ArtifactKind::NativeBinary,
                        "2.1.226",
                    ),
                ],
            },
            claude_desktop: ClaudeDesktopRelease {
                version: "1.28929.0".to_owned(),
                windows_appx_version: "1.28929.0.0".to_owned(),
                release: "d1a6bcd4ef8627d603a8290548a984220b6701cf".to_owned(),
                build: "d1a6bc".to_owned(),
                commit_timestamp: "2026-08-11T18:58:06.000Z".to_owned(),
                resource_sha256: hash('c'),
                patch_profile: "desktop-profile".to_owned(),
                artifacts: vec![
                    desktop_artifact(
                        DesktopArtifactPlatform::MacosUniversal,
                        ArtifactKind::Dmg,
                        "1.28929.0",
                    ),
                    desktop_artifact(
                        DesktopArtifactPlatform::WindowsX86_64,
                        ArtifactKind::Msix,
                        "1.28929.0",
                    ),
                ],
            },
        }
    }

    fn inventory(product: Product, version: &str) -> InstallationInventory {
        InstallationInventory {
            product,
            platform: Platform::MacosArm64,
            running: false,
            candidates: vec![InstallationCandidate {
                canonical_path: "/Applications/Claude.app".to_owned(),
                install_kind: match product {
                    Product::Cli => InstallKind::CliNative,
                    Product::Desktop => InstallKind::DesktopMacosApp,
                },
                version: Some(version.to_owned()),
                identity: IdentityState::Verified,
                patch: PatchState::Clean,
            }],
        }
    }

    #[test]
    fn evaluates_exact_clean_and_patched() {
        let exact = inventory(Product::Cli, "2.1.226");
        assert_eq!(
            evaluate_installation(&exact, &manifest()).state,
            InstallState::ExactClean
        );

        let mut patched = exact;
        patched.candidates[0].patch = PatchState::Verified;
        assert_eq!(
            evaluate_installation(&patched, &manifest()).state,
            InstallState::ExactPatched
        );
    }

    #[test]
    fn evaluates_older_and_newer_and_blocks_newer() {
        let older = evaluate_installation(&inventory(Product::Cli, "2.1.225"), &manifest());
        assert_eq!(older.state, InstallState::Older);
        assert!(!older.write_allowed);

        let newer = evaluate_installation(&inventory(Product::Cli, "2.1.227"), &manifest());
        assert_eq!(newer.state, InstallState::Newer);
        assert!(!newer.write_allowed);
    }

    #[test]
    fn evaluates_exact_windows_msix_with_the_four_segment_appx_version() {
        let mut input = inventory(Product::Desktop, "1.28929.0.0");
        input.platform = Platform::WindowsX86_64;
        input.candidates[0].canonical_path = "C:/Program Files/WindowsApps/Claude".to_owned();
        input.candidates[0].install_kind = InstallKind::DesktopMsix;

        let evaluation = evaluate_installation(&input, &manifest());
        assert_eq!(evaluation.state, InstallState::ExactClean);
        assert!(evaluation.write_allowed);
    }

    #[test]
    fn multiple_candidates_are_unknown_including_native_and_npm() {
        let mut input = inventory(Product::Cli, "2.1.226");
        input.candidates.push(InstallationCandidate {
            canonical_path: "/usr/local/lib/node_modules/@anthropic-ai/claude-code/cli.js"
                .to_owned(),
            install_kind: InstallKind::CliNpm,
            version: Some("2.1.226".to_owned()),
            identity: IdentityState::Verified,
            patch: PatchState::Clean,
        });
        assert_eq!(
            evaluate_installation(&input, &manifest()).state,
            InstallState::Unknown
        );
    }

    #[test]
    fn msix_and_squirrel_are_unknown() {
        let mut input = inventory(Product::Desktop, "1.28929.0");
        input.platform = Platform::WindowsX86_64;
        input.candidates[0].install_kind = InstallKind::DesktopMsix;
        input.candidates.push(InstallationCandidate {
            canonical_path: "C:/Users/test/AppData/Local/AnthropicClaude/app-1.28929.0".to_owned(),
            install_kind: InstallKind::DesktopSquirrel,
            version: Some("1.28929.0".to_owned()),
            identity: IdentityState::Verified,
            patch: PatchState::Clean,
        });
        assert_eq!(
            evaluate_installation(&input, &manifest()).state,
            InstallState::Unknown
        );
    }

    #[test]
    fn corrupt_and_running_fail_closed() {
        let mut corrupt = inventory(Product::Cli, "2.1.226");
        corrupt.candidates[0].patch = PatchState::Drift;
        let result = evaluate_installation(&corrupt, &manifest());
        assert_eq!(result.state, InstallState::Corrupt);
        assert!(!result.write_allowed);

        let mut running = inventory(Product::Cli, "2.1.226");
        running.running = true;
        let result = evaluate_installation(&running, &manifest());
        assert_eq!(result.state, InstallState::Running);
        assert_eq!(result.underlying_state, Some(InstallState::ExactClean));
        assert!(!result.write_allowed);
    }

    #[test]
    fn npm_only_and_squirrel_only_are_detection_only_unknown() {
        let mut npm = inventory(Product::Cli, "2.1.226");
        npm.candidates[0].install_kind = InstallKind::CliNpm;
        assert_eq!(
            evaluate_installation(&npm, &manifest()).state,
            InstallState::Unknown
        );

        let mut squirrel = inventory(Product::Desktop, "1.28929.0");
        squirrel.platform = Platform::WindowsX86_64;
        squirrel.candidates[0].install_kind = InstallKind::DesktopSquirrel;
        assert_eq!(
            evaluate_installation(&squirrel, &manifest()).state,
            InstallState::Unknown
        );
    }

    #[test]
    fn malformed_version_and_identity_are_fail_closed() {
        let malformed = inventory(Product::Cli, "2.1");
        assert_eq!(
            evaluate_installation(&malformed, &manifest()).state,
            InstallState::Unknown
        );

        let mut bad_identity = inventory(Product::Cli, "2.1.226");
        bad_identity.candidates[0].identity = IdentityState::Corrupt;
        assert_eq!(
            evaluate_installation(&bad_identity, &manifest()).state,
            InstallState::Corrupt
        );
    }
}
