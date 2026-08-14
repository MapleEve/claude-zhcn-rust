import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const [assetsArgument, outputArgument] = process.argv.slice(2);
const version = process.env.RELEASE_VERSION;
const repository = process.env.GITHUB_REPOSITORY ?? "MapleEve/claude-zhcn-rust";
if (!assetsArgument || !outputArgument || !version) {
  console.error(
    "Usage: RELEASE_VERSION=x.y.z bun scripts/assemble-updater-manifest.ts <assets-dir> <latest.json>",
  );
  process.exit(1);
}

const assets = resolve(assetsArgument);
const output = resolve(outputArgument);
const expectedPlatforms = ["darwin-aarch64", "darwin-x86_64", "windows-x86_64"];
const metadataFiles = readdirSync(assets).filter((name) => name.endsWith(".metadata.json"));
const platformEntries: Record<string, { signature: string; url: string }> = {};

for (const metadataFile of metadataFiles) {
  const metadata = JSON.parse(readFileSync(join(assets, metadataFile), "utf8")) as {
    version: string;
    platform: string;
    updater: string;
    signature: string;
  };
  if (!expectedPlatforms.includes(metadata.platform)) {
    console.error(`Unexpected updater platform: ${metadata.platform}`);
    process.exit(1);
  }
  if (metadata.version !== version || platformEntries[metadata.platform]) {
    console.error(`Invalid or duplicate metadata for ${metadata.platform}`);
    process.exit(1);
  }
  const updaterPath = join(assets, basename(metadata.updater));
  const signaturePath = join(assets, basename(metadata.signature));
  if (!existsSync(updaterPath) || !existsSync(signaturePath)) {
    console.error(`Missing updater archive or signature for ${metadata.platform}`);
    process.exit(1);
  }
  const signature = readFileSync(signaturePath, "utf8").trim();
  if (signature.length < 32) {
    console.error(`Updater signature is empty or invalid for ${metadata.platform}`);
    process.exit(1);
  }
  platformEntries[metadata.platform] = {
    signature,
    url: `https://github.com/${repository}/releases/download/v${version}/${encodeURIComponent(basename(metadata.updater))}`,
  };
}

for (const platform of expectedPlatforms) {
  if (!platformEntries[platform]) {
    console.error(`Updater manifest is incomplete: missing ${platform}`);
    process.exit(1);
  }
}

const manifest = {
  version,
  notes: `claude-zhcn-rust v${version}`,
  pub_date: new Date().toISOString(),
  platforms: Object.fromEntries(expectedPlatforms.map((platform) => [platform, platformEntries[platform]])),
};
writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
console.log(`Assembled complete updater manifest at ${output}.`);
