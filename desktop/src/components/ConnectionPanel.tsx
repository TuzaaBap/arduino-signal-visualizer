import type {
  ConnectionStatus,
  SerialPortDescriptor,
} from "../domain/types";

interface ConnectionPanelProps {
  status: ConnectionStatus;
  ports: SerialPortDescriptor[];
  selectedPort: string;
  selectedBaud: number;
  busy: boolean;
  backendReady: boolean;
  onSelectedPortChange: (port: string) => void;
  onSelectedBaudChange: (baud: number) => void;
  onRefresh: () => void;
  onConnect: () => void;
  onMock: () => void;
  onDisconnect: () => void;
}

const BAUD_RATES = [4_800, 9_600, 19_200, 38_400, 57_600, 115_200];

export function ConnectionPanel({
  status,
  ports,
  selectedPort,
  selectedBaud,
  busy,
  backendReady,
  onSelectedPortChange,
  onSelectedBaudChange,
  onRefresh,
  onConnect,
  onMock,
  onDisconnect,
}: ConnectionPanelProps) {
  const active =
    status.phase === "connected" || status.phase === "waitingForHello";

  return (
    <section className="connection-panel" aria-labelledby="connection-heading">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Transport</p>
          <h2 id="connection-heading">Board connection</h2>
        </div>
        <span className={`status-dot status-dot--${status.phase}`} />
      </div>

      <label className="field-label" htmlFor="serial-port">
        Serial port
      </label>
      <div className="port-row">
        <select
          id="serial-port"
          value={selectedPort}
          disabled={!backendReady || busy || active}
          onChange={(event) => onSelectedPortChange(event.target.value)}
        >
          <option value="">Select a port</option>
          {ports.map((port) => (
            <option key={port.name} value={port.name}>
              {port.name}
              {port.product ? ` — ${port.product}` : ""}
            </option>
          ))}
        </select>
        <button
          className="icon-button"
          type="button"
          title="Refresh serial ports"
          disabled={!backendReady || busy || active}
          onClick={onRefresh}
        >
          ↻
        </button>
      </div>

      <label className="field-label" htmlFor="serial-baud">
        Baud rate
      </label>
      <select
        id="serial-baud"
        value={selectedBaud}
        disabled={!backendReady || busy || active}
        onChange={(event) => onSelectedBaudChange(Number(event.target.value))}
      >
        {BAUD_RATES.map((baud) => (
          <option key={baud} value={baud}>
            {baud.toLocaleString("en-US")} baud
          </option>
        ))}
      </select>

      <div className="connection-actions">
        {active ? (
          <button
            className="button button--danger"
            type="button"
            disabled={busy}
            onClick={onDisconnect}
          >
            Disconnect
          </button>
        ) : (
          <button
            className="button button--primary"
            type="button"
            disabled={!backendReady || busy || !selectedPort}
            onClick={onConnect}
          >
            Connect
          </button>
        )}
        <button
          className="button button--secondary"
          type="button"
          disabled={!backendReady || busy || active}
          onClick={onMock}
        >
          Start Mock Mode
        </button>
      </div>

      <p className="connection-detail" role="status">
        {backendReady
          ? status.detail
          : "Open the Tauri desktop app to access serial and Mock Mode."}
      </p>
      <p className="serial-ownership-note">
        One desktop program can own a serial port at a time. Close Arduino IDE
        Serial Monitor before connecting, then use this app&apos;s Serial tab for
        normal sketch input and output.
      </p>
      {status.mode === "mock" && (
        <p className="mock-warning">Mock Mode — no physical board connected</p>
      )}
    </section>
  );
}
