import { copyFileSync, existsSync, mkdirSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const [target, sourceArgument, outputArgument] = process.argv.slice(2);
const version = process.env.RELEASE_VERSION;
if (!target || !sourceArgument || !outputArgument || !version) {
  console.error(
    "Usage: RELEASE_VERSION=x.y.z bun scripts/collect-release-assets.ts <rust-target> <bundle-root> <output-dir>",
  );
  process.exit(1);
}

const targetMap: Record<string, { updater: string; installer: string }> = {
  "aarch64-apple-darwin": { updater: "darwin-aarch64", installer: ".dmg" },
  "x86_64-apple-darwin": { updater: "darwin-x86_64", installer: ".dmg" },
  "x86_64-pc-windows-msvc": { updater: "windows-x86_64", installer: ".exe" },
};
const platform = targetMap[target];
if (!platform) {
  console.error(`Unsupported release target: ${target}`);
  process.exit(1);
}

const source = resolve(sourceArgument);
const output = resolve(outputArgument);
if (!existsSync(source)) {
  console.error(`Bundle root does not exist: ${source}`);
  process.exit(1);
}

function walk(path: string): string[] {
  return readdirSync(path).flatMap((name) => {
    const child = join(path, name);
    return statSync(child).isDirectory() ? walk(child) : [child];
  });
}

function exactlyOne(label: string, candidates: string[]): string {
  if (candidates.length !== 1) {
    console.error(`${label}: expected exactly one file, found ${candidates.length}`);
    for (const candidate of candidates) console.error(`- ${candidate}`);
    process.exit(1);
  }
  return candidates[0]!;
}

const files = walk(source);
const updaterSuffix = target.includes("apple") ? ".app.tar.gz" : ".nsis.zip";
const updater = exactlyOne(
  "updater archive",
  files.filter((file) => file.endsWith(updaterSuffix)),
);
const signature = exactlyOne(
  "updater signature",
  files.filter((file) => file === `${updater}.sig`),
);
const installers = files.filter((file) => {
  if (platform.installer === ".dmg") return file.endsWith(".dmg");
  return file.endsWith(".exe") && !file.endsWith("-setup.exe.sig");
});
const installer = exactlyOne("native installer", installers);

mkdirSync(output, { recursive: true });
const prefix = `claude-zhcn-rust_${version}_${platform.updater}`;
const updaterName = `${prefix}${updaterSuffix}`;
const signatureName = `${updaterName}.sig`;
const installerName = `${prefix}${platform.installer === ".exe" ? "-setup.exe" : ".dmg"}`;
copyFileSync(updater, join(output, updaterName));
copyFileSync(signature, join(output, signatureName));
copyFileSync(installer, join(output, installerName));

const metadata = {
  version,
  platform: platform.updater,
  updater: updaterName,
  signature: signatureName,
  installer: installerName,
  originalUpdater: basename(updater),
};
writeFileSync(
  join(output, `${platform.updater}.metadata.json`),
  `${JSON.stringify(metadata, null, 2)}\n`,
  "utf8",
);
console.log(`Collected signed updater and installer for ${platform.updater}.`);
