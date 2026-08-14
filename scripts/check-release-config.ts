import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const releaseMode = process.argv.includes("--release");
const failures: string[] = [];
const notices: string[] = [];
const expectedEndpoint =
  "https://github.com/MapleEve/claude-zhcn-rust/releases/latest/download/latest.json";
const blockedPublicKey = "BLOCKED_UNTIL_A_REAL_TAURI_UPDATER_PUBLIC_KEY_IS_CONFIGURED";

function readJson(path: string): Record<string, any> {
  if (!existsSync(path)) {
    failures.push(`${path.slice(root.length + 1)} is missing`);
    return {};
  }
  try {
    return JSON.parse(readFileSync(path, "utf8")) as Record<string, any>;
  } catch (error) {
    failures.push(`${path.slice(root.length + 1)} is not valid JSON: ${String(error)}`);
    return {};
  }
}

function isUpdaterPublicKey(value: unknown): value is string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length % 4 !== 0 ||
    !/^[A-Za-z0-9+/]+={0,2}$/.test(value)
  ) {
    return false;
  }

  const decoded = Buffer.from(value, "base64");
  if (decoded.toString("base64") !== value) {
    return false;
  }
  const lines = decoded.toString("utf8").trimEnd().split(/\r?\n/);
  return (
    lines.length === 2 &&
    /^untrusted comment: .+/.test(lines[0] ?? "") &&
    /^RW[A-Za-z0-9+/=]{30,}$/.test(lines[1] ?? "")
  );
}

const releaseConfig = readJson(resolve(root, "src-tauri/tauri.release.conf.json"));
const endpoint = releaseConfig.plugins?.updater?.endpoints;
const publicKey = releaseConfig.plugins?.updater?.pubkey;

if (releaseConfig.bundle?.createUpdaterArtifacts !== true) {
  failures.push("release config must enable bundle.createUpdaterArtifacts");
}
if (!Array.isArray(endpoint) || endpoint.length !== 1 || endpoint[0] !== expectedEndpoint) {
  failures.push("release config must use the single stable GitHub Release updater endpoint");
}

const productionManifestPath = resolve(root, "resources/supported-release.json");
let productionManifest: Record<string, any> | undefined;
if (existsSync(productionManifestPath)) {
  productionManifest = readJson(productionManifestPath);
  if (readFileSync(productionManifestPath, "utf8").includes("TODO_BLOCKED")) {
    failures.push("production supported-release manifest contains TODO_BLOCKED");
  }
  const packageJson = readJson(resolve(root, "package.json"));
  if (productionManifest.schemaVersion !== 1) {
    failures.push("production supported-release manifest must use schemaVersion=1");
  }
  if (productionManifest.localizerVersion !== packageJson.version) {
    failures.push("production manifest localizerVersion must equal the project version");
  }
  if (productionManifest.claudeCli?.version !== "2.1.226") {
    failures.push("production manifest must pin Claude CLI 2.1.226 exactly");
  }
  if (
    productionManifest.claudeDesktop?.version !== "1.28929.0" ||
    productionManifest.claudeDesktop?.windowsAppxVersion !== "1.28929.0.0"
  ) {
    failures.push("production manifest must pin the exact Desktop and Windows AppX versions");
  }
} else if (releaseMode) {
  failures.push("production supported-release manifest is missing");
} else {
  notices.push("production supported-release manifest is intentionally absent");
}

if (releaseMode) {
  if (
    publicKey === blockedPublicKey ||
    !isUpdaterPublicKey(publicKey)
  ) {
    failures.push("a real Tauri updater public key is required for release");
  }
  if ((process.env.TAURI_SIGNING_PRIVATE_KEY ?? "").length < 64) {
    failures.push("TAURI_SIGNING_PRIVATE_KEY is missing or invalid");
  }
  if ((process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?? "").length === 0) {
    failures.push("TAURI_SIGNING_PRIVATE_KEY_PASSWORD is missing");
  }

  const packageJson = readJson(resolve(root, "package.json"));
  const expectedTag = `v${String(packageJson.version ?? "")}`;
  if (process.env.RELEASE_TAG !== expectedTag) {
    failures.push(`RELEASE_TAG must exactly equal ${expectedTag}`);
  }
} else if (publicKey === blockedPublicKey) {
  notices.push("updater public key is intentionally blocked; manual release remains disabled");
} else if (typeof publicKey !== "string" || publicKey.length < 32) {
  failures.push("updater public key must be explicitly blocked or fully configured");
}

for (const notice of notices) {
  console.log(`NOTICE: ${notice}.`);
}
if (failures.length > 0) {
  console.error("Release configuration check failed:\n" + failures.map((item) => `- ${item}`).join("\n"));
  process.exit(1);
}

console.log(`Release configuration check passed (${releaseMode ? "release" : "scaffold"} mode).`);
