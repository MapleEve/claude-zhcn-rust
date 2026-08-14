export type PlatformStatus = {
  id: "macos-arm64" | "macos-x64" | "windows-x64";
  label: string;
  architecture: string;
  packageFormat: string;
};

export const supportedPlatforms: readonly PlatformStatus[] = [
  {
    id: "macos-arm64",
    label: "macOS",
    architecture: "Apple Silicon",
    packageFormat: "APP / DMG",
  },
  {
    id: "macos-x64",
    label: "macOS",
    architecture: "Intel",
    packageFormat: "APP / DMG",
  },
  {
    id: "windows-x64",
    label: "Windows",
    architecture: "x64",
    packageFormat: "NSIS",
  },
] as const;

export function platformSummary(platform: PlatformStatus): string {
  return `${platform.label} · ${platform.architecture}`;
}
