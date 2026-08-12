import { adcFullScale, type AnalogState } from "../domain/analog-store";
import type { GpioState } from "../domain/gpio-store";
import type { PwmState } from "../domain/pwm-store";
import type { SerialLedVisibility } from "../domain/serial-led-state";
import {
  UNO_R3_ANALOG_PINS,
  UNO_R3_AUXILIARY_HEADER,
  UNO_R3_DIGITAL_PINS,
  UNO_R3_POWER_HEADER,
  describeCapabilities,
  type UnoHeaderPinDefinition,
} from "../domain/uno-r3-pinout";

interface UnoBoardProps {
  pins: GpioState;
  analog: AnalogState;
  pwm: PwmState;
  serialLeds: SerialLedVisibility;
  selectedDigitalPin: number;
  selectedAnalogChannel: number;
  selectedPwmPin: number;
  activeTab: "digital" | "analog" | "pwm";
  onSelectDigitalPin: (pin: number) => void;
  onSelectAnalogChannel: (channel: number) => void;
  onSelectPwmPin: (pin: number) => void;
}

const digitalPinX = (index: number): number =>
  index < 6 ? 378 + index * 31 : 574 + (index - 6) * 31;

const auxiliaryPinX = [254, 285, 316, 347] as const;

function auxiliaryPinCoordinate(index: number): number {
  const coordinate = auxiliaryPinX[index];
  if (coordinate === undefined) {
    throw new RangeError(`Missing Uno auxiliary-header coordinate ${index}`);
  }
  return coordinate;
}

function StaticHeaderSocket({
  definition,
  x,
  y,
  compact = false,
  labelPlacement = "below",
}: {
  definition: UnoHeaderPinDefinition;
  x: number;
  y: number;
  compact?: boolean;
  labelPlacement?: "above" | "below";
}) {
  return (
    <g className="static-header-pin">
      <title>
        {definition.label}: {definition.description}
        {definition.aliasOf ? ` (same signal as ${definition.aliasOf})` : ""}
      </title>
      <rect x={x - 11} y={y - 12} width="22" height="24" rx="2" />
      <circle cx={x} cy={y} r="6" />
      <text
        className={compact ? "static-pin-label static-pin-label--compact" : "static-pin-label"}
        x={x}
        y={labelPlacement === "above" ? y - 25 : y + 28}
        textAnchor="middle"
      >
        {definition.label}
      </text>
    </g>
  );
}

