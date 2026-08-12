import { describe, expect, it } from "vitest";

import {
  isUpdateSkipped,
  skipUpdateVersion,
  skippedUpdateVersion,
  type PreferenceStorage,
} from "./update-preferences";

function memoryStorage(): PreferenceStorage {
  const entries = new Map<string, string>();
  return {
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => entries.set(key, value),
  };
}

describe("update preferences", () => {
  it("skips only the exact selected version", () => {
    const storage = memoryStorage();

    skipUpdateVersion("0.6.0", storage);

    expect(skippedUpdateVersion(storage)).toBe("0.6.0");
    expect(isUpdateSkipped("0.6.0", storage)).toBe(true);
    expect(isUpdateSkipped("0.6.1", storage)).toBe(false);
  });

  it("fails closed when preference storage is unavailable", () => {
    const storage: PreferenceStorage = {
      getItem: () => {
        throw new Error("disabled");
      },
      setItem: () => {
        throw new Error("disabled");
      },
    };

    expect(skippedUpdateVersion(storage)).toBeNull();
    expect(() => skipUpdateVersion("0.6.0", storage)).not.toThrow();
  });
});
