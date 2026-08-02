param(
    [Parameter(Mandatory = $true)]
    [int]$ProcessId,

    [Parameter(Mandatory = $true)]
    [ValidateRange(1, 86400)]
    [int]$DurationSeconds,

    [Parameter(Mandatory = $true)]
    [ValidateRange(1, 3600)]
    [int]$IntervalSeconds,

    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"
$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$parentDirectory = [System.IO.Path]::GetDirectoryName($resolvedOutput)
if ($parentDirectory) {
    [System.IO.Directory]::CreateDirectory($parentDirectory) | Out-Null
}

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$samples = [System.Collections.Generic.List[object]]::new()

while ($stopwatch.Elapsed.TotalSeconds -le $DurationSeconds) {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    $samples.Add([pscustomobject]@{
        TimestampUtc = [DateTime]::UtcNow.ToString("O")
        ElapsedSeconds = [Math]::Round($stopwatch.Elapsed.TotalSeconds, 3)
        WorkingSetBytes = $process.WorkingSet64
        PrivateMemoryBytes = $process.PrivateMemorySize64
    })
    $samples | Export-Csv -LiteralPath $resolvedOutput -NoTypeInformation

    $remaining = $DurationSeconds - $stopwatch.Elapsed.TotalSeconds
    if ($remaining -le 0) {
        break
    }
    Start-Sleep -Seconds ([Math]::Min($IntervalSeconds, [Math]::Ceiling($remaining)))
}

if ($samples.Count -eq 0 -or $samples[$samples.Count - 1].ElapsedSeconds -lt $DurationSeconds) {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    $samples.Add([pscustomobject]@{
        TimestampUtc = [DateTime]::UtcNow.ToString("O")
        ElapsedSeconds = [Math]::Round($stopwatch.Elapsed.TotalSeconds, 3)
        WorkingSetBytes = $process.WorkingSet64
        PrivateMemoryBytes = $process.PrivateMemorySize64
    })
    $samples | Export-Csv -LiteralPath $resolvedOutput -NoTypeInformation
}
