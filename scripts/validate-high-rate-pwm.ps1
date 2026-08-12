param(
  [string]$Port = "COM6",
  [int]$DurationSeconds = 600,
  [int]$BaudRate = 115200
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot
$validator = Join-Path $workspace "target\release\arduino-signal-visualizer.exe"
$library = Join-Path $workspace "firmware\ArduinoSignalVisualizer"
$sketch = Join-Path $workspace "firmware\tests\sketches\HighRateMultiPwm"
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$output = Join-Path $workspace "outputs\high-rate-pwm-$stamp"
$reportRoot = Join-Path $workspace "work\hardware-validation\high-rate-pwm-$stamp"
$reportPath = Join-Path $reportRoot "report.json"
$memoryPath = Join-Path $reportRoot "memory.csv"

if ($DurationSeconds -lt 60) {
  throw "DurationSeconds must be at least 60"
}
if (-not (Test-Path -LiteralPath $validator)) {
  throw "Hardware-validation executable not found: $validator"
}

New-Item -ItemType Directory -Path $output -Force | Out-Null
New-Item -ItemType Directory -Path $reportRoot -Force | Out-Null

& arduino-cli compile --fqbn arduino:avr:uno --library $library --warnings all --output-dir $output $sketch
if ($LASTEXITCODE -ne 0) {
  throw "High-rate PWM compile failed with exit code $LASTEXITCODE"
}

$boardList = & arduino-cli board list --format json | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) {
  throw "Board detection failed before upload"
}
$detected = @($boardList.detected_ports | Where-Object { $_.port.address -eq $Port })
if ($detected.Count -ne 1) {
  throw "Expected one board on $Port, found $($detected.Count)"
}
$uno = @($detected[0].matching_boards | Where-Object { $_.fqbn -eq "arduino:avr:uno" })
if ($uno.Count -lt 1) {
  throw "$Port was not identified as arduino:avr:uno"
}

& arduino-cli upload --fqbn arduino:avr:uno --port $Port --input-dir $output $sketch
if ($LASTEXITCODE -ne 0) {
  throw "High-rate PWM upload failed with exit code $LASTEXITCODE"
}

$env:ASV_VALIDATION_REPORT = $reportPath
$env:ASV_VALIDATION_PORT = $Port
$env:ASV_VALIDATION_RECONNECT_AFTER_SECS = [Math]::Floor($DurationSeconds / 2).ToString()
$startedAt = Get-Date
$samples = @()
$app = Start-Process -FilePath $validator -WindowStyle Hidden -PassThru
try {
  while (((Get-Date) - $startedAt).TotalSeconds -lt $DurationSeconds) {
    Start-Sleep -Seconds 10
    $app.Refresh()
    if ($app.HasExited) {
      throw "Desktop application exited during the high-rate PWM soak"
    }
    $process = Get-Process -Id $app.Id
    $samples += [pscustomobject]@{
      elapsedSeconds = [Math]::Round(((Get-Date) - $startedAt).TotalSeconds, 1)
      workingSetBytes = $process.WorkingSet64
      privateMemoryBytes = $process.PrivateMemorySize64
    }
  }
} finally {
  $app.Refresh()
  if (-not $app.HasExited) {
    Stop-Process -Id $app.Id -Force
    $app.WaitForExit()
  }
  Remove-Item Env:\ASV_VALIDATION_REPORT -ErrorAction SilentlyContinue
  Remove-Item Env:\ASV_VALIDATION_PORT -ErrorAction SilentlyContinue
  Remove-Item Env:\ASV_VALIDATION_RECONNECT_AFTER_SECS -ErrorAction SilentlyContinue
}

$samples | Export-Csv -LiteralPath $memoryPath -NoTypeInformation
if (-not (Test-Path -LiteralPath $reportPath)) {
  throw "Validation report was not created"
}

function Get-SlopeBytesPerMinute {
  param(
    [object[]]$Points,
    [string]$PropertyName
  )

  if ($Points.Count -lt 3) {
    return 0.0
  }

  $meanX = ($Points | Measure-Object -Property elapsedSeconds -Average).Average
  $meanY = ($Points | Measure-Object -Property $PropertyName -Average).Average
  $numerator = 0.0
  $denominator = 0.0
  foreach ($point in $Points) {
    $xDelta = [double]$point.elapsedSeconds - $meanX
    $yDelta = [double]$point.$PropertyName - $meanY
    $numerator += $xDelta * $yDelta
    $denominator += $xDelta * $xDelta
  }

  if ($denominator -eq 0.0) {
    return 0.0
  }
  return ($numerator / $denominator) * 60.0
}

