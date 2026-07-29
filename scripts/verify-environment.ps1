$ErrorActionPreference = "Stop"

$requirements = @(
  @{ Name = "Git"; Command = "git"; Arguments = @("--version") },
  @{ Name = "Node.js"; Command = "node"; Arguments = @("--version") },
  @{ Name = "npm"; Command = "npm.cmd"; Arguments = @("--version") },
  @{ Name = "Rust"; Command = "rustc"; Arguments = @("--version") },
  @{ Name = "Cargo"; Command = "cargo"; Arguments = @("--version") },
  @{ Name = "Arduino CLI"; Command = "arduino-cli"; Arguments = @("version") }
)

$failed = $false
foreach ($requirement in $requirements) {
  $command = Get-Command $requirement.Command -ErrorAction SilentlyContinue
  if (-not $command) {
    Write-Host "[missing] $($requirement.Name)"
    $failed = $true
    continue
  }

  $version = & $requirement.Command @($requirement.Arguments)
  Write-Host "[ok] $($requirement.Name): $version"
}

$webView = Get-ItemProperty `
  -Path "HKCU:\Software\Microsoft\EdgeUpdate\Clients\*", `
        "HKLM:\Software\Microsoft\EdgeUpdate\Clients\*", `
        "HKLM:\Software\WOW6432Node\Microsoft\EdgeUpdate\Clients\*" `
  -ErrorAction SilentlyContinue |
  Where-Object { $_.name -like "*WebView2*" } |
  Select-Object -First 1

if ($webView) {
  Write-Host "[ok] WebView2 Runtime: $($webView.pv)"
} else {
  Write-Host "[missing] WebView2 Runtime"
  $failed = $true
}

$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path -LiteralPath $vsWhere) {
  $installation = & $vsWhere -latest -products * `
    -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
    -property installationPath
  if ($installation) {
    Write-Host "[ok] MSVC C++ tools: $installation"
  } else {
    Write-Host "[missing] MSVC C++ build tools"
    $failed = $true
  }
} else {
  Write-Host "[missing] Visual Studio Installer"
  $failed = $true
}

if ($failed) {
  exit 1
}

Write-Host "Environment is ready."

