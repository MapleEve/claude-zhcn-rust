import { describe, expect, test } from "bun:test";
import { platformSummary, supportedPlatforms } from "./platforms";

describe("supportedPlatforms", () => {
  test("只声明三个首发目标", () => {
    expect(supportedPlatforms.map(({ id }) => id)).toEqual([
      "macos-arm64",
      "macos-x64",
      "windows-x64",
    ]);
  });

  test("生成可读平台摘要", () => {
    expect(platformSummary(supportedPlatforms[2])).toBe("Windows · x64");
  });
});
