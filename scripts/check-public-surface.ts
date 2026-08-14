import { readFileSync, statSync } from "node:fs";
import { basename, extname, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dir, "..");
const git = spawnSync(
  "git",
  ["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
  { cwd: root, encoding: "utf8" },
);

if (git.status !== 0) {
  console.error(git.stderr.trim() || "Unable to enumerate the public surface.");
  process.exit(1);
}

const files = git.stdout.split("\0").filter(Boolean);
const forbiddenNames = new Set(["AGENTS.local.md", ".env"]);
const forbiddenExtensions = new Set([
  ".appx",
  ".asar",
  ".dmg",
  ".exe",
  ".key",
  ".msix",
  ".pem",
  ".pkg",
]);
const internalRepositoryName = ["claude", "zhcn", "rust", "internal"].join("-");
const internalProductTerms = [
  ["Pol", "aris"].join(""),
  ["Code", "up"].join(""),
];
const internalProductToken = new RegExp(
  ["(^|[^A-Za-z0-9])", "C", "5", "([^A-Za-z0-9]|$)"].join(""),
);
const machinePaths = [
  ["/", "Users", "/"].join(""),
  ["/", "home", "/"].join(""),
  ["Documents", "/", "GitHub"].join(""),
  [".", "codex", "/"].join(""),
  ["C:", "\\", "Users", "\\"].join(""),
];
const privateKeyHeader = ["-----BEGIN ", "PRIVATE KEY-----"].join("");
const highConfidenceSecrets = [
  new RegExp(["gh", "p_", "[A-Za-z0-9]{30,}"].join(""), "g"),
  new RegExp(["github", "_pat_", "[A-Za-z0-9_]{40,}"].join(""), "g"),
  new RegExp(["AKIA", "[A-Z0-9]{16}"].join(""), "g"),
];

const failures: string[] = [];

for (const file of files) {
  const path = resolve(root, file);
  const name = basename(file);
  const extension = extname(name).toLowerCase();

  if (
    forbiddenNames.has(name) ||
    (name.startsWith(".env.") && name !== ".env.example") ||
    forbiddenExtensions.has(extension)
  ) {
    failures.push(`${file}: forbidden sensitive filename`);
    continue;
  }

  if (!statSync(path).isFile()) {
    continue;
  }
  if (statSync(path).size > 2_000_000) {
    failures.push(`${file}: exceeds the public source-file size limit`);
    continue;
  }

  const bytes = readFileSync(path);
  if (bytes.includes(0)) {
    continue;
  }
  const text = bytes.toString("utf8");
  const pathSafeText = text
    .replaceAll(["C:", "/", "Users", "/", "test", "/"].join(""), "")
    .replaceAll(["C:", "\\", "Users", "\\", "test", "\\"].join(""), "");
  const displayPath = relative(root, path);

  if (text.includes(internalRepositoryName)) {
    failures.push(`${displayPath}: contains a private repository identifier`);
  }
  for (const term of internalProductTerms) {
    if (text.includes(term)) {
      failures.push(`${displayPath}: contains an internal product identifier`);
    }
  }
  if (internalProductToken.test(text)) {
    failures.push(`${displayPath}: contains an internal product identifier`);
  }
  for (const prefix of machinePaths) {
    if (pathSafeText.includes(prefix)) {
      failures.push(`${displayPath}: contains a machine-specific absolute path`);
    }
  }
  if (text.includes(privateKeyHeader)) {
    failures.push(`${displayPath}: contains a private-key header`);
  }
  for (const pattern of highConfidenceSecrets) {
    pattern.lastIndex = 0;
    if (pattern.test(text)) {
      failures.push(`${displayPath}: contains a high-confidence credential pattern`);
    }
  }
}

if (failures.length > 0) {
  console.error("Public surface check failed:\n" + failures.map((item) => `- ${item}`).join("\n"));
  process.exit(1);
}

console.log(`Public surface check passed (${files.length} files).`);
