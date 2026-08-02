import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { ConnectionPanel } from "./components/ConnectionPanel";
import { Diagnostics } from "./components/Diagnostics";
import { AnalogInspector } from "./components/AnalogInspector";
import { AnalogPanel } from "./components/AnalogPanel";
import { PinInspector } from "./components/PinInspector";
import { UnoBoard } from "./components/UnoBoard";
import {
  applyAdcSamples,
  type AnalogState,
} from "./domain/analog-store";
import { applyGpioUpdates, type GpioState } from "./domain/gpio-store";
import type {
  AdcBatch,
  BoardDescriptor,
  ConnectionStatus,
  DiagnosticEntry,
  GpioBatch,
  ProtocolDiagnostic,
  SerialPortDescriptor,
} from "./domain/types";
import { useFramesPerSecond } from "./hooks/use-performance-metrics";
import {
  backendAvailable,
  acknowledgeValidationAdc,
  acknowledgeValidationGpio,
  connectSerial,
  disconnect,
  listSerialPorts,
  startMock,
  startHardwareValidation,
  subscribeToBackend,
} from "./infrastructure/tauri-bridge";

const INITIAL_STATUS: ConnectionStatus = {
  phase: "disconnected",
  mode: null,
  portName: null,
  detail: "Select a serial port or use Mock Mode",
};

