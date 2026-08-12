import { useCallback, useEffect, useRef, useState } from "react";

import { isUpdateSkipped, skipUpdateVersion } from "../domain/update-preferences";
import {
  checkForUpdate,
  dismissUpdate,
  installUpdate,
  type UpdateDownloadEvent,
  type UpdateMetadata,
} from "../infrastructure/tauri-bridge";

export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "upToDate"
  | "error";

export interface AppUpdaterState {
  phase: UpdatePhase;
  update: UpdateMetadata | null;
  downloadedBytes: number;
  contentLength: number | null;
  error: string | null;
}

const INITIAL_STATE: AppUpdaterState = {
  phase: "idle",
  update: null,
  downloadedBytes: 0,
  contentLength: null,
  error: null,
};

export function useAppUpdater(enabled: boolean) {
  const [state, setState] = useState(INITIAL_STATE);
  const automaticCheckStarted = useRef(false);

  const check = useCallback(async (manual: boolean) => {
    setState((current) => ({ ...current, phase: "checking", error: null }));
    try {
      const update = await checkForUpdate();
      if (!update) {
        setState({ ...INITIAL_STATE, phase: manual ? "upToDate" : "idle" });
        return;
      }
      if (!manual && isUpdateSkipped(update.version)) {
        await dismissUpdate();
        setState(INITIAL_STATE);
        return;
      }
      setState({ ...INITIAL_STATE, phase: "available", update });
    } catch (error) {
      setState({
        ...INITIAL_STATE,
        phase: "error",
        error: errorMessage(error),
      });
    }
  }, []);

  useEffect(() => {
    if (!enabled || automaticCheckStarted.current) return;
    automaticCheckStarted.current = true;
    const timer = window.setTimeout(() => void check(false), 1_500);
    return () => window.clearTimeout(timer);
  }, [check, enabled]);

  const downloadAndInstall = useCallback(async () => {
    if (
      !state.update ||
      (state.phase !== "available" && state.phase !== "error")
    )
      return;
    setState((current) => ({
      ...current,
      phase: "downloading",
      downloadedBytes: 0,
      contentLength: null,
      error: null,
    }));
    try {
      await installUpdate((event) => {
        setState((current) => applyDownloadEvent(current, event));
      });
    } catch (error) {
      setState((current) => ({
        ...current,
        phase: "error",
        error: errorMessage(error),
      }));
    }
  }, [state.phase, state.update]);

  const notNow = useCallback(async () => {
    await dismissUpdate().catch(() => undefined);
    setState(INITIAL_STATE);
  }, []);

  const skip = useCallback(async () => {
    if (state.update) skipUpdateVersion(state.update.version);
    await dismissUpdate().catch(() => undefined);
    setState(INITIAL_STATE);
  }, [state.update]);

  return {
    state,
    checkNow: () => void check(true),
    downloadAndInstall: () => void downloadAndInstall(),
    notNow: () => void notNow(),
    skip: () => void skip(),
  };
}

export function applyDownloadEvent(
  state: AppUpdaterState,
  event: UpdateDownloadEvent,
): AppUpdaterState {
  switch (event.event) {
    case "started":
      return {
        ...state,
        phase: "downloading",
        contentLength: event.data.contentLength,
      };
    case "progress":
      return {
        ...state,
        phase: "downloading",
        downloadedBytes: state.downloadedBytes + event.data.chunkLength,
      };
    case "installing":
    case "complete":
      return { ...state, phase: "installing" };
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
