import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateCheckResult =
  | { kind: "available"; update: Update; version: string }
  | { kind: "current" };

export async function checkForUpdate(): Promise<UpdateCheckResult> {
  const update = await check();

  if (!update) {
    return { kind: "current" };
  }

  return { kind: "available", update, version: update.version };
}

export async function installUpdate(update: Update): Promise<void> {
  await update.downloadAndInstall();
  await relaunch();
}
