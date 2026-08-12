import { describe, expect, it } from "vitest";

import { applyDownloadEvent, type AppUpdaterState } from "./use-app-updater";

const state: AppUpdaterState = {
  phase: "available",
  update: null,
  downloadedBytes: 0,
  contentLength: null,
  error: null,
};

describe("updater download progress", () => {
  it("accumulates chunks and preserves the announced total", () => {
    const started = applyDownloadEvent(state, {
      event: "started",
      data: { contentLength: 1_000 },
    });
    const first = applyDownloadEvent(started, {
      event: "progress",
      data: { chunkLength: 320 },
    });
    const second = applyDownloadEvent(first, {
      event: "progress",
      data: { chunkLength: 180 },
    });

    expect(second.phase).toBe("downloading");
    expect(second.downloadedBytes).toBe(500);
    expect(second.contentLength).toBe(1_000);
  });

  it("switches to installing only after the verified download finishes", () => {
    expect(applyDownloadEvent(state, { event: "installing" }).phase).toBe(
      "installing",
    );
  });
});
