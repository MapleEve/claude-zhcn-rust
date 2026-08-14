<sub>🌐 <a href="README.en.md">English</a> · <b>中文</b></sub>

<div align="center">

# claude-zhcn-rust 🀄

> *“只对明确支持的 Claude 版本动手，其余情况一律停下。”*

<a href="https://github.com/MapleEve/claude-zhcn-rust/actions/workflows/ci.yml">
  <img src="https://img.shields.io/github/actions/workflow/status/MapleEve/claude-zhcn-rust/ci.yml?branch=main&style=flat-square" alt="CI" />
</a>
<a href="https://github.com/MapleEve/claude-zhcn-rust/releases">
  <img src="https://img.shields.io/github/v/release/MapleEve/claude-zhcn-rust?style=flat-square" alt="Release" />
</a>

<br>

一个面向 Claude Code CLI 与 Claude Desktop 的中文化工具脚手架。<br>
Rust 核心同时服务命令行与 Tauri 桌面端。<br>
精确版本清单把“能处理”和“必须拒绝”写成同一份契约。<br>

<br>

<a href="#开始用">开始用</a> ·
<a href="#两种用法--功能特性">功能特性</a> ·
<a href="#核心流程">核心流程</a> ·
<a href="#文档">文档</a>

</div>

---

## 你是不是遇到过这个

> 官方应用刚更新，你不确定旧规则能否安全复用，也不想让工具猜版本后直接改文件。

> Windows 同时可能出现 MSIX、Squirrel 或安装引导程序；路径找到了，却不代表这套安装能安全写入。

claude-zhcn-rust 解决的就是这个。**只匹配清单锁定的当前版本，并把不确定状态明确拒绝。**

> [!IMPORTANT]
> 当前仓库只是 `0.1.0` 工程脚手架。真实的系统探测、官方下载、安装、汉化、备份与恢复尚未实现，请勿将其用于修改已安装应用。

## 开始用

目前只能检出源码并运行工程质量门：

```bash
git clone https://github.com/MapleEve/claude-zhcn-rust.git
cd claude-zhcn-rust
bun install --frozen-lockfile
cargo test --workspace
bun run check
```

## 两种用法 / 功能特性

计划由同一个 Rust 核心提供两个入口；下列产品能力均未在 `0.1.0` 脚手架中完成：

- **CLI**：面向自动化与终端用户，未来只会写入官方原生 Claude Code CLI；npm 安装仅识别并提示迁移。
- **Desktop**：Tauri 2 + React 图形界面，未来提供相同状态机、可见确认和恢复流程。
- **current-only**：项目当前计划支持并锁定 Claude Code CLI `2.1.226` 与 Claude Desktop `1.28929.0`（Windows 包版本 `1.28929.0.0`）。新的官方版本不会自动变成“受支持”。
- **三平台**：macOS arm64、macOS x86_64、Windows x64。没有 Linux、Windows ARM64 或 32 位 Windows 计划。
- **原生安装边界**：Windows Desktop 主动下载/安装只走完整 MSIX；Squirrel 与官方安装引导程序仅检测，不写入。
- **独立自更新**：本工具未来通过签名的 GitHub Release 资产自更新；这与 Claude 版本支持完全分离。

## 你会得到什么

当前脚手架提供可审查的公开工程基座：

- 严格的 supported-release JSON Schema，以及明确标记 `TODO_BLOCKED` 的示例；不存在可冒充正式支持清单的生产文件。
- Rust、前端和三种原生目标的 CI 骨架；CI 不下载任何 Claude 官方包。
- 仅手动触发、仅创建草稿、缺少更新签名条件即失败的 Release 工作流。
- 公开面、版本一致性和发布配置检查脚本。

未来只有在真实制品身份、散列、安装状态与用户确认全部满足后，写入路径才会开放。Windows 文件级汉化会使原厂签名/包信任失效，产品必须在操作前明确提示，并以真实 Windows 启动与 UI 验收作为发布门。

## 核心流程

```text
读取精确支持清单
        │
        ▼
只读发现安装 ── 多安装 / npm / Squirrel / 未知版本 ──▶ 拒绝写入
        │
        ▼
版本、平台、制品身份全部精确匹配
        │
        ▼
显示影响并等待用户确认
        │
        ▼
下载校验 → 备份 → 汉化 → 启动/UI 验收 → 可恢复
```

上图描述的是目标行为，不代表 `0.1.0` 已实现。

## 文档

| 中文 | English |
| --- | --- |
| [中文 README](README.md) | [English README](README.en.md) |
| [安全策略](SECURITY.md) | [Security policy](SECURITY.md) |
| [支持清单 Schema](resources/supported-release.schema.json) | [Supported-release schema](resources/supported-release.schema.json) |
| [阻塞示例清单](resources/supported-release.example.json) | [Blocked example manifest](resources/supported-release.example.json) |
| [清单资源说明](resources/README.md) | [Manifest resource notes](resources/README.md) |

## 联系

- 一般问题：使用 [GitHub Issues](https://github.com/MapleEve/claude-zhcn-rust/issues)。
- 安全问题：请按 [SECURITY.md](SECURITY.md) 使用私密漏洞报告，不要公开披露。

<a href="https://star-history.com/#MapleEve/claude-zhcn-rust&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=MapleEve/claude-zhcn-rust&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=MapleEve/claude-zhcn-rust&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=MapleEve/claude-zhcn-rust&type=Date" />
  </picture>
</a>

## 贡献 & License

本仓库尚未选择开源许可证。公开可见不等于获得许可：目前**不授予复制、修改或再分发本代码的权限**，也暂不接受外部代码贡献。许可证与贡献政策确定后，本节会同步更新；在此之前可以通过 Issue 提交问题报告。