$warmSamples = @($samples | Select-Object -Skip ([Math]::Floor($samples.Count / 2)))
$workingSetSlope = Get-SlopeBytesPerMinute -Points $warmSamples -PropertyName "workingSetBytes"
$privateMemorySlope = Get-SlopeBytesPerMinute -Points $warmSamples -PropertyName "privateMemoryBytes"
$warmPrivateGrowth = [long]$warmSamples[-1].privateMemoryBytes - [long]$warmSamples[0].privateMemoryBytes

# A real leak remains positive after warm-up. Permit ordinary allocator noise, but
# reject a sustained private-memory rise in the second half of a long soak.
if ($DurationSeconds -ge 600 -and
    $warmPrivateGrowth -gt 2097152 -and
    $privateMemorySlope -gt 131072) {
  throw "Private memory kept growing after warm-up"
}

$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
$connectedCount = @($report.statusHistory | Where-Object { $_.phase -eq "connected" }).Count
$requiredPwmPins = @(3, 5, 6, 9, 10, 11)
$missingPwmPins = @($requiredPwmPins | Where-Object {
  $null -eq $report.pwmPins.PSObject.Properties[$_.ToString()]
})

if ($report.applicationVersion -ne "0.6.0" -or
    $report.board.firmwareVersion.major -ne 0 -or
    $report.board.firmwareVersion.minor -ne 6 -or
    $report.board.firmwareVersion.patch -ne 0) {
  throw "Application or firmware version mismatch"
}
if ($connectedCount -lt 2) {
  throw "Disconnect/reconnect recovery was not observed"
}
if ($report.diagnostics.Count -ne 0 -or
    $report.crcFailures -ne 0 -or
    $report.droppedPacketWarnings -ne 0 -or
    $report.droppedUserSerialBytes -ne 0) {
  throw "Transport integrity failed during the high-rate PWM soak"
}
if ($missingPwmPins.Count -ne 0 -or
    $report.receivedPwmUpdates -eq 0 -or
    $report.receivedGpioUpdates -eq 0 -or
    $report.receivedUserSerialBytes -eq 0 -or
    -not $report.uiPwmMatchObserved -or
    -not $report.uiGpioMatchObserved) {
  throw "Expected high-rate PWM, GPIO, Serial, or UI synchronization was missing"
}
if ($report.maximumUiPwmBufferLength -gt 180) {
  throw "PWM UI buffer exceeded its bound"
}

$result = [pscustomobject]@{
  schemaVersion = 1
  createdAt = (Get-Date).ToString("o")
  durationSeconds = $DurationSeconds
  port = $Port
  applicationVersion = $report.applicationVersion
  firmwareVersion = "0.6.0"
  connectedSessions = $connectedCount
  gpioUpdates = $report.receivedGpioUpdates
  pwmUpdates = $report.receivedPwmUpdates
  userSerialBytes = $report.receivedUserSerialBytes
  diagnostics = $report.diagnostics.Count
  crcFailures = $report.crcFailures
  droppedPacketWarnings = $report.droppedPacketWarnings
  droppedUserSerialBytes = $report.droppedUserSerialBytes
  maximumUiPwmBufferLength = $report.maximumUiPwmBufferLength
  workingSetStart = $samples[0].workingSetBytes
  workingSetEnd = $samples[-1].workingSetBytes
  privateMemoryStart = $samples[0].privateMemoryBytes
  privateMemoryEnd = $samples[-1].privateMemoryBytes
  warmPrivateMemoryGrowthBytes = $warmPrivateGrowth
  warmWorkingSetSlopeBytesPerMinute = [Math]::Round($workingSetSlope, 2)
  warmPrivateMemorySlopeBytesPerMinute = [Math]::Round($privateMemorySlope, 2)
  result = "passed"
}
$summaryPath = Join-Path $reportRoot "summary.json"
$result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $summaryPath -Encoding utf8
Write-Output "HIGH_RATE_PWM_SOAK_PASS"
Write-Output "HIGH_RATE_PWM_SOAK_REPORT $summaryPath"
