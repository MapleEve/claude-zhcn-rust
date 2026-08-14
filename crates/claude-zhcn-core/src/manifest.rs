use std::collections::BTreeSet;

use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{Platform, Product};

const SUPPORTED_SCHEMA_VERSION: u32 = 1;
const REQUIRED_PLATFORMS: [Platform; 3] = [
    Platform::MacosArm64,
    Platform::MacosX86_64,
    Platform::WindowsX86_64,
];

/// The supported release channel or native installer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    NativeBinary,
    Dmg,
    Msix,
}

/// Archive format used to distribute an official Claude CLI native binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveFormat {
    TarGz,
    Zip,
}

/// Claude Desktop has one Universal macOS artifact and one Windows x64 MSIX.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DesktopArtifactPlatform {
    #[serde(rename = "macos-universal")]
    MacosUniversal,
    #[serde(rename = "windows-x86_64")]
    WindowsX86_64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MacosArchitecture {
    #[serde(rename = "aarch64")]
    Aarch64,
    #[serde(rename = "x86_64")]
    X86_64,
}

/// One version-pinned official artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactManifest {
    pub platform: Platform,
    pub kind: ArtifactKind,
    pub archive_format: ArchiveFormat,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    /// Platform identity such as a bundle ID or AppX package identity.
    pub identity: String,
    /// Expected publisher/team identity. This is not a replacement for runtime
    /// signature verification.
    pub publisher: String,
}

/// A Desktop distribution artifact. The Universal DMG explicitly declares its
/// two slices instead of duplicating the same DMG for two host platforms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopArtifactManifest {
    pub platform: DesktopArtifactPlatform,
    pub kind: ArtifactKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architectures: Option<Vec<MacosArchitecture>>,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub identity: String,
    pub publisher: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClaudeCliRelease {
    pub version: String,
    pub resource_sha256: String,
    pub patch_profile: String,
    pub artifacts: Vec<ArtifactManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClaudeDesktopRelease {
    pub version: String,
    pub windows_appx_version: String,
    pub release: String,
    pub build: String,
    pub commit_timestamp: String,
    pub resource_sha256: String,
    pub patch_profile: String,
    pub artifacts: Vec<DesktopArtifactManifest>,
}

/// A single exact Claude CLI/Desktop release pair compiled into a tool release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SupportedReleaseManifest {
    pub schema_version: u32,
    pub localizer_version: String,
    pub claude_cli: ClaudeCliRelease,
    pub claude_desktop: ClaudeDesktopRelease,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest JSON is invalid: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unsupported schemaVersion {actual}; expected {expected}")]
    UnsupportedSchema { actual: u32, expected: u32 },
    #[error("{field} is empty")]
    Empty { field: String },
    #[error("{field} contains a placeholder")]
    Placeholder { field: String },
    #[error("{field} has invalid version {value:?}")]
    InvalidVersion { field: String, value: String },
    #[error("{field} must be a lowercase 64-character SHA-256")]
    InvalidSha256 { field: String },
    #[error("{field} must be greater than zero")]
    InvalidSize { field: String },
    #[error("{field} has invalid URL {value:?}: {reason}")]
    InvalidUrl {
        field: String,
        value: String,
        reason: String,
    },
    #[error("{product:?} contains duplicate artifact platform {platform:?}")]
    DuplicatePlatform {
        product: Product,
        platform: Platform,
    },
    #[error("Desktop contains duplicate artifact platform {platform:?}")]
    DuplicateDesktopPlatform { platform: DesktopArtifactPlatform },
    #[error("{product:?} artifact platforms do not exactly match the supported targets")]
    PlatformSet { product: Product },
    #[error("Desktop artifact platforms must be exactly macos-universal and windows-x86_64")]
    DesktopPlatformSet,
    #[error("the macos-universal DMG must contain exactly aarch64 and x86_64")]
    MacosUniversalArchitectures,
    #[error("{field} is only valid for macos-universal")]
    UnexpectedArchitectures { field: String },
    #[error("{field} has the wrong artifact kind")]
    ArtifactKind { field: String },
    #[error("{field} has the wrong archive format for its platform")]
    ArchiveFormat { field: String },
    #[error("{field} is not a lowercase hexadecimal release identity of length {length}")]
    InvalidHexIdentity { field: String, length: usize },
    #[error("claudeDesktop.build must be a prefix of claudeDesktop.release")]
    BuildMismatch,
}

