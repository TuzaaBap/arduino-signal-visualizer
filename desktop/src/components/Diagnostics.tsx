import type { DiagnosticEntry } from "../domain/types";

interface DiagnosticsProps {
  entries: DiagnosticEntry[];
}

export function Diagnostics({ entries }: DiagnosticsProps) {
  return (
    <section className="diagnostics" aria-labelledby="diagnostics-heading">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Protocol health</p>
          <h2 id="diagnostics-heading">Diagnostics</h2>
        </div>
        <span className="diagnostic-count">{entries.length}</span>
      </div>
      {entries.length === 0 ? (
        <p className="empty-state">No protocol faults in this connection.</p>
      ) : (
        <ol className="diagnostic-list">
          {entries.map((entry) => (
            <li key={entry.id}>
              <time>{entry.receivedAt.toLocaleTimeString()}</time>
              <div>
                <strong>{humanize(entry.category)}</strong>
                <p>{entry.message}</p>
              </div>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

function humanize(value: string): string {
  return value.replace(/([A-Z])/g, " $1").replace(/^./, (letter) =>
    letter.toUpperCase(),
  );
}

