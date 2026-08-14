<sub>🌐 <a href="README.md">中文</a> · <b>English</b></sub>

<div align="center">

# claude-zhcn-rust 🀄

> *“Touch only a Claude release we can identify exactly; stop everywhere else.”*

<a href="https://github.com/MapleEve/claude-zhcn-rust/actions/workflows/ci.yml">
  <img src="https://img.shields.io/github/actions/workflow/status/MapleEve/claude-zhcn-rust/ci.yml?branch=main&style=flat-square" alt="CI" />
</a>
<a href="https://github.com/MapleEve/claude-zhcn-rust/releases">
  <img src="https://img.shields.io/github/v/release/MapleEve/claude-zhcn-rust?style=flat-square" alt="Release" />
</a>

<br>

A Chinese localization tool scaffold for Claude Code CLI and Claude Desktop.<br>
One Rust core powers both a command-line interface and a Tauri desktop app.<br>
An exact release manifest defines both what is safe and what must be refused.<br>

<br>

<a href="#get-started">Get started</a> ·
<a href="#two-interfaces--feature-scope">Feature scope</a> ·
<a href="#how-it-works">How it works</a> ·
<a href="#documentation">Documentation</a>

</div>

---

## Have you hit this

> Claude has just updated. Reusing yesterday's patch rules might work—or silently damage a new build—and you do not want a tool to guess.

> On Windows, MSIX, Squirrel, and a setup bootstrapper can all appear in the same ecosystem. Finding a path does not make that installation safe to modify.

claude-zhcn-rust is built around that boundary. **It targets one manifest-pinned planned release pair and explicitly refuses ambiguous state.**

> [!IMPORTANT]
> This repository is only a `0.1.0` engineering scaffold. Real system discovery, official downloads, installation, localization, backup, and recovery are not implemented. Do not use it to modify an installed application.

## Get started

For now, you can clone the source and run its engineering gates:

```bash
git clone https://github.com/MapleEve/claude-zhcn-rust.git
cd claude-zhcn-rust
bun install --frozen-lockfile
cargo test --workspace
bun run check
```

## Two interfaces / Feature scope

One Rust core is planned to serve two interfaces. None of the product operations below are complete in the `0.1.0` scaffold:

- **CLI**: intended for automation and terminal users. Only the official native Claude Code CLI will ever be writable; npm installations are detection-only with a migration prompt.
- **Desktop**: a Tauri 2 + React interface intended to expose the same state machine, visible confirmation, and recovery path.
- **Current-only**: the project currently plans to support and pins Claude Code CLI `2.1.226` plus Claude Desktop `1.28929.0` (Windows package version `1.28929.0.0`). A newer official release does not become supported automatically.
- **Three host targets**: macOS arm64, macOS x86_64, and Windows x64. Linux, Windows ARM64, and 32-bit Windows are out of scope.
- **Native install boundary**: active Windows Desktop download/install uses the full MSIX only. Squirrel and the official setup bootstrapper are detected but never modified.
- **Independent self-update**: the tool will eventually update from signed GitHub Release assets. This never changes which Claude release is supported.

## What you get

The scaffold currently provides a reviewable public engineering baseline:

- A strict supported-release JSON Schema and an example explicitly marked `TODO_BLOCKED`; there is no production manifest that could imply unsupported readiness.
- CI skeletons for Rust, frontend, and all three native targets. CI never downloads official Claude packages.
- A manually dispatched, draft-only Release workflow that fails closed when updater signing is not configured.
- Public-surface, version-consistency, and release-configuration checks.

A future write path will open only after artifact identity, digest, install state, and user confirmation all match. File-level Windows localization invalidates the original vendor signature/package trust, so the product must disclose that impact before writing and require real Windows launch and UI acceptance for release.

## How it works

```text
Read exact supported-release manifest
        │
        ▼
Read-only discovery ── multiple / npm / Squirrel / unknown ──▶ refuse writes
        │
        ▼
Exact version + platform + artifact identity
        │
        ▼
Show impact and wait for user confirmation
        │
        ▼
verify download → back up → localize → launch/UI acceptance → recoverable state
```

This is the target behavior, not functionality already present in `0.1.0`.

## Documentation

| 中文 | English |
| --- | --- |
| [中文 README](README.md) | [English README](README.en.md) |
| [安全策略](SECURITY.md) | [Security policy](SECURITY.md) |
| [支持清单 Schema](resources/supported-release.schema.json) | [Supported-release schema](resources/supported-release.schema.json) |
| [阻塞示例清单](resources/supported-release.example.json) | [Blocked example manifest](resources/supported-release.example.json) |
| [清单资源说明](resources/README.md) | [Manifest resource notes](resources/README.md) |

## Contact

- General questions: use [GitHub Issues](https://github.com/MapleEve/claude-zhcn-rust/issues).
- Security issues: follow [SECURITY.md](SECURITY.md) and use private vulnerability reporting.

<a href="https://star-history.com/#MapleEve/claude-zhcn-rust&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=MapleEve/claude-zhcn-rust&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=MapleEve/claude-zhcn-rust&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=MapleEve/claude-zhcn-rust&type=Date" />
  </picture>
</a>

## Contributing & License

No open-source license has been selected. Public visibility is not a license: **no permission is currently granted to copy, modify, or redistribute this code**, and external code contributions are not accepted yet. This section will be updated when a license and contribution policy are chosen. Issue reports are welcome in the meantime.