impl SupportedReleaseManifest {
    /// Deserialize and validate without accepting unknown fields or enum values.
    pub fn parse_and_validate(json: &str) -> Result<Self, ManifestError> {
        let manifest: Self = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Fail closed unless every release identity is concrete and all three
    /// supported platforms are present exactly once for both products.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchema {
                actual: self.schema_version,
                expected: SUPPORTED_SCHEMA_VERSION,
            });
        }

        validate_semver("localizerVersion", &self.localizer_version)?;
        validate_dotted_version("claudeCli.version", &self.claude_cli.version, 3)?;
        validate_dotted_version("claudeDesktop.version", &self.claude_desktop.version, 3)?;
        validate_dotted_version(
            "claudeDesktop.windowsAppxVersion",
            &self.claude_desktop.windows_appx_version,
            4,
        )?;

        validate_sha256("claudeCli.resourceSha256", &self.claude_cli.resource_sha256)?;
        validate_sha256(
            "claudeDesktop.resourceSha256",
            &self.claude_desktop.resource_sha256,
        )?;
        validate_text("claudeCli.patchProfile", &self.claude_cli.patch_profile)?;
        validate_text(
            "claudeDesktop.patchProfile",
            &self.claude_desktop.patch_profile,
        )?;
        validate_hex_identity("claudeDesktop.release", &self.claude_desktop.release, 40)?;
        validate_hex_identity("claudeDesktop.build", &self.claude_desktop.build, 6)?;
        if !self
            .claude_desktop
            .release
            .starts_with(&self.claude_desktop.build)
        {
            return Err(ManifestError::BuildMismatch);
        }
        validate_text(
            "claudeDesktop.commitTimestamp",
            &self.claude_desktop.commit_timestamp,
        )?;

        validate_artifacts(
            Product::Cli,
            &self.claude_cli.artifacts,
            &self.claude_cli.version,
            None,
        )?;
        validate_desktop_artifacts(
            &self.claude_desktop.artifacts,
            &self.claude_desktop.version,
            &self.claude_desktop.release,
        )?;
        Ok(())
    }

    #[must_use]
    pub fn supported_version(&self, product: Product, platform: Platform) -> &str {
        match (product, platform) {
            (Product::Cli, _) => &self.claude_cli.version,
            (Product::Desktop, Platform::WindowsX86_64) => {
                &self.claude_desktop.windows_appx_version
            }
            (Product::Desktop, Platform::MacosArm64 | Platform::MacosX86_64) => {
                &self.claude_desktop.version
            }
        }
    }
}

fn validate_artifacts(
    product: Product,
    artifacts: &[ArtifactManifest],
    version: &str,
    release: Option<&str>,
) -> Result<(), ManifestError> {
    let mut platforms = BTreeSet::new();
    for (index, artifact) in artifacts.iter().enumerate() {
        if !platforms.insert(artifact.platform) {
            return Err(ManifestError::DuplicatePlatform {
                product,
                platform: artifact.platform,
            });
        }

        let field = format!("{product:?}.artifacts[{index}]");
        if artifact.kind != ArtifactKind::NativeBinary {
            return Err(ManifestError::ArtifactKind {
                field: format!("{field}.kind"),
            });
        }
        let expected_archive_format = match artifact.platform {
            Platform::MacosArm64 | Platform::MacosX86_64 => ArchiveFormat::TarGz,
            Platform::WindowsX86_64 => ArchiveFormat::Zip,
        };
        if artifact.archive_format != expected_archive_format {
            return Err(ManifestError::ArchiveFormat {
                field: format!("{field}.archiveFormat"),
            });
        }
        validate_url(&format!("{field}.url"), &artifact.url, version, release)?;
        if artifact.size == 0 {
            return Err(ManifestError::InvalidSize {
                field: format!("{field}.size"),
            });
        }
        validate_sha256(&format!("{field}.sha256"), &artifact.sha256)?;
        validate_text(&format!("{field}.identity"), &artifact.identity)?;
        validate_text(&format!("{field}.publisher"), &artifact.publisher)?;
    }

    let required = BTreeSet::from(REQUIRED_PLATFORMS);
    if platforms != required {
        return Err(ManifestError::PlatformSet { product });
    }
    Ok(())
}

