import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dir, "..");
const expected = "0.1.0";
const failures: string[] = [];

function readJson(path: string): Record<string, unknown> {
  if (!existsSync(path)) {
    failures.push(`${path.slice(root.length + 1)} is missing`);
    return {};
  }
  try {
    return JSON.parse(readFileSync(path, "utf8")) as Record<string, unknown>;
  } catch (error) {
    failures.push(`${path.slice(root.length + 1)} is not valid JSON: ${String(error)}`);
    return {};
  }
}

function expectVersion(label: string, actual: unknown): void {
  if (actual !== expected) {
    failures.push(`${label}: expected ${expected}, got ${String(actual)}`);
  }
}

const packageJson = readJson(resolve(root, "package.json"));
const tauriConfig = readJson(resolve(root, "src-tauri/tauri.conf.json"));
expectVersion("package.json", packageJson.version);
expectVersion("src-tauri/tauri.conf.json", tauriConfig.version);

const rootCargoPath = resolve(root, "Cargo.toml");
if (!existsSync(rootCargoPath)) {
  failures.push("Cargo.toml is missing");
} else {
  const rootCargo = readFileSync(rootCargoPath, "utf8");
  const workspacePackage = rootCargo.match(/\[workspace\.package\]([\s\S]*?)(?=\n\[|$)/)?.[1];
  const workspaceVersion = workspacePackage?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  expectVersion("Cargo.toml [workspace.package]", workspaceVersion);
}

const manifestList = spawnSync("git", ["ls-files", "--cached", "--others", "--exclude-standard", "*Cargo.toml"], {
  cwd: root,
  encoding: "utf8",
});
if (manifestList.status !== 0) {
  failures.push(manifestList.stderr.trim() || "Unable to enumerate Cargo manifests");
} else {
  const manifests = manifestList.stdout.split("\n").filter(Boolean);
  if (manifests.length === 0) {
    failures.push("no Cargo.toml files found");
  }
  for (const manifest of manifests) {
    const contents = readFileSync(resolve(root, manifest), "utf8");
    const packageSection = contents.match(/\[package\]([\s\S]*?)(?=\n\[|$)/)?.[1];
    if (!packageSection) {
      continue;
    }
    const directVersion = packageSection.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
    const workspaceVersion = /^version\.workspace\s*=\s*true/m.test(packageSection);
    if (directVersion && directVersion !== expected) {
      failures.push(`${manifest}: expected package version ${expected}, got ${directVersion}`);
    } else if (!directVersion && !workspaceVersion) {
      failures.push(`${manifest}: package version is missing`);
    }
  }
}

for (const readme of ["README.md", "README.en.md"]) {
  const path = resolve(root, readme);
  if (!existsSync(path) || !readFileSync(path, "utf8").includes(`\`${expected}\``)) {
    failures.push(`${readme}: scaffold version ${expected} is not documented`);
  }
}

if (failures.length > 0) {
  console.error("Version consistency check failed:\n" + failures.map((item) => `- ${item}`).join("\n"));
  process.exit(1);
}

console.log(`Version consistency check passed (${expected}).`);
