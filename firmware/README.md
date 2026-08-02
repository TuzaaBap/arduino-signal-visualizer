# Arduino firmware library

The library instruments operations explicitly called through `ASV`. It neither
simulates hardware nor observes unrelated calls made directly through the
Arduino core.

## Instrumentation API

```cpp
ASV.begin();
ASV.pinMode(13, OUTPUT);
ASV.digitalWrite(13, HIGH);
int level = ASV.digitalRead(7);
int raw = ASV.analogRead(A0);
```

The API follows the standard Arduino names to make migration visible and
mechanical.

`ASV.analogRead()` returns the Arduino core result unchanged, then reports the
channel, raw count, resolution, reference mode, integer reference millivolts,
and board timestamp. The Uno does not calculate or transmit floating-point
voltage values.

## Compile the example

```powershell
arduino-cli compile --fqbn arduino:avr:uno `
  --library firmware/ArduinoSignalVisualizer `
firmware/examples/GpioDemo
```

Compile the Milestone 2 ADC example:

```powershell
arduino-cli compile --fqbn arduino:avr:uno `
  --library firmware/ArduinoSignalVisualizer `
  firmware/examples/AdcDemo
```

PlatformIO keeps the two build and upload targets explicit:

```powershell
platformio run -e uno_gpio_demo
platformio run --project-conf platformio-adc.ini -e uno_adc_demo
```

Pins D0 and D1 carry the Uno's hardware UART. Instrumenting those pins while
the ASV protocol uses USB serial will interfere with communication.

Digital voltage in the desktop UI is an estimate of either 0 V or the nominal
5 V logic supply; ASV does not measure digital-pin voltage.

ADC voltage is also an estimate. It is calculated on the desktop from the raw
count and declared reference metadata; the default 5000 mV value is not a
calibrated measurement of the Uno supply or AREF pin.