fn validate_desktop_artifacts(
    artifacts: &[DesktopArtifactManifest],
    version: &str,
    release: &str,
) -> Result<(), ManifestError> {
    let mut platforms = BTreeSet::new();
    for (index, artifact) in artifacts.iter().enumerate() {
        if !platforms.insert(artifact.platform) {
            return Err(ManifestError::DuplicateDesktopPlatform {
                platform: artifact.platform,
            });
        }
        let field = format!("Desktop.artifacts[{index}]");
        match artifact.platform {
            DesktopArtifactPlatform::MacosUniversal => {
                if artifact.kind != ArtifactKind::Dmg {
                    return Err(ManifestError::ArtifactKind {
                        field: format!("{field}.kind"),
                    });
                }
                let architectures = artifact.architectures.as_deref().unwrap_or_default();
                let actual: BTreeSet<_> = architectures.iter().copied().collect();
                let expected =
                    BTreeSet::from([MacosArchitecture::Aarch64, MacosArchitecture::X86_64]);
                if architectures.len() != 2 || actual != expected {
                    return Err(ManifestError::MacosUniversalArchitectures);
                }
            }
            DesktopArtifactPlatform::WindowsX86_64 => {
                if artifact.kind != ArtifactKind::Msix {
                    return Err(ManifestError::ArtifactKind {
                        field: format!("{field}.kind"),
                    });
                }
                if artifact.architectures.is_some() {
                    return Err(ManifestError::UnexpectedArchitectures {
                        field: format!("{field}.architectures"),
                    });
                }
            }
        }
        validate_url(
            &format!("{field}.url"),
            &artifact.url,
            version,
            Some(release),
        )?;
        if artifact.size == 0 {
            return Err(ManifestError::InvalidSize {
                field: format!("{field}.size"),
            });
        }
        validate_sha256(&format!("{field}.sha256"), &artifact.sha256)?;
        validate_text(&format!("{field}.identity"), &artifact.identity)?;
        validate_text(&format!("{field}.publisher"), &artifact.publisher)?;
    }

    let expected = BTreeSet::from([
        DesktopArtifactPlatform::MacosUniversal,
        DesktopArtifactPlatform::WindowsX86_64,
    ]);
    if platforms != expected {
        return Err(ManifestError::DesktopPlatformSet);
    }
    Ok(())
}

fn validate_url(
    field: &str,
    value: &str,
    version: &str,
    release: Option<&str>,
) -> Result<(), ManifestError> {
    validate_text(field, value)?;
    let parsed = Url::parse(value).map_err(|error| ManifestError::InvalidUrl {
        field: field.to_owned(),
        value: value.to_owned(),
        reason: error.to_string(),
    })?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() || !parsed.username().is_empty() {
        return Err(ManifestError::InvalidUrl {
            field: field.to_owned(),
            value: value.to_owned(),
            reason: "must be an HTTPS URL without embedded credentials".to_owned(),
        });
    }
    let normalized = value.to_ascii_lowercase();
    if normalized
        .split(['/', '?', '&', '='])
        .any(|part| part == "latest")
    {
        return Err(ManifestError::InvalidUrl {
            field: field.to_owned(),
            value: value.to_owned(),
            reason: "must not select latest at runtime".to_owned(),
        });
    }
    if !value.contains(version) && release.is_none_or(|release| !value.contains(release)) {
        return Err(ManifestError::InvalidUrl {
            field: field.to_owned(),
            value: value.to_owned(),
            reason: "must contain the exact supported version or release identity".to_owned(),
        });
    }
    Ok(())
}

fn validate_semver(field: &str, value: &str) -> Result<(), ManifestError> {
    validate_text(field, value)?;
    Version::parse(value).map_err(|_| ManifestError::InvalidVersion {
        field: field.to_owned(),
        value: value.to_owned(),
    })?;
    Ok(())
}

fn validate_dotted_version(field: &str, value: &str, segments: usize) -> Result<(), ManifestError> {
    validate_text(field, value)?;
    let parts: Vec<_> = value.split('.').collect();
    let valid = parts.len() == segments
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == &"0" || !part.starts_with('0'))
                && part.parse::<u64>().is_ok()
        });
    if valid {
        Ok(())
    } else {
        Err(ManifestError::InvalidVersion {
            field: field.to_owned(),
            value: value.to_owned(),
        })
    }
}

fn validate_sha256(field: &str, value: &str) -> Result<(), ManifestError> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value.bytes().any(|byte| byte != b'0');
    if valid {
        Ok(())
    } else {
        Err(ManifestError::InvalidSha256 {
            field: field.to_owned(),
        })
    }
}

fn validate_hex_identity(field: &str, value: &str, length: usize) -> Result<(), ManifestError> {
    let valid = value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value.bytes().any(|byte| byte != b'0');
    if valid {
        Ok(())
    } else {
        Err(ManifestError::InvalidHexIdentity {
            field: field.to_owned(),
            length,
        })
    }
}

