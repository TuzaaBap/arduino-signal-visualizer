import { useMemo, useState } from "react";

import type { UserSerialState } from "../domain/user-serial-store";

interface SerialMonitorProps {
  state: UserSerialState;
  canSend: boolean;
  onSend: (bytes: number[]) => Promise<number>;
  onClear: () => void;
}

export function SerialMonitor({
  state,
  canSend,
  onSend,
  onClear,
}: SerialMonitorProps) {
  const [displayMode, setDisplayMode] = useState<"text" | "hex">("text");
  const [input, setInput] = useState("");
  const [lineEnding, setLineEnding] = useState<"none" | "lf" | "crlf">("lf");
  const [sending, setSending] = useState(false);
  const [sendError, setSendError] = useState("");
  const output = useMemo(
    () =>
      displayMode === "text"
        ? new TextDecoder().decode(Uint8Array.from(state.bytes))
        : state.bytes
            .map((byte, index) =>
              `${byte.toString(16).padStart(2, "0").toUpperCase()}${
                (index + 1) % 16 === 0 ? "\n" : " "
              }`,
            )
            .join("")
            .trimEnd(),
    [displayMode, state.bytes],
  );

  const send = async () => {
    if (!canSend || sending || input.length === 0) {
      return;
    }
    const suffix = lineEnding === "lf" ? "\n" : lineEnding === "crlf" ? "\r\n" : "";
    const bytes = Array.from(new TextEncoder().encode(input + suffix));
    setSending(true);
    setSendError("");
    try {
      await onSend(bytes);
      setInput("");
    } catch (error) {
      setSendError(error instanceof Error ? error.message : String(error));
    } finally {
      setSending(false);
    }
  };

  return (
    <section className="serial-monitor" aria-labelledby="serial-monitor-heading">
      <div className="serial-monitor-toolbar">
        <div>
          <p className="eyebrow">User UART stream</p>
          <h3 id="serial-monitor-heading">Serial Monitor</h3>
        </div>
        <div className="serial-monitor-actions">
          <button
            type="button"
            className={displayMode === "text" ? "active" : ""}
            onClick={() => setDisplayMode("text")}
          >
            Text
          </button>
          <button
            type="button"
            className={displayMode === "hex" ? "active" : ""}
            onClick={() => setDisplayMode("hex")}
          >
            Hex
          </button>
          <button type="button" onClick={onClear}>
            Clear
          </button>
        </div>
      </div>
      <pre className="serial-monitor-output">
        {output || "Waiting for sketch Serial output..."}
      </pre>
      <form
        className="serial-monitor-input"
        onSubmit={(event) => {
          event.preventDefault();
          void send();
        }}
      >
        <input
          type="text"
          value={input}
          maxLength={240}
          disabled={!canSend || sending}
          placeholder={canSend ? "Send text to the sketch" : "Connect a physical board to send"}
          onChange={(event) => setInput(event.target.value)}
        />
        <select
          value={lineEnding}
          disabled={!canSend || sending}
          aria-label="Serial line ending"
          onChange={(event) =>
            setLineEnding(event.target.value as "none" | "lf" | "crlf")
          }
        >
          <option value="none">No ending</option>
          <option value="lf">Newline</option>
          <option value="crlf">CRLF</option>
        </select>
        <button type="submit" disabled={!canSend || sending || input.length === 0}>
          {sending ? "Sending..." : "Send"}
        </button>
      </form>
      {sendError && <p className="serial-monitor-error">{sendError}</p>}
      <div className="serial-monitor-status">
        <span>{state.receivedBytes.toLocaleString()} bytes received</span>
        <span>{state.droppedBytes.toLocaleString()} bytes dropped</span>
      </div>
    </section>
  );
}
