param([switch]$Package)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$cargoDirectory = Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cargo/bin'
$env:PATH = $cargoDirectory + [IO.Path]::PathSeparator + $env:PATH
Push-Location -LiteralPath $root
try {
    & (Join-Path $cargoDirectory 'cargo') skyline build --release
    if ($LASTEXITCODE -ne 0) { throw "Skyline build failed: $LASTEXITCODE" }
    if ($Package) {
        & /usr/bin/python3 (Join-Path $PSScriptRoot 'package_debug.py')
        if ($LASTEXITCODE -ne 0) { throw "Debug packaging failed: $LASTEXITCODE" }
    }
} finally {
    Pop-Location
}