fn validate_text(field: &str, value: &str) -> Result<(), ManifestError> {
    if value.trim().is_empty() {
        return Err(ManifestError::Empty {
            field: field.to_owned(),
        });
    }
    if contains_placeholder(value) {
        return Err(ManifestError::Placeholder {
            field: field.to_owned(),
        });
    }
    Ok(())
}

fn contains_placeholder(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    value.contains('<')
        || value.contains('>')
        || normalized.contains("placeholder")
        || normalized.contains("changeme")
        || normalized.contains("example.com")
        || normalized.starts_with("todo")
        || matches!(normalized.as_str(), "todo" | "tbd" | "unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

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
            url: format!("https://downloads.anthropic.test/{version}/{platform:?}"),
            size: 42,
            sha256: hash('a'),
            identity: "official-product-identity".to_owned(),
            publisher: "official-publisher".to_owned(),
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
            url: format!("https://downloads.anthropic.test/{version}/{platform:?}"),
            size: 42,
            sha256: hash('a'),
            identity: "official-product-identity".to_owned(),
            publisher: "official-publisher".to_owned(),
        }
    }

    fn valid_manifest() -> SupportedReleaseManifest {
        SupportedReleaseManifest {
            schema_version: 1,
            localizer_version: "0.1.0".to_owned(),
            claude_cli: ClaudeCliRelease {
                version: "2.1.226".to_owned(),
                resource_sha256: hash('b'),
                patch_profile: "cli-2.1.226-ui-only".to_owned(),
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
                patch_profile: "desktop-1.28929.0-ui-only".to_owned(),
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

    #[test]
    fn accepts_one_exact_complete_release_pair() {
        assert!(valid_manifest().validate().is_ok());
    }

    #[test]
    fn rejects_placeholder_values() {
        let mut manifest = valid_manifest();
        manifest.claude_cli.patch_profile = "<profile-id>".to_owned();
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::Placeholder { .. })
        ));

        let mut blocked = valid_manifest();
        blocked.claude_desktop.artifacts[0].identity = "TODO_BLOCKED".to_owned();
        assert!(matches!(
            blocked.validate(),
            Err(ManifestError::Placeholder { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_platforms() {
        let mut manifest = valid_manifest();
        manifest.claude_cli.artifacts[1].platform = Platform::MacosArm64;
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::DuplicatePlatform { .. })
        ));
    }

    #[test]
    fn rejects_zero_size_and_malformed_hash() {
        let mut manifest = valid_manifest();
        manifest.claude_desktop.artifacts[0].size = 0;
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::InvalidSize { .. })
        ));

        let mut manifest = valid_manifest();
        manifest.claude_desktop.artifacts[0].sha256 = "not-a-sha".to_owned();
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::InvalidSha256 { .. })
        ));
    }

    #[test]
    fn rejects_cli_archive_format_that_does_not_match_the_platform() {
        let mut manifest = valid_manifest();
        manifest.claude_cli.artifacts[0].archive_format = ArchiveFormat::Zip;
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::ArchiveFormat { .. })
        ));
    }

    #[test]
    fn rejects_latest_and_unknown_platform_json() {
        let mut manifest = valid_manifest();
        manifest.claude_cli.artifacts[0].url =
            "https://downloads.anthropic.test/latest/2.1.226".to_owned();
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::InvalidUrl { .. })
        ));

        let json = serde_json::to_string(&valid_manifest())
            .expect("valid manifest should serialize")
            .replacen("macos-aarch64", "linux-x86_64", 1);
        assert!(matches!(
            SupportedReleaseManifest::parse_and_validate(&json),
            Err(ManifestError::InvalidJson(_))
        ));
    }

    #[test]
    fn rejects_wrong_version_shape() {
        let mut manifest = valid_manifest();
        manifest.claude_desktop.version = "1.28929".to_owned();
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::InvalidVersion { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_desktop_artifact_and_incomplete_universal_slices() {
        let mut duplicate = valid_manifest();
        duplicate.claude_desktop.artifacts[1].platform = DesktopArtifactPlatform::MacosUniversal;
        assert!(matches!(
            duplicate.validate(),
            Err(ManifestError::DuplicateDesktopPlatform { .. })
        ));

        let mut missing_slice = valid_manifest();
        missing_slice.claude_desktop.artifacts[0].architectures =
            Some(vec![MacosArchitecture::Aarch64]);
        assert!(matches!(
            missing_slice.validate(),
            Err(ManifestError::MacosUniversalArchitectures)
        ));
    }
}
