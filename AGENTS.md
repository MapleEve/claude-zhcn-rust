# AGENTS.md

## Project boundary

`claude-zhcn-rust` is a Rust core with CLI and Tauri desktop entry points. Keep
the core independent of UI frameworks: the CLI and Tauri command layer may
depend on the core, while the core must not depend on Tauri or React.

This repository is **current-only**. Every operation against Claude Code CLI or
Claude Desktop must be gated by the exact supported-release manifest. Unknown,
older, newer, corrupt, ambiguous, or concurrently installed variants must fail
closed. Never reinterpret `latest` as permission to support an unlisted Claude
release.

## Supported scope

- Host targets: macOS arm64, macOS x86_64, and Windows x64.
- Claude Code CLI: official native binary is the only writable install kind.
  npm installations are detection-only.
- Claude Desktop on Windows: MSIX is the only active download/install path.
  Squirrel and the setup bootstrapper are detection-only.
- Tool self-update is separate from Claude product support and uses signed
  GitHub Release updater artifacts.

Do not add Linux, Windows ARM64, 32-bit Windows, historical-version support,
automatic version guessing, or silent privilege elevation without an explicit
scope decision.

## Public repository safety

- Never commit credentials, signing keys, official application packages,
  extracted application code/assets, private research, machine-specific paths,
  or raw reverse-engineering evidence.
- Keep local configuration in `AGENTS.local.md` or `.env` files; both are
  ignored.
- Do not add a production `resources/supported-release.json` until every URL,
  digest, package identity, and supported platform has been independently
  verified. Example manifests containing `TODO_BLOCKED` must be rejected by
  production validation.
- Run `bun run check:public`, `bun run check:versions`, and
  `bun run check:release-config` before proposing a public change.

## Quality gates

- Rust: `cargo fmt --all -- --check`, Clippy with warnings denied, and workspace
  tests for the core and CLI.
- Frontend: frozen dependency install, type checking, tests, and production
  build.
- Native desktop: compile smoke checks on all three supported host targets.
- Release: manual dispatch only, signed updater artifacts, draft release only,
  and no Claude package downloads in CI.

Keep documentation in `README.md` and `README.en.md` synchronized.
