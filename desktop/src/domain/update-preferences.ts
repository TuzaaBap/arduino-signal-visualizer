const SKIPPED_UPDATE_KEY = "asv.skippedUpdateVersion";

export interface PreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function skippedUpdateVersion(
  storage: PreferenceStorage = window.localStorage,
): string | null {
  try {
    return storage.getItem(SKIPPED_UPDATE_KEY);
  } catch {
    return null;
  }
}

export function skipUpdateVersion(
  version: string,
  storage: PreferenceStorage = window.localStorage,
): void {
  try {
    storage.setItem(SKIPPED_UPDATE_KEY, version);
  } catch {
    // A read-only browser profile must not block the rest of the application.
  }
}

export function isUpdateSkipped(
  version: string,
  storage: PreferenceStorage = window.localStorage,
): boolean {
  return skippedUpdateVersion(storage) === version;
}
