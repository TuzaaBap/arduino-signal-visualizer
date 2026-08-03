import { UNO_PWM_PINS } from "./pwm-store";

export type UnoPinCapability =
  | "gpio"
  | "adc"
  | "pwm"
  | "uart-rx"
  | "uart-tx"
  | "spi-ss"
  | "spi-mosi"
  | "spi-miso"
  | "spi-sck"
  | "i2c-sda"
  | "i2c-scl"
  | "external-interrupt"
  | "led";

export interface UnoDigitalPinDefinition {
  readonly pin: number;
  readonly mcuPort: string;
  readonly capabilities: readonly UnoPinCapability[];
  readonly boardMarkings?: readonly string[];
}

export interface UnoAnalogPinDefinition {
  readonly channel: number;
  readonly digitalPin: number;
  readonly mcuPort: string;
  readonly capabilities: readonly UnoPinCapability[];
  readonly boardMarking?: string;
}

export interface UnoHeaderPinDefinition {
  readonly label: string;
  readonly description: string;
  readonly aliasOf?: string;
}

const specialDigitalCapabilities: Readonly<
  Record<number, readonly UnoPinCapability[]>
> = {
  0: ["uart-rx"],
  1: ["uart-tx"],
  2: ["external-interrupt"],
  3: ["external-interrupt"],
  10: ["spi-ss"],
  11: ["spi-mosi"],
  12: ["spi-miso"],
  13: ["led", "spi-sck"],
};

const digitalPort = (pin: number): string => {
  if (pin <= 7) return `PD${pin}`;
  return `PB${pin - 8}`;
};

const digitalMarkings = (pin: number): readonly string[] | undefined => {
  const markings: Readonly<Record<number, readonly string[]>> = {
    0: ["RX"],
    1: ["TX"],
    2: ["INT 0"],
    3: ["PWM", "INT 1"],
    5: ["PWM"],
    6: ["PWM"],
    9: ["PWM"],
    10: ["PWM", "SS"],
    11: ["PWM", "MOSI"],
    12: ["MISO"],
    13: ["LED", "SCK"],
  };
  return markings[pin];
};

/** Physical left-to-right order of the Uno R3 digital header. */
export const UNO_R3_DIGITAL_PINS: readonly UnoDigitalPinDefinition[] =
  Array.from({ length: 14 }, (_, index): UnoDigitalPinDefinition => {
    const pin = 13 - index;
    const pwm = UNO_PWM_PINS.includes(pin as (typeof UNO_PWM_PINS)[number]);
    return {
      pin,
      mcuPort: digitalPort(pin),
      capabilities: [
        "gpio",
        ...(pwm ? (["pwm"] as const) : []),
        ...(specialDigitalCapabilities[pin] ?? []),
      ],
      boardMarkings: digitalMarkings(pin),
    };
  });

/** Physical left-to-right order of the Uno R3 analog header. */
export const UNO_R3_ANALOG_PINS: readonly UnoAnalogPinDefinition[] =
  Array.from({ length: 6 }, (_, channel): UnoAnalogPinDefinition => ({
    channel,
    digitalPin: channel + 14,
    mcuPort: `PC${channel}`,
    capabilities: [
      "gpio",
      "adc",
      ...(channel === 4 ? (["i2c-sda"] as const) : []),
      ...(channel === 5 ? (["i2c-scl"] as const) : []),
    ],
    boardMarking: channel === 4 ? "SDA" : channel === 5 ? "SCL" : undefined,
  }));

/** R3-only auxiliary pins immediately before D13 on the top header. */
export const UNO_R3_AUXILIARY_HEADER: readonly UnoHeaderPinDefinition[] = [
  { label: "SCL", description: "I2C clock", aliasOf: "A5 / D19" },
  { label: "SDA", description: "I2C data", aliasOf: "A4 / D18" },
  { label: "AREF", description: "External analog reference" },
  { label: "GND", description: "Ground" },
];

export const UNO_R3_POWER_HEADER: readonly UnoHeaderPinDefinition[] = [
  { label: "NC", description: "Reserved, not connected" },
  { label: "IOREF", description: "I/O voltage reference" },
  { label: "RESET", description: "Active-low reset" },
  { label: "3V3", description: "3.3 V supply" },
  { label: "5V", description: "5 V supply" },
  { label: "GND", description: "Ground" },
  { label: "GND", description: "Ground" },
  { label: "VIN", description: "External input voltage" },
];

/** Standard AVR 2x3 ICSP header, read row-by-row from its pin-1 end. */
export const UNO_R3_ICSP_HEADER: readonly UnoHeaderPinDefinition[] = [
  { label: "MISO", description: "SPI controller-in peripheral-out" },
  { label: "5V", description: "5 V supply" },
  { label: "SCK", description: "SPI clock" },
  { label: "MOSI", description: "SPI controller-out peripheral-in" },
  { label: "RESET", description: "Active-low reset" },
  { label: "GND", description: "Ground" },
];

export const UNO_R3_CAPABILITY_LABELS: Readonly<
  Partial<Record<UnoPinCapability, string>>
> = {
  pwm: "PWM",
  "uart-rx": "UART RX",
  "uart-tx": "UART TX",
  "spi-ss": "SPI SS",
  "spi-mosi": "SPI MOSI",
  "spi-miso": "SPI MISO",
  "spi-sck": "SPI SCK",
  "i2c-sda": "I2C SDA",
  "i2c-scl": "I2C SCL",
  "external-interrupt": "external interrupt",
  led: "on-board L LED",
};

export function describeCapabilities(
  capabilities: readonly UnoPinCapability[],
): string {
  const labels = capabilities
    .filter((capability) => capability !== "gpio" && capability !== "adc")
    .map((capability) => UNO_R3_CAPABILITY_LABELS[capability] ?? capability);
  return labels.length === 0 ? "GPIO" : `GPIO, ${labels.join(", ")}`;
}
