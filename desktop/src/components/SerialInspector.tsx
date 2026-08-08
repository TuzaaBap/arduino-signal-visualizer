import {
  USER_SERIAL_BUFFER_CAPACITY,
  type UserSerialState,
} from "../domain/user-serial-store";

interface SerialInspectorProps {
  state: UserSerialState;
}

export function SerialInspector({ state }: SerialInspectorProps) {
  return (
    <aside className="pin-inspector" aria-labelledby="serial-inspector-heading">
      <div className="pin-title-row">
        <div>
          <p className="eyebrow">Shared transport</p>
          <h2 id="serial-inspector-heading">User Serial</h2>
        </div>
        <span className="logic-badge">UART</span>
      </div>
      <dl className="pin-facts">
        <div>
          <dt>Received</dt>
          <dd>{state.receivedBytes.toLocaleString()} bytes</dd>
        </div>
        <div>
          <dt>Buffered</dt>
          <dd>
            {state.bytes.length.toLocaleString()} /{" "}
            {USER_SERIAL_BUFFER_CAPACITY.toLocaleString()}
          </dd>
        </div>
        <div>
          <dt>Dropped</dt>
          <dd>{state.droppedBytes.toLocaleString()} bytes</dd>
        </div>
        <div>
          <dt>Separation</dt>
          <dd>ASV v2 framing</dd>
        </div>
      </dl>
      <p className="measurement-note">
        This view contains sketch Serial bytes only. ASV telemetry is decoded
        separately and never printed into the terminal.
      </p>
    </aside>
  );
}
