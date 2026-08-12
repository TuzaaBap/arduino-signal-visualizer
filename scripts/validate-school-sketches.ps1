param(
  [string]$Port = "COM6",
  [int]$BaudRate = 115200
)

$ErrorActionPreference = "Stop"
$workspace = if ([string]::IsNullOrWhiteSpace($PSScriptRoot)) {
  (Get-Location).Path
} else {
  Split-Path -Parent $PSScriptRoot
}
$validator = Join-Path $workspace "target\release\arduino-signal-visualizer.exe"
$library = Join-Path $workspace "firmware\ArduinoSignalVisualizer"
$sketchRoot = Join-Path $workspace "firmware\tests\school-sketches"
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$firmwareRoot = Join-Path $workspace "outputs\school-validation-$stamp"
$reportRoot = Join-Path $workspace "work\hardware-validation\school-suite-$stamp"

if (-not (Test-Path -LiteralPath $validator)) {
  throw "Hardware-validation desktop executable not found: $validator"
}

New-Item -ItemType Directory -Path $firmwareRoot -Force | Out-Null
New-Item -ItemType Directory -Path $reportRoot -Force | Out-Null

$allDigitalPins = 2..13
$allPwmPins = @(3, 5, 6, 9, 10, 11)
$tests = @(
  [pscustomobject]@{ Name = "School01Empty"; Seconds = 5; Gpio = @(); Toggle = @(); Adc = @(); Pwm = @(); Serial = $false },
  [pscustomobject]@{ Name = "School02Blink"; Seconds = 6; Gpio = @(13); Toggle = @(13); Adc = @(); Pwm = @(); Serial = $false },
  [pscustomobject]@{ Name = "School03TwoLeds"; Seconds = 5; Gpio = @(12, 13); Toggle = @(12, 13); Adc = @(); Pwm = @(); Serial = $false },
  [pscustomobject]@{ Name = "School04TrafficLights"; Seconds = 9; Gpio = @(8, 9, 10); Toggle = @(8, 9, 10); Adc = @(); Pwm = @(); Serial = $false },
  [pscustomobject]@{ Name = "School05AllDigitalPins"; Seconds = 5; Gpio = $allDigitalPins; Toggle = $allDigitalPins; Adc = @(); Pwm = @(); Serial = $false },
  [pscustomobject]@{ Name = "School06RunningLight"; Seconds = 6; Gpio = $allDigitalPins; Toggle = $allDigitalPins; Adc = @(); Pwm = @(); Serial = $false },
  [pscustomobject]@{ Name = "School07InputPullup"; Seconds = 6; Gpio = @(2, 13); Toggle = @(); Adc = @(); Pwm = @(); Serial = $false },
  [pscustomobject]@{ Name = "School08SerialCounter"; Seconds = 5; Gpio = @(); Toggle = @(); Adc = @(); Pwm = @(); Serial = $true },
  [pscustomobject]@{ Name = "School09SerialBlink"; Seconds = 6; Gpio = @(13); Toggle = @(13); Adc = @(); Pwm = @(); Serial = $true },
  [pscustomobject]@{ Name = "School10AnalogRead"; Seconds = 5; Gpio = @(); Toggle = @(); Adc = @(0); Pwm = @(); Serial = $true },
  [pscustomobject]@{ Name = "School11AnalogThreshold"; Seconds = 5; Gpio = @(13); Toggle = @(); Adc = @(0); Pwm = @(); Serial = $true },
  [pscustomobject]@{ Name = "School12PwmFade"; Seconds = 7; Gpio = @(); Toggle = @(); Adc = @(); Pwm = @(9); Serial = $false },
  [pscustomobject]@{ Name = "School13AllPwmPins"; Seconds = 6; Gpio = @(); Toggle = @(); Adc = @(); Pwm = $allPwmPins; Serial = $false },
  [pscustomobject]@{ Name = "School14MixedDashboard"; Seconds = 5; Gpio = @(13); Toggle = @(); Adc = @(0); Pwm = @(9); Serial = $true },
  [pscustomobject]@{ Name = "School15DigitalSweep"; Seconds = 6; Gpio = $allDigitalPins; Toggle = $allDigitalPins; Adc = @(); Pwm = @(); Serial = $false }
)

function Assert-NativeSuccess([string]$Operation) {
  if ($LASTEXITCODE -ne 0) {
    throw "$Operation failed with exit code $LASTEXITCODE"
  }
}

function Get-ReportEntry($Collection, [int]$Key) {
  $property = $Collection.PSObject.Properties[$Key.ToString()]
  if ($null -eq $property) {
    return $null
  }
  return $property.Value
}

Get-Process -Name "arduino-signal-visualizer" -ErrorAction SilentlyContinue |
  Stop-Process -Force

