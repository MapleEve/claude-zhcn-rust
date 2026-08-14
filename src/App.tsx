import { useState } from "react";
import { platformSummary, supportedPlatforms } from "./lib/platforms";
import {
  checkForUpdate,
  installUpdate,
  type UpdateCheckResult,
} from "./services/updater";

type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "available"; result: Extract<UpdateCheckResult, { kind: "available" }> }
  | { kind: "installing" }
  | { kind: "error"; message: string };

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export default function App() {
  const [updateState, setUpdateState] = useState<UpdateState>({ kind: "idle" });

  async function handleCheckUpdate() {
    setUpdateState({ kind: "checking" });
    try {
      const result = await checkForUpdate();
      setUpdateState(
        result.kind === "available"
          ? { kind: "available", result }
          : { kind: "current" },
      );
    } catch (error) {
      setUpdateState({ kind: "error", message: errorMessage(error) });
    }
  }

  async function handleInstallUpdate() {
    if (updateState.kind !== "available") return;
    const { update } = updateState.result;
    setUpdateState({ kind: "installing" });
    try {
      await installUpdate(update);
    } catch (error) {
      setUpdateState({ kind: "error", message: errorMessage(error) });
    }
  }

  return (
    <main className="shell">
      <section className="hero">
        <div className="eyebrow"><span className="dot" /> 项目基座</div>
        <h1>Claude 中文化工具</h1>
        <p className="lede">
          Rust 核心、命令行与桌面端的统一入口。目前仅完成工程基座，尚未执行本机系统探测。
        </p>
        <div className="notice" role="status">
          <span className="notice-icon">i</span>
          <div>
            <strong>功能尚未开放</strong>
            <p>安装检测、版本识别、下载与汉化流程仍在实现中，本页面不代表功能可用。</p>
          </div>
        </div>
      </section>

      <section className="panel" aria-labelledby="platform-title">
        <div className="section-heading">
          <div>
            <span className="kicker">首发范围</span>
            <h2 id="platform-title">三平台状态</h2>
          </div>
          <span className="pending-pill">等待探测</span>
        </div>

        <div className="platform-grid">
          {supportedPlatforms.map((platform) => (
            <article className="platform-card" key={platform.id}>
              <div className="platform-mark" aria-hidden="true">
                {platform.label === "macOS" ? "M" : "W"}
              </div>
              <div>
                <h3>{platformSummary(platform)}</h3>
                <p>{platform.packageFormat}</p>
              </div>
              <span className="status">尚未探测</span>
            </article>
          ))}
        </div>
      </section>

      <section className="panel update-panel" aria-labelledby="update-title">
        <div>
          <span className="kicker">应用维护</span>
          <h2 id="update-title">检查工具更新</h2>
          <p className="muted">正式发布构建通过 GitHub Releases 获取签名更新。</p>
          {updateState.kind === "current" && <p className="feedback">当前已是最新版本。</p>}
          {updateState.kind === "error" && (
            <p className="feedback error">无法检查更新：{updateState.message}</p>
          )}
          {updateState.kind === "available" && (
            <p className="feedback">发现版本 {updateState.result.version}。</p>
          )}
        </div>
        {updateState.kind === "available" ? (
          <button type="button" onClick={handleInstallUpdate}>下载并安装</button>
        ) : (
          <button
            type="button"
            onClick={handleCheckUpdate}
            disabled={updateState.kind === "checking" || updateState.kind === "installing"}
          >
            {updateState.kind === "checking" ? "正在检查…" : "检查更新"}
          </button>
        )}
      </section>
    </main>
  );
}
