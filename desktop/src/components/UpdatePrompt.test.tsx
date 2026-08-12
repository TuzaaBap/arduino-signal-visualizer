import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { AppUpdaterState } from "../hooks/use-app-updater";
import { UpdatePrompt } from "./UpdatePrompt";

const available: AppUpdaterState = {
  phase: "available",
  update: {
    version: "0.7.0",
    currentVersion: "0.6.0",
    notes: "Improved protocol stability.",
    publishedAt: "2026-08-20T12:00:00Z",
    releaseUrl: "https://github.test/release",
    prerelease: true,
  },
  downloadedBytes: 0,
  contentLength: null,
  error: null,
};

describe("update prompt", () => {
  it("shows the three requested update choices", () => {
    const markup = renderToStaticMarkup(
      <UpdatePrompt
        state={available}
        onDownload={() => undefined}
        onCheckAgain={() => undefined}
        onSkip={() => undefined}
        onNotNow={() => undefined}
      />,
    );

    expect(markup).toContain("Download &amp; install");
    expect(markup).toContain("Skip this version");
    expect(markup).toContain("Not now");
    expect(markup).toContain("BETA");
  });

  it("reports verified installation after download completion", () => {
    const markup = renderToStaticMarkup(
      <UpdatePrompt
        state={{ ...available, phase: "installing" }}
        onDownload={() => undefined}
        onCheckAgain={() => undefined}
        onSkip={() => undefined}
        onNotNow={() => undefined}
      />,
    );

    expect(markup).toContain("Download verified");
    expect(markup).toContain("disabled");
  });

  it("offers a retry when GitHub update discovery fails", () => {
    const markup = renderToStaticMarkup(
      <UpdatePrompt
        state={{
          ...available,
          phase: "error",
          update: null,
          error: "GitHub is unavailable",
        }}
        onDownload={() => undefined}
        onCheckAgain={() => undefined}
        onSkip={() => undefined}
        onNotNow={() => undefined}
      />,
    );

    expect(markup).toContain("Could not check for updates");
    expect(markup).toContain("Try again");
    expect(markup).not.toContain("Skip this version");
  });
});
