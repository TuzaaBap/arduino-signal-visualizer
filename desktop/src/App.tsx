import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { ConnectionPanel } from "./components/ConnectionPanel";
import { Diagnostics } from "./components/Diagnostics";
import { AnalogInspector } from "./components/AnalogInspector";
import { AnalogPanel } from "./components/AnalogPanel";
import { PinInspector } from "./components/PinInspector";
import { PwmInspector } from "./components/PwmInspector";
import { PwmPanel } from "./components/PwmPanel";
import { SerialInspector } from "./components/SerialInspector";
import { SerialMonitor } from "./components/SerialMonitor";
import { UnoBoard } from "./components/UnoBoard";
import {
  applyAdcSamples,
  type AnalogState,
} from "./domain/analog-store";
import { applyGpioUpdates, type GpioState } from "./domain/gpio-store";
import { applyPwmUpdates, type PwmState } from "./domain/pwm-store";
import {
  INACTIVE_SERIAL_LEDS,
  applySerialActivity,
  serialLedVisibility,
  type SerialLedDeadlines,
} from "./domain/serial-led-state";
import {
  EMPTY_USER_SERIAL_STATE,
  appendUserSerial,
  type UserSerialState,
} from "./domain/user-serial-store";
import type {
  AdcBatch,
  BoardDescriptor,
  ConnectionStatus,
  DiagnosticEntry,
  GpioBatch,
  ProtocolDiagnostic,
  PwmBatch,
  SerialActivityBatch,
  SerialPortDescriptor,
  UserSerialBatch,
} from "./domain/types";
import { useFramesPerSecond } from "./hooks/use-performance-metrics";
import {
  backendAvailable,
  acknowledgeValidationAdc,
  acknowledgeValidationGpio,
  acknowledgeValidationPwm,
  connectSerial,
  disconnect,
  listSerialPorts,
  startMock,
  startHardwareValidation,
  subscribeToBackend,
  writeUserSerial,
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
  const [selectedBaud, setSelectedBaud] = useState(115_200);
  const [board, setBoard] = useState<BoardDescriptor | null>(null);
  const [pins, setPins] = useState<GpioState>({});
  const [analog, setAnalog] = useState<AnalogState>({});
  const [pwm, setPwm] = useState<PwmState>({});
  const [serialLedDeadlines, setSerialLedDeadlines] =
    useState<SerialLedDeadlines>(INACTIVE_SERIAL_LEDS);
  const [serialLedClock, setSerialLedClock] = useState(0);
  const [selectedPin, setSelectedPin] = useState(13);
  const [selectedAnalogChannel, setSelectedAnalogChannel] = useState(0);
  const [selectedPwmPin, setSelectedPwmPin] = useState(9);
  const [activeTab, setActiveTab] = useState<
    "digital" | "analog" | "pwm" | "serial"
  >("digital");
  const [userSerial, setUserSerial] = useState<UserSerialState>(
    EMPTY_USER_SERIAL_STATE,
  );
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

  const acceptPwmBatch = useCallback((batch: PwmBatch) => {
    packetCounter.current += batch.updates.length;
    setPwm((current) => applyPwmUpdates(current, batch.updates));
  }, []);

  const acceptSerialActivity = useCallback((batch: SerialActivityBatch) => {
    const observedAtMs = performance.now();
    setSerialLedDeadlines((current) =>
      applySerialActivity(current, batch, observedAtMs),
    );
    setSerialLedClock(observedAtMs);
  }, []);

  const acceptUserSerial = useCallback((batch: UserSerialBatch) => {
    setUserSerial((current) => appendUserSerial(current, batch));
  }, []);

  useEffect(() => {
    if (!backendReady) {
      return;
    }

    let cleanup: (() => void) | undefined;
    void subscribeToBackend({
      onConnectionStatus: (nextStatus) => {
        setStatus(nextStatus);
        if (
          nextStatus.phase === "waitingForHello" ||
          nextStatus.phase === "disconnected" ||
          nextStatus.phase === "error"
        ) {
          setSerialLedDeadlines(INACTIVE_SERIAL_LEDS);
          setSerialLedClock(performance.now());
        }
        if (nextStatus.phase === "waitingForHello") {
          setPins({});
          setAnalog({});
          setPwm({});
          setUserSerial(EMPTY_USER_SERIAL_STATE);
          setBoard(null);
          setDiagnostics([]);
        }
      },
      onBoardInfo: setBoard,
      onGpioBatch: acceptBatch,
      onAdcBatch: acceptAdcBatch,
      onPwmBatch: acceptPwmBatch,
      onSerialActivity: acceptSerialActivity,
      onUserSerial: acceptUserSerial,
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
  }, [
    acceptAdcBatch,
    acceptBatch,
    acceptPwmBatch,
    acceptSerialActivity,
    acceptUserSerial,
    appendDiagnostic,
    backendReady,
  ]);

  useEffect(() => {
    const nowMs = performance.now();
    const nextDeadline = [
      serialLedDeadlines.txActiveUntilMs,
      serialLedDeadlines.rxActiveUntilMs,
    ]
      .filter((deadline) => deadline > nowMs)
      .sort((left, right) => left - right)[0];
    if (nextDeadline === undefined) {
      return;
    }

    const timer = window.setTimeout(
      () => setSerialLedClock(performance.now()),
      Math.max(1, nextDeadline - nowMs),
    );
    return () => window.clearTimeout(timer);
  }, [serialLedClock, serialLedDeadlines]);

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
    if (!validationActive) {
      return;
    }
    const pins = Object.entries(pwm).map(([pin, state]) => ({
      pin: Number(pin),
      bufferLength: state.history.length,
      latest: state.latest,
    }));
    if (pins.length > 0) {
      void acknowledgeValidationPwm(pins);
    }
  }, [pwm, validationActive]);

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
  const observedPwmCount = useMemo(() => Object.keys(pwm).length, [pwm]);
  const serialLeds = useMemo(
    () => serialLedVisibility(serialLedDeadlines, serialLedClock),
    [serialLedClock, serialLedDeadlines],
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
            <p>GPIO, ADC, and PWM instrumentation workspace</p>
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
            selectedBaud={selectedBaud}
            busy={busy}
            backendReady={backendReady}
            onSelectedPortChange={setSelectedPort}
            onSelectedBaudChange={setSelectedBaud}
            onRefresh={() => void refreshPorts()}
            onConnect={() =>
              void perform(() => connectSerial(selectedPort, selectedBaud))
            }
            onMock={() => void perform(startMock)}
            onDisconnect={() => void perform(disconnect)}
          />

          <section className="telemetry" aria-label="Application telemetry">
            <p className="eyebrow">Live telemetry</p>
            <div className="telemetry-grid">
              <Metric label="Board" value="Arduino Uno" />
              <Metric label="Firmware" value={firmware} />
              <Metric label="Port" value={status.portName ?? "—"} />
              <Metric label="App" value="0.5.0" />
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
              <p className="eyebrow">
                {activeTab === "serial" ? "Separated user stream" : "Interactive board"}
              </p>
              <h2 id="board-heading">
                {activeTab === "serial" ? "Serial Monitor" : "Arduino Uno"}
              </h2>
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
              <button
                type="button"
                role="tab"
                aria-selected={activeTab === "pwm"}
                className={activeTab === "pwm" ? "active" : ""}
                onClick={() => setActiveTab("pwm")}
              >
                PWM
              </button>
              <button
                type="button"
                role="tab"
                aria-selected={activeTab === "serial"}
                className={activeTab === "serial" ? "active" : ""}
                onClick={() => setActiveTab("serial")}
              >
                Serial
              </button>
            </div>
          </div>
          <div className="board-content">
            {activeTab === "serial" ? (
              <SerialMonitor
                state={userSerial}
                canSend={
                  status.phase === "connected" && status.mode === "serial"
                }
                onSend={writeUserSerial}
                onClear={() => setUserSerial(EMPTY_USER_SERIAL_STATE)}
              />
            ) : (
              <>
                <div className="board-and-summary">
                  <div className="observation-summary">
                    <span>
                      {activeTab === "digital"
                        ? connectedPinCount
                        : activeTab === "analog"
                          ? observedAnalogCount
                          : observedPwmCount}
                    </span>
                    {activeTab === "digital"
                      ? "of 14 digital pins observed"
                      : activeTab === "analog"
                        ? "of 6 analog channels observed"
                        : "of 6 hardware PWM pins observed"}
                  </div>
                  <UnoBoard
                    pins={pins}
                    pwm={pwm}
                    serialLeds={serialLeds}
                    selectedDigitalPin={selectedPin}
                    selectedAnalogChannel={selectedAnalogChannel}
                    selectedPwmPin={selectedPwmPin}
                    activeTab={activeTab}
                    onSelectDigitalPin={(pin) => {
                      setSelectedPin(pin);
                      setActiveTab("digital");
                    }}
                    onSelectAnalogChannel={(channel) => {
                      setSelectedAnalogChannel(channel);
                      setActiveTab("analog");
                    }}
                    onSelectPwmPin={(pin) => {
                      setSelectedPwmPin(pin);
                      setActiveTab("pwm");
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
                {activeTab === "pwm" && (
                  <PwmPanel
                    pins={pwm}
                    selectedPin={selectedPwmPin}
                    mockMode={status.mode === "mock"}
                    onSelectPin={setSelectedPwmPin}
                  />
                )}
              </>
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
        ) : activeTab === "analog" ? (
          <AnalogInspector
            channel={selectedAnalogChannel}
            state={analog[selectedAnalogChannel]}
          />
        ) : activeTab === "pwm" ? (
          <PwmInspector pin={selectedPwmPin} state={pwm[selectedPwmPin]} />
        ) : (
          <SerialInspector state={userSerial} />
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