$results = @()
foreach ($test in $tests) {
  Write-Output "SCHOOL_TEST_START $($test.Name)"
  $sketch = Join-Path $sketchRoot $test.Name
  $output = Join-Path $firmwareRoot $test.Name
  $reportPath = Join-Path $reportRoot "$($test.Name).json"
  New-Item -ItemType Directory -Path $output -Force | Out-Null

  & arduino-cli compile --fqbn arduino:avr:uno --library $library --warnings all --output-dir $output $sketch
  Assert-NativeSuccess "Compile $($test.Name)"

  $boardList = & arduino-cli board list --format json | ConvertFrom-Json
  Assert-NativeSuccess "Board detection before $($test.Name) upload"
  $detected = @($boardList.detected_ports | Where-Object { $_.port.address -eq $Port })
  if ($detected.Count -ne 1) {
    throw "Expected one board on $Port before $($test.Name), found $($detected.Count)"
  }
  $uno = @($detected[0].matching_boards | Where-Object { $_.fqbn -eq "arduino:avr:uno" })
  if ($uno.Count -lt 1) {
    throw "$Port was not identified as arduino:avr:uno before $($test.Name)"
  }

  & arduino-cli upload --fqbn arduino:avr:uno --port $Port --input-dir $output $sketch
  Assert-NativeSuccess "Upload $($test.Name)"

  $env:ASV_VALIDATION_REPORT = $reportPath
  $env:ASV_VALIDATION_PORT = $Port
  $env:ASV_VALIDATION_RECONNECT_AFTER_SECS = "300"
  $app = Start-Process -FilePath $validator -PassThru
  try {
    Start-Sleep -Seconds $test.Seconds
    $app.Refresh()
    if ($app.HasExited) {
      throw "Desktop application exited during $($test.Name)"
    }
    if (-not (Test-Path -LiteralPath $reportPath)) {
      throw "Validation report was not created for $($test.Name)"
    }
    $report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
  } finally {
    $app.Refresh()
    if (-not $app.HasExited) {
      Stop-Process -Id $app.Id -Force
    }
  }

  if ($report.applicationVersion -ne "0.6.0" -or
      $report.board.firmwareVersion.major -ne 0 -or
      $report.board.firmwareVersion.minor -ne 6 -or
      $report.board.firmwareVersion.patch -ne 0) {
    throw "Version mismatch in $($test.Name)"
  }
  if ($report.statusHistory.phase -contains "error" -or
      $report.statusHistory.phase -notcontains "connected") {
    throw "Connection failed in $($test.Name)"
  }
  if ($report.diagnostics.Count -ne 0 -or
      $report.crcFailures -ne 0 -or
      $report.droppedPacketWarnings -ne 0 -or
      $report.droppedUserSerialBytes -ne 0) {
    throw "Transport integrity failed in $($test.Name)"
  }

  foreach ($pin in $test.Gpio) {
    $state = Get-ReportEntry $report.pins $pin
    if ($null -eq $state -or $state.backendUpdateCount -eq 0) {
      throw "D$pin was not observed in $($test.Name)"
    }
  }
  foreach ($pin in $test.Toggle) {
    $state = Get-ReportEntry $report.pins $pin
    if ($state.highObservations -eq 0 -or $state.lowObservations -eq 0) {
      throw "D$pin did not show both levels in $($test.Name)"
    }
  }
  foreach ($channel in $test.Adc) {
    $state = Get-ReportEntry $report.analogChannels $channel
    if ($null -eq $state -or $state.sampleCount -eq 0) {
      throw "A$channel was not observed in $($test.Name)"
    }
  }
  foreach ($pin in $test.Pwm) {
    $state = Get-ReportEntry $report.pwmPins $pin
    if ($null -eq $state -or $state.updateCount -eq 0) {
      throw "PWM D$pin was not observed in $($test.Name)"
    }
  }
  if ($test.Serial -and $report.receivedUserSerialBytes -eq 0) {
    throw "User Serial was not observed in $($test.Name)"
  }
  if ($test.Gpio.Count -gt 0 -and -not $report.uiGpioMatchObserved) {
    throw "GPIO UI did not synchronize in $($test.Name)"
  }
  if ($test.Adc.Count -gt 0 -and -not $report.uiAdcMatchObserved) {
    throw "ADC UI did not synchronize in $($test.Name)"
  }
  if ($test.Pwm.Count -gt 0 -and -not $report.uiPwmMatchObserved) {
    throw "PWM UI did not synchronize in $($test.Name)"
  }

  $results += [pscustomobject]@{
    name = $test.Name
    durationSeconds = $test.Seconds
    gpioUpdates = $report.receivedGpioUpdates
    adcSamples = $report.receivedAdcSamples
    pwmUpdates = $report.receivedPwmUpdates
    userSerialBytes = $report.receivedUserSerialBytes
    diagnostics = $report.diagnostics.Count
    crcFailures = $report.crcFailures
    droppedPacketWarnings = $report.droppedPacketWarnings
    droppedUserSerialBytes = $report.droppedUserSerialBytes
    result = "passed"
  }
  Write-Output "SCHOOL_TEST_PASS $($test.Name)"
}

$summary = [pscustomobject]@{
  schemaVersion = 1
  createdAt = (Get-Date).ToString("o")
  port = $Port
  fqbn = "arduino:avr:uno"
  applicationVersion = "0.6.0"
  firmwareVersion = "0.6.0"
  testCount = $results.Count
  passed = $results.Count
  failed = 0
  results = $results
}
$summaryPath = Join-Path $reportRoot "summary.json"
$summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $summaryPath -Encoding utf8
Write-Output "SCHOOL_SUITE_PASS $($results.Count)/$($tests.Count)"
Write-Output "SCHOOL_SUITE_REPORT $summaryPath"
