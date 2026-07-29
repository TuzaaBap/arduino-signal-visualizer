import type {
  ConnectionStatus,
  SerialPortDescriptor,
} from "../domain/types";

interface ConnectionPanelProps {
  status: ConnectionStatus;
  ports: SerialPortDescriptor[];
  selectedPort: string;
  busy: boolean;
  backendReady: boolean;
  onSelectedPortChange: (port: string) => void;
  onRefresh: () => void;
  onConnect: () => void;
  onMock: () => void;
  onDisconnect: () => void;
}

export function ConnectionPanel({
  status,
  ports,
  selectedPort,
  busy,
  backendReady,
  onSelectedPortChange,
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
      {status.mode === "mock" && (
        <p className="mock-warning">Mock Mode — no physical board connected</p>
      )}
    </section>
  );
}

