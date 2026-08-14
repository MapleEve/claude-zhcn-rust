# Supported-release resources

## 中文

`supported-release.schema.json` 与 Rust 核心的 `SupportedReleaseManifest`
结构保持一致；真正的语义校验以 Rust 的 fail-closed validator 为准。

`supported-release.example.json` 是**故意不可通过验证**的阻塞骨架。
其中 `TODO_BLOCKED` 和 `size: 0` 表示尚未闭合的公开制品身份，不能改成看似真实的占位值。
仓库在所有字段经独立验证前不会提供 `supported-release.json`，Release 工作流也会因此停止。

## English

`supported-release.schema.json` mirrors the Rust core
`SupportedReleaseManifest` structure. The fail-closed Rust validator remains
authoritative for semantic checks.

`supported-release.example.json` is **intentionally invalid**. `TODO_BLOCKED`
and `size: 0` mark artifact identity that has not been independently closed;
they must never be replaced with plausible-looking dummy data. The repository
does not provide `supported-release.json` until every field is verified, and
the Release workflow stops while that production manifest is absent.
