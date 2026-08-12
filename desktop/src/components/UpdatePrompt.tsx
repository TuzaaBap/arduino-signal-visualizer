import type { AppUpdaterState } from "../hooks/use-app-updater";

interface UpdatePromptProps {
  state: AppUpdaterState;
  onDownload: () => void;
  onCheckAgain: () => void;
  onSkip: () => void;
  onNotNow: () => void;
}

export function UpdatePrompt({
  state,
  onDownload,
  onCheckAgain,
  onSkip,
  onNotNow,
}: UpdatePromptProps) {
  if (!["available", "downloading", "installing", "error"].includes(state.phase)) {
    return null;
  }
  const update = state.update;
  const checkFailed = state.phase === "error" && update === null;
  const busy = state.phase === "downloading" || state.phase === "installing";
  const percent =
    state.contentLength && state.contentLength > 0
      ? Math.min(100, Math.round((state.downloadedBytes / state.contentLength) * 100))
      : null;

  return (
    <div className="update-overlay" role="presentation">
      <section
        className="update-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="update-title"
        aria-describedby={update ? "update-description" : undefined}
      >
        <div className="update-dialog__heading">
          <div className="update-icon" aria-hidden="true">↑</div>
          <div>
            <p className="eyebrow">Signed application update</p>
            <h2 id="update-title">
              {checkFailed
                ? "Could not check for updates"
                : `Version ${update?.version} is available`}
            </h2>
          </div>
          {update?.prerelease && <span className="update-badge">BETA</span>}
        </div>

        {update && (
          <p id="update-description" className="update-summary">
            You are using {update.currentVersion}. The installer is downloaded from
            the official GitHub release and cryptographically verified before it runs.
          </p>
        )}

        {update?.notes && (
          <div className="update-notes" aria-label="Release notes">
            <strong>What changed</strong>
            <p>{update.notes}</p>
          </div>
        )}

        {state.phase === "downloading" && (
          <div className="update-progress" aria-live="polite">
            <div>
              <span>Downloading update</span>
              <strong>{percent === null ? formatBytes(state.downloadedBytes) : `${percent}%`}</strong>
            </div>
            <progress value={state.downloadedBytes} max={state.contentLength ?? undefined} />
          </div>
        )}
        {state.phase === "installing" && (
          <p className="update-installing" aria-live="assertive">
            Download verified. Installing and restarting the application…
          </p>
        )}
        {state.phase === "error" && (
          <p className="update-error" role="alert">{state.error}</p>
        )}

        <div className="update-actions">
          {!checkFailed && (
            <button type="button" className="secondary" disabled={busy} onClick={onSkip}>
              Skip this version
            </button>
          )}
          <button type="button" className="secondary" disabled={busy} onClick={onNotNow}>
            Not now
          </button>
          <button
            type="button"
            className="primary"
            disabled={busy}
            onClick={checkFailed ? onCheckAgain : onDownload}
          >
            {checkFailed
              ? "Try again"
              : state.phase === "error"
                ? "Try download again"
                : "Download & install"}
          </button>
        </div>
      </section>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1_024) return `${bytes} B`;
  if (bytes < 1_048_576) return `${(bytes / 1_024).toFixed(1)} KB`;
  return `${(bytes / 1_048_576).toFixed(1)} MB`;
}
