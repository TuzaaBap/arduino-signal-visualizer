import type { GpioState } from "../domain/gpio-store";

interface UnoBoardProps {
  pins: GpioState;
  selectedPin: number;
  onSelectPin: (pin: number) => void;
}

const DIGITAL_PINS = Array.from({ length: 14 }, (_, index) => 13 - index);

export function UnoBoard({
  pins,
  selectedPin,
  onSelectPin,
}: UnoBoardProps) {
  return (
    <div className="board-stage">
      <svg
        className="uno-board"
        viewBox="0 0 860 480"
        role="img"
        aria-labelledby="uno-title uno-description"
      >
        <title id="uno-title">Interactive Arduino Uno R3 digital pins</title>
        <desc id="uno-description">
          Select a digital pin from D0 through D13 to inspect its latest
          instrumented state.
        </desc>
        <defs>
          <linearGradient id="pcb" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stopColor="#087f87" />
            <stop offset="1" stopColor="#075760" />
          </linearGradient>
          <filter id="board-shadow" x="-20%" y="-20%" width="140%" height="160%">
            <feDropShadow
              dx="0"
              dy="18"
              stdDeviation="16"
              floodColor="#02080d"
              floodOpacity="0.5"
            />
          </filter>
        </defs>

        <path
          className="board-shape"
          d="M120 68 H748 Q770 68 770 90 V388 Q770 410 748 410 H120 Q98 410 98 388 V335 H72 V145 H98 V90 Q98 68 120 68Z"
          fill="url(#pcb)"
          filter="url(#board-shadow)"
        />
        <circle className="mount-hole" cx="132" cy="105" r="13" />
        <circle className="mount-hole" cx="730" cy="105" r="13" />
        <circle className="mount-hole" cx="730" cy="370" r="13" />
        <circle className="mount-hole" cx="132" cy="370" r="13" />

        <rect className="usb-shell" x="38" y="165" width="112" height="96" rx="8" />
        <rect className="usb-mouth" x="38" y="186" width="34" height="54" rx="3" />
        <rect className="barrel-jack" x="76" y="306" width="104" height="72" rx="8" />
        <circle className="barrel-hole" cx="91" cy="342" r="20" />

        <rect className="chip" x="330" y="208" width="250" height="88" rx="8" />
        {Array.from({ length: 14 }, (_, index) => (
          <g key={`leg-${index}`}>
            <rect x={344 + index * 16} y="198" width="7" height="10" rx="1" />
            <rect x={344 + index * 16} y="296" width="7" height="10" rx="1" />
          </g>
        ))}
        <circle cx="355" cy="252" r="6" fill="#57616b" />
        <text className="chip-label" x="455" y="246" textAnchor="middle">
          ATmega328P
        </text>
        <text className="chip-subtitle" x="455" y="266" textAnchor="middle">
          INSTRUMENTED MCU
        </text>

        <g className="crystal">
          <rect x="262" y="230" width="45" height="24" rx="11" />
          <text x="284" y="247" textAnchor="middle">
            16
          </text>
        </g>

        <g className="uno-mark">
          <text x="616" y="230">ARDUINO</text>
          <text x="616" y="265">UNO</text>
          <path d="M615 286 H700" />
        </g>

        <text className="header-label" x="420" y="104" textAnchor="middle">
          DIGITAL GPIO
        </text>
        <rect className="pin-header" x="205" y="116" width="506" height="48" rx="6" />

        {DIGITAL_PINS.map((pin, index) => {
          const state = pins[pin];
          const high = state?.level === "high";
          const selected = selectedPin === pin;
          const x = 229 + index * 35.2;
          return (
            <g
              key={pin}
              className={`board-pin ${high ? "board-pin--high" : ""} ${
                selected ? "board-pin--selected" : ""
              }`}
              role="button"
              tabIndex={0}
              aria-label={`Digital pin D${pin}, ${state?.level ?? "not observed"}`}
              onClick={() => onSelectPin(pin)}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  onSelectPin(pin);
                }
              }}
            >
              <circle cx={x} cy="140" r="11" />
              <text x={x} y="188" textAnchor="middle">
                {pin}
              </text>
              {(pin === 0 || pin === 1) && (
                <text className="serial-label" x={x} y="207" textAnchor="middle">
                  {pin === 0 ? "RX" : "TX"}
                </text>
              )}
            </g>
          );
        })}

        <g className="power-led">
          <circle cx="677" cy="323" r="8" />
          <text x="677" y="347" textAnchor="middle">
            ON
          </text>
        </g>
        <g className="signal-led">
          <circle
            className={pins[13]?.level === "high" ? "active" : ""}
            cx="638"
            cy="323"
            r="8"
          />
          <text x="638" y="347" textAnchor="middle">
            L
          </text>
        </g>
      </svg>
    </div>
  );
}

