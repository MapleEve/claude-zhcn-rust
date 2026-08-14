use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_discloses_read_only_bootstrap_scope() {
    Command::cargo_bin("claude-zhcn")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Read-only bootstrap CLI"));
}

#[test]
fn validate_manifest_fails_closed_on_placeholder_json() {
    let directory = tempfile::tempdir().expect("temporary directory should exist");
    let manifest = directory.path().join("supported-release.json");
    fs::write(
        &manifest,
        r#"{
          "schemaVersion": 1,
          "localizerVersion": "0.1.0",
          "claudeCli": {
            "version": "2.1.226",
            "resourceSha256": "<sha256>",
            "patchProfile": "<profile>",
            "artifacts": []
          },
          "claudeDesktop": {
            "version": "1.28929.0",
            "windowsAppxVersion": "1.28929.0.0",
            "release": "d1a6bcd4ef8627d603a8290548a984220b6701cf",
            "build": "d1a6bc",
            "commitTimestamp": "2026-08-11T18:58:06.000Z",
            "resourceSha256": "<sha256>",
            "patchProfile": "<profile>",
            "artifacts": []
          }
        }"#,
    )
    .expect("fixture should be written");

    Command::cargo_bin("claude-zhcn")
        .expect("binary should build")
        .args(["validate-manifest", manifest.to_str().expect("UTF-8 path")])
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest was rejected"));
}

#[test]
fn status_evaluates_a_supplied_exact_inventory_without_system_discovery() {
    let directory = tempfile::tempdir().expect("temporary directory should exist");
    let manifest = directory.path().join("supported-release.json");
    let inventory = directory.path().join("inventory.json");
    let hash = "a".repeat(64);
    let resource_hash = "b".repeat(64);

    let cli_artifact = |platform: &str| {
        serde_json::json!({
            "platform": platform,
            "kind": "native-binary",
            "archiveFormat": if platform.starts_with("macos-") { "tar-gz" } else { "zip" },
            "url": format!("https://downloads.anthropic.test/2.1.226/{platform}"),
            "size": 1,
            "sha256": hash,
            "identity": "official-cli",
            "publisher": "official-publisher"
        })
    };
    let manifest_json = serde_json::json!({
        "schemaVersion": 1,
        "localizerVersion": "0.1.0",
        "claudeCli": {
            "version": "2.1.226",
            "resourceSha256": resource_hash,
            "patchProfile": "cli-ui-only",
            "artifacts": [
                cli_artifact("macos-aarch64"),
                cli_artifact("macos-x86_64"),
                cli_artifact("windows-x86_64")
            ]
        },
        "claudeDesktop": {
            "version": "1.28929.0",
            "windowsAppxVersion": "1.28929.0.0",
            "release": "d1a6bcd4ef8627d603a8290548a984220b6701cf",
            "build": "d1a6bc",
            "commitTimestamp": "2026-08-11T18:58:06.000Z",
            "resourceSha256": "c".repeat(64),
            "patchProfile": "desktop-ui-only",
            "artifacts": [
                {
                    "platform": "macos-universal",
                    "kind": "dmg",
                    "architectures": ["aarch64", "x86_64"],
                    "url": "https://downloads.anthropic.test/1.28929.0/Claude.dmg",
                    "size": 1,
                    "sha256": "d".repeat(64),
                    "identity": "com.anthropic.claudefordesktop",
                    "publisher": "official-apple-team"
                },
                {
                    "platform": "windows-x86_64",
                    "kind": "msix",
                    "url": "https://downloads.anthropic.test/1.28929.0/Claude.msix",
                    "size": 1,
                    "sha256": "e".repeat(64),
                    "identity": "official-appx-identity",
                    "publisher": "official-windows-publisher"
                }
            ]
        }
    });
    fs::write(
        &manifest,
        serde_json::to_vec_pretty(&manifest_json).expect("manifest should serialize"),
    )
    .expect("manifest should be written");
    fs::write(
        &inventory,
        serde_json::to_vec_pretty(&serde_json::json!({
            "product": "cli",
            "platform": "macos-aarch64",
            "running": false,
            "candidates": [{
                "canonicalPath": "/usr/local/bin/claude",
                "installKind": "cli-native",
                "version": "2.1.226",
                "identity": "verified",
                "patch": "clean"
            }]
        }))
        .expect("inventory should serialize"),
    )
    .expect("inventory should be written");

    Command::cargo_bin("claude-zhcn")
        .expect("binary should build")
        .args([
            "status",
            "--input",
            inventory.to_str().expect("UTF-8 inventory path"),
            "--manifest",
            manifest.to_str().expect("UTF-8 manifest path"),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"state\": \"ExactClean\""))
        .stdout(predicate::str::contains("\"writeAllowed\": true"));
}