export function App() {
  const backendReady = backendAvailable();
  const [status, setStatus] = useState(INITIAL_STATUS);
  const [ports, setPorts] = useState<SerialPortDescriptor[]>([]);
  const [selectedPort, setSelectedPort] = useState("");
  const [board, setBoard] = useState<BoardDescriptor | null>(null);
  const [pins, setPins] = useState<GpioState>({});
  const [analog, setAnalog] = useState<AnalogState>({});
  const [selectedPin, setSelectedPin] = useState(13);
  const [selectedAnalogChannel, setSelectedAnalogChannel] = useState(0);
  const [activeTab, setActiveTab] = useState<"digital" | "analog">("digital");
  const [diagnostics, setDiagnostics] = useState<DiagnosticEntry[]>([]);
  const [busy, setBusy] = useState(false);
  const [packetRate, setPacketRate] = useState(0);
  const [validationActive, setValidationActive] = useState(false);
  const packetCounter = useRef(0);
  const diagnosticId = useRef(0);
  const fps = useFramesPerSecond();

  const appendDiagnostic = useCallback((diagnostic: ProtocolDiagnostic) => {
    diagnosticId.current += 1;
    setDiagnostics((current) =>
      [
        {
          ...diagnostic,
          id: diagnosticId.current,
          receivedAt: new Date(),
        },
        ...current,
      ].slice(0, 20),
    );
  }, []);

  const acceptBatch = useCallback((batch: GpioBatch) => {
    packetCounter.current += batch.updates.length;
    setPins((current) => applyGpioUpdates(current, batch.updates));
  }, []);

  const acceptAdcBatch = useCallback((batch: AdcBatch) => {
    packetCounter.current += batch.samples.length;
    setAnalog((current) => applyAdcSamples(current, batch.samples));
  }, []);

  useEffect(() => {
    if (!backendReady) {
      return;
    }

    let cleanup: (() => void) | undefined;
    void subscribeToBackend({
      onConnectionStatus: (nextStatus) => {
        setStatus(nextStatus);
        if (nextStatus.phase === "waitingForHello") {
          setPins({});
          setAnalog({});
          setBoard(null);
          setDiagnostics([]);
        }
      },
      onBoardInfo: setBoard,
      onGpioBatch: acceptBatch,
      onAdcBatch: acceptAdcBatch,
      onDiagnostic: appendDiagnostic,
    })
      .then((removeListeners) => {
        cleanup = removeListeners;
        return startHardwareValidation();
      })
      .then((active) => {
        setValidationActive(active);
      })
      .catch((error: unknown) => {
        setStatus({
          phase: "error",
          mode: null,
          portName: null,
          detail: errorMessage(error),
        });
      });

    return () => cleanup?.();
  }, [acceptAdcBatch, acceptBatch, appendDiagnostic, backendReady]);

  useEffect(() => {
    if (!validationActive) {
      return;
    }
    const updates = Object.values(pins);
    if (updates.length > 0) {
      void acknowledgeValidationGpio(updates);
    }
  }, [pins, validationActive]);

  useEffect(() => {
    if (!validationActive) {
      return;
    }
    const channels = Object.entries(analog).map(([channel, state]) => ({
      channel: Number(channel),
      bufferLength: state.history.length,
      latest: state.latest,
    }));
    if (channels.length > 0) {
      void acknowledgeValidationAdc(channels);
    }
  }, [analog, validationActive]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setPacketRate(packetCounter.current);
      packetCounter.current = 0;
    }, 1_000);
    return () => window.clearInterval(timer);
  }, []);

  const refreshPorts = useCallback(async () => {
    if (!backendReady) {
      return;
    }
    setBusy(true);
    try {
      const discovered = await listSerialPorts();
      setPorts(discovered);
      setSelectedPort((current) => {
        if (discovered.some((port) => port.name === current)) {
          return current;
        }
        const usb = discovered.find((port) => port.kind === "usb");
        return usb?.name ?? discovered[0]?.name ?? "";
      });
    } catch (error) {
      setStatus({
        phase: "error",
        mode: null,
        portName: null,
        detail: errorMessage(error),
      });
    } finally {
      setBusy(false);
    }
  }, [backendReady]);

  useEffect(() => {
    void refreshPorts();
  }, [refreshPorts]);

  const perform = useCallback(async (operation: () => Promise<void>) => {
    setBusy(true);
    try {
      await operation();
    } catch (error) {
      setStatus((current) => ({
        ...current,
        phase: "error",
        detail: errorMessage(error),
      }));
    } finally {
      setBusy(false);
    }
  }, []);

  const connectedPinCount = useMemo(
    () => Object.keys(pins).length,
    [pins],
  );
  const observedAnalogCount = useMemo(
    () => Object.keys(analog).length,
    [analog],
  );
  const firmware = board
    ? `${board.firmwareVersion.major}.${board.firmwareVersion.minor}.${board.firmwareVersion.patch}`
    : "—";

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            ∞
          </span>
          <div>
            <h1>Arduino Signal Visualizer</h1>
            <p>GPIO and ADC instrumentation workspace</p>
          </div>
        </div>
        <div className={`header-status header-status--${status.phase}`}>
          <span />
          {status.phase === "waitingForHello"
            ? "Waiting for firmware"
            : status.phase}
        </div>
      </header>

      <main className="workspace">
        <aside className="left-rail">
          <ConnectionPanel
            status={status}
            ports={ports}
            selectedPort={selectedPort}
            busy={busy}
            backendReady={backendReady}
            onSelectedPortChange={setSelectedPort}
            onRefresh={() => void refreshPorts()}
            onConnect={() =>
              void perform(() => connectSerial(selectedPort, 115_200))
            }
            onMock={() => void perform(startMock)}
            onDisconnect={() => void perform(disconnect)}
          />

          <section className="telemetry" aria-label="Application telemetry">
            <p className="eyebrow">Live telemetry</p>
            <div className="telemetry-grid">
              <Metric label="Board" value="Uno R3" />
              <Metric label="Firmware" value={firmware} />
              <Metric label="Port" value={status.portName ?? "—"} />
              <Metric label="App" value="0.2.0" />
              <Metric label="Render" value={`${fps} FPS`} />
              <Metric label="Packets" value={`${packetRate}/s`} />
            </div>
          </section>

          <Diagnostics entries={diagnostics} />
        </aside>

        <section
          className={`board-workspace board-workspace--${activeTab}`}
          aria-labelledby="board-heading"
        >
          <div className="board-workspace-heading">
            <div>
              <p className="eyebrow">Interactive board</p>
              <h2 id="board-heading">Arduino Uno R3</h2>
            </div>
            <div className="workspace-tabs" role="tablist" aria-label="Signal type">
              <button
                type="button"
                role="tab"
                aria-selected={activeTab === "digital"}
                className={activeTab === "digital" ? "active" : ""}
                onClick={() => setActiveTab("digital")}
              >
                Digital
              </button>
              <button
                type="button"
                role="tab"
                aria-selected={activeTab === "analog"}
                className={activeTab === "analog" ? "active" : ""}
                onClick={() => setActiveTab("analog")}
              >
                Analog
              </button>
            </div>
          </div>
          <div className="board-content">
            <div className="board-and-summary">
              <div className="observation-summary">
                <span>
                  {activeTab === "digital"
                    ? connectedPinCount
                    : observedAnalogCount}
                </span>
                {activeTab === "digital"
                  ? "of 14 digital pins observed"
                  : "of 6 analog channels observed"}
              </div>
              <UnoBoard
                pins={pins}
                selectedDigitalPin={selectedPin}
                selectedAnalogChannel={selectedAnalogChannel}
                activeTab={activeTab}
                onSelectDigitalPin={(pin) => {
                  setSelectedPin(pin);
                  setActiveTab("digital");
                }}
                onSelectAnalogChannel={(channel) => {
                  setSelectedAnalogChannel(channel);
                  setActiveTab("analog");
                }}
              />
            </div>
            {activeTab === "analog" && (
              <AnalogPanel
                channels={analog}
                selectedChannel={selectedAnalogChannel}
                mockMode={status.mode === "mock"}
                onSelectChannel={setSelectedAnalogChannel}
              />
            )}
          </div>
          {activeTab === "digital" && (
            <div className="legend" aria-label="GPIO state legend">
              <span>
                <i className="legend-dot legend-dot--high" /> HIGH
              </span>
              <span>
                <i className="legend-dot" /> LOW / not observed
              </span>
            </div>
          )}
        </section>

        {activeTab === "digital" ? (
          <PinInspector
            pin={selectedPin}
            state={pins[selectedPin]}
            nominalLogicMv={board?.nominalLogicMv ?? 5_000}
          />
        ) : (
          <AnalogInspector
            channel={selectedAnalogChannel}
            state={analog[selectedAnalogChannel]}
          />
        )}
      </main>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong title={value}>{value}</strong>
    </div>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
