# Arduino firmware library

The library instruments operations explicitly called through `ASV`. It neither
simulates hardware nor observes unrelated calls made directly through the
Arduino core.

## API in Milestone 1

```cpp
ASV.begin();
ASV.pinMode(13, OUTPUT);
ASV.digitalWrite(13, HIGH);
int level = ASV.digitalRead(7);
```

The API follows the standard Arduino names to make migration visible and
mechanical.

## Compile the example

```powershell
arduino-cli compile --fqbn arduino:avr:uno `
  --library firmware/ArduinoSignalVisualizer `
  firmware/examples/GpioDemo
```

Pins D0 and D1 carry the Uno's hardware UART. Instrumenting those pins while
the ASV protocol uses USB serial will interfere with communication.

Digital voltage in the desktop UI is an estimate of either 0 V or the nominal
5 V logic supply; ASV does not measure digital-pin voltage.