export function UnoBoard({
  pins,
  analog,
  pwm,
  serialLeds,
  selectedDigitalPin,
  selectedAnalogChannel,
  selectedPwmPin,
  activeTab,
  onSelectDigitalPin,
  onSelectAnalogChannel,
  onSelectPwmPin,
}: UnoBoardProps) {
  return (
    <div className="board-stage">
      <svg
        className="uno-board"
        viewBox="0 0 940 650"
        role="img"
        aria-labelledby="uno-title uno-description"
      >
        <title id="uno-title">Interactive Arduino Uno board</title>
        <desc id="uno-description">
          Physically representative Arduino Uno board with selectable D0 to
          D13, A0 to A5, and accurate PWM, UART, SPI, I2C, interrupt and power
          pin markings. The drawing is an educational diagram, not a PCB
          manufacturing file.
        </desc>
        <defs>
          <linearGradient id="pcb" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stopColor="#07939b" />
            <stop offset="0.48" stopColor="#087c84" />
            <stop offset="1" stopColor="#05626a" />
          </linearGradient>
          <linearGradient id="metal" x1="0" y1="0" x2="0.8" y2="1">
            <stop offset="0" stopColor="#edf2f3" />
            <stop offset="0.5" stopColor="#aab6bb" />
            <stop offset="1" stopColor="#7f8c92" />
          </linearGradient>
          <filter id="board-shadow" x="-20%" y="-20%" width="145%" height="155%">
            <feDropShadow
              dx="0"
              dy="15"
              stdDeviation="14"
              floodColor="#02080d"
              floodOpacity="0.55"
            />
          </filter>
          <pattern id="pcb-texture" width="7" height="7" patternUnits="userSpaceOnUse">
            <path d="M0 1 H7 M1 0 V7" stroke="#ffffff" strokeOpacity="0.025" strokeWidth="0.5" />
          </pattern>
        </defs>

        <path
          className="board-shape"
          d="M144 36 H824 L872 84 V169 Q889 175 889 195 V510 Q889 531 870 536 V580 Q870 608 842 608 H151 Q123 608 123 580 V554 H93 V436 H123 V285 H86 V145 H123 V63 Q123 36 144 36Z"
          fill="url(#pcb)"
          filter="url(#board-shadow)"
        />
        <path
          className="board-texture"
          d="M144 36 H824 L872 84 V169 Q889 175 889 195 V510 Q889 531 870 536 V580 Q870 608 842 608 H151 Q123 608 123 580 V554 H93 V436 H123 V285 H86 V145 H123 V63 Q123 36 144 36Z"
          fill="url(#pcb-texture)"
        />

        <circle className="mount-hole" cx="160" cy="74" r="18" />
        <circle className="mount-hole" cx="835" cy="190" r="18" />
        <circle className="mount-hole" cx="831" cy="563" r="18" />
        <circle className="mount-hole" cx="159" cy="566" r="18" />

        <g className="usb-connector">
          <path d="M24 149 H164 Q176 149 176 161 V268 Q176 280 164 280 H24Z" fill="url(#metal)" />
          <rect x="24" y="165" width="55" height="98" rx="4" />
          <path d="M82 164 H161 V268 H82 L70 252 V180Z" />
          <circle cx="157" cy="170" r="5" />
          <circle cx="157" cy="260" r="5" />
        </g>

        <g className="reset-button">
          <rect x="126" y="55" width="72" height="58" rx="5" />
          <circle cx="162" cy="84" r="21" />
          <text x="118" y="85" transform="rotate(-90 118 85)" textAnchor="middle">
            RESET
          </text>
        </g>

        <g className="barrel-connector">
          <path d="M54 447 H193 Q209 447 209 463 V564 Q209 580 193 580 H54Z" />
          <rect x="54" y="460" width="65" height="108" rx="7" />
          <circle cx="105" cy="514" r="28" />
          <circle cx="105" cy="514" r="15" />
        </g>

        <text className="external-header-label" x="523" y="27" textAnchor="middle">
          DIGITAL
        </text>
        <rect className="pin-header" x="240" y="49" width="307" height="38" rx="4" />
        <rect className="pin-header" x="560" y="49" width="245" height="38" rx="4" />

        {UNO_R3_AUXILIARY_HEADER.map((definition, index) => (
          <StaticHeaderSocket
            key={definition.label}
            definition={definition}
            x={auxiliaryPinCoordinate(index)}
            y={68}
            compact
          />
        ))}

        {UNO_R3_DIGITAL_PINS.map((definition, index) => {
          const { pin, capabilities, mcuPort, boardMarkings } = definition;
          const state = pins[pin];
          const high = state?.level === "high";
          const pwmCapable = capabilities.includes("pwm");
          const pwmActive = (pwm[pin]?.latest.dutyValue ?? 0) > 0;
          const selected =
            (activeTab === "digital" && selectedDigitalPin === pin) ||
            (activeTab === "pwm" && selectedPwmPin === pin);
          const selectable =
            activeTab === "digital" || (activeTab === "pwm" && pwmCapable);
          const x = digitalPinX(index);
          const selectPin = () => {
            if (!selectable) return;
            if (activeTab === "pwm") {
              onSelectPwmPin(pin);
            } else {
              onSelectDigitalPin(pin);
            }
          };
          const usesSerial = capabilities.includes("uart-rx") || capabilities.includes("uart-tx");
          const usesSpi = capabilities.some((capability) => capability.startsWith("spi-"));
          const usesInterrupt = capabilities.includes("external-interrupt");
          const usesLed = capabilities.includes("led");
          return (
            <g
              key={pin}
              className={`board-pin ${high ? "board-pin--high" : ""} ${
                selected ? "board-pin--selected" : ""
              } ${pwmCapable ? "board-pin--pwm-capable" : ""} ${
                activeTab === "pwm" && pwmActive ? "board-pin--pwm-active" : ""
              } ${activeTab === "pwm" && !pwmCapable ? "board-pin--unavailable" : ""} ${
                activeTab === "analog" ? "board-pin--inactive-mode" : ""
              } ${usesSerial ? "board-pin--serial" : ""} ${
                usesSpi ? "board-pin--spi" : ""
              } ${usesInterrupt ? "board-pin--interrupt" : ""}`}
              role="button"
              tabIndex={selectable ? 0 : -1}
              aria-disabled={!selectable}
              aria-label={`Digital pin D${pin}, ${mcuPort}, ${describeCapabilities(capabilities)}, ${state?.level ?? "not observed"}`}
              onClick={selectPin}
              onKeyDown={(event) => {
                if (selectable && (event.key === "Enter" || event.key === " ")) {
                  event.preventDefault();
                  selectPin();
                }
              }}
            >
              <title>
                D{pin} / {mcuPort}: {describeCapabilities(capabilities)}
              </title>
              <rect className="pin-hit-target" x={x - 13} y="44" width="26" height="91" fill="transparent" />
              <rect className="socket-body" x={x - 11} y="56" width="22" height="24" rx="2" />
              <circle cx={x} cy="68" r="6" />
              <text className="digital-pin-number" x={x} y="101" textAnchor="middle">
                {pin}
              </text>
              {boardMarkings?.map((marking, markingIndex) => (
                <text
                  key={marking}
                  className={`special-pin-label special-pin-label--${
                    markingIndex === 0 ? "primary" : "secondary"
                  } ${usesLed ? "special-pin-label--led" : ""}`}
                  x={x}
                  y={117 + markingIndex * 10}
                  textAnchor="middle"
                >
                  {marking}
                </text>
              ))}
            </g>
          );
        })}

        <g className="usb-bridge">
          <rect className="qfp-chip" x="205" y="184" width="86" height="86" rx="4" />
          {Array.from({ length: 7 }, (_, index) => (
            <g key={`usb-qfp-${index}`}>
              <rect x={215 + index * 10} y="175" width="5" height="9" />
              <rect x={215 + index * 10} y="270" width="5" height="9" />
              <rect x="196" y={194 + index * 10} width="9" height="5" />
              <rect x="291" y={194 + index * 10} width="9" height="5" />
            </g>
          ))}
          <circle cx="220" cy="199" r="4" />
          <text x="248" y="217" textAnchor="middle">ATmega</text>
          <text x="248" y="233" textAnchor="middle">16U2</text>
          <text className="component-purpose" x="248" y="253" textAnchor="middle">USB SERIAL</text>
        </g>

        <g className="main-mcu">
          <rect className="chip" x="438" y="350" width="372" height="126" rx="9" />
          {Array.from({ length: 14 }, (_, index) => (
            <g key={`mcu-leg-${index}`}>
              <rect x={454 + index * 25.5} y="340" width="9" height="10" rx="1" />
              <rect x={454 + index * 25.5} y="476" width="9" height="10" rx="1" />
            </g>
          ))}
          <path d="M438 396 q18 0 18 18 q0 18 -18 18" />
          <circle cx="782" cy="378" r="7" />
          <text className="chip-label" x="624" y="401" textAnchor="middle">ATMEGA328P</text>
          <text className="chip-subtitle" x="624" y="425" textAnchor="middle">8-BIT AVR MCU</text>
          <text className="chip-subtitle" x="624" y="448" textAnchor="middle">INSTRUMENTED BY ASV</text>
        </g>

        <g className="uno-silkscreen">
          <path className="infinity-mark" d="M518 187 C493 158 457 161 457 187 C457 213 493 216 518 187 C543 158 579 161 579 187 C579 213 543 216 518 187Z" />
          <path className="logo-minus" d="M475 187 H495" />
          <path className="logo-plus" d="M541 187 H561 M551 177 V197" />
          <text className="uno-word" x="635" y="197" textAnchor="middle">UNO</text>
          <text className="arduino-word" x="570" y="253" textAnchor="middle">ARDUINO</text>
          <text className="open-source-mark" x="570" y="272" textAnchor="middle">OPEN-SOURCE ELECTRONICS</text>
        </g>

        <g
          className="status-led tx-led"
          aria-label={`TX serial activity ${serialLeds.tx ? "active" : "inactive"}`}
        >
          <title>TX: bytes transmitted by the Uno USB bridge to the desktop</title>
          <circle className={serialLeds.tx ? "active" : ""} cx="315" cy="199" r="7" />
          <text x="315" y="218" textAnchor="middle">TX</text>
        </g>
        <g
          className="status-led rx-led"
          aria-label={`RX serial activity ${serialLeds.rx ? "active" : "inactive"}`}
        >
          <title>RX: bytes received by the Uno USB bridge from the desktop</title>
          <circle className={serialLeds.rx ? "active" : ""} cx="315" cy="229" r="7" />
          <text x="315" y="248" textAnchor="middle">RX</text>
        </g>
        <g
          className="status-led signal-led"
          aria-label={`L LED D13 ${pins[13]?.level === "high" ? "active" : "inactive"}`}
        >
          <title>L: instrumented digital state of D13 / SCK</title>
          <circle className={pins[13]?.level === "high" ? "active" : ""} cx="315" cy="259" r="7" />
          <text x="315" y="278" textAnchor="middle">L</text>
        </g>
        <g className="status-led power-led">
          <circle cx="786" cy="215" r="7" />
          <text x="786" y="234" textAnchor="middle">ON</text>
        </g>

        <rect className="pin-header bottom-header" x="337" y="544" width="245" height="38" rx="4" />
        {UNO_R3_POWER_HEADER.map((definition, index) => (
          <StaticHeaderSocket
            key={`${definition.label}-${index}`}
            definition={definition}
            x={351 + index * 31}
            y={563}
            compact
            labelPlacement="above"
          />
        ))}
        <text className="external-header-label" x="460" y="628" textAnchor="middle">POWER</text>

        <rect className="pin-header bottom-header" x="617" y="544" width="183" height="38" rx="4" />
        {UNO_R3_ANALOG_PINS.map((definition, index) => {
          const { channel, mcuPort, capabilities, boardMarking } = definition;
          const analogState = analog[channel];
          const analogLevel = analogState
            ? Math.max(
                0,
                Math.min(
                  1,
                  analogState.latest.rawValue /
                    adcFullScale(analogState.latest.resolutionBits),
                ),
              )
            : 0;
          const selected = activeTab === "analog" && selectedAnalogChannel === channel;
          const selectable = activeTab === "analog";
          const inputActive = selectable && analogState !== undefined;
          const x = 631 + index * 31;
          return (
            <g
              key={`analog-${channel}`}
              className={`board-pin board-pin--analog ${selected ? "board-pin--selected" : ""} ${
                selectable ? "" : "board-pin--inactive-mode"
              } ${inputActive ? "board-pin--analog-active" : ""} ${
                boardMarking ? "board-pin--i2c" : ""
              }`}
              role="button"
              tabIndex={selectable ? 0 : -1}
              aria-disabled={!selectable}
              aria-label={`Analog input A${channel}, digital D${definition.digitalPin}, ${mcuPort}${
                boardMarking ? `, I2C ${boardMarking}` : ""
              }, ${
                analogState
                  ? `input active at ${analogState.latest.rawValue} of ${adcFullScale(analogState.latest.resolutionBits)}`
                  : "not observed"
              }`}
              onClick={() => selectable && onSelectAnalogChannel(channel)}
              onKeyDown={(event) => {
                if (selectable && (event.key === "Enter" || event.key === " ")) {
                  event.preventDefault();
                  onSelectAnalogChannel(channel);
                }
              }}
            >
              <title>
                A{channel} / D{definition.digitalPin} / {mcuPort}: ADC input and GPIO{boardMarking ? `, I2C ${boardMarking}` : ""}{analogState ? `; live input ${analogState.latest.rawValue} / ${adcFullScale(analogState.latest.resolutionBits)}` : "; waiting for input samples"}
              </title>
              <rect className="pin-hit-target" x={x - 14} y="505" width="28" height="82" fill="transparent" />
              <rect className="socket-body" x={x - 11} y="551" width="22" height="24" rx="2" />
              {inputActive && (
                <circle
                  className="analog-activity-ring"
                  cx={x}
                  cy="563"
                  r="10"
                  style={{ opacity: 0.28 + analogLevel * 0.42 }}
                />
              )}
              <circle
                cx={x}
                cy="563"
                r="6"
                style={
                  analogState
                    ? {
                        fill: `rgb(106 169 255 / ${0.24 + analogLevel * 0.76})`,
                        filter: `drop-shadow(0 0 ${3 + analogLevel * 6}px rgb(106 169 255 / 82%))`,
                      }
                    : undefined
                }
              />
              <text className="analog-pin-number" x={x} y="533" textAnchor="middle">A{channel}</text>
              {boardMarking && <text className="analog-special-label" x={x} y="518" textAnchor="middle">{boardMarking}</text>}
            </g>
          );
        })}

        <text className="external-header-label" x="709" y="628" textAnchor="middle">ANALOG IN</text>
      </svg>
    </div>
  );
}
