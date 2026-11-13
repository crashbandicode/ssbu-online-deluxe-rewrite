param(
    [switch]$Package,
    # Coverage variant: the plugin crate is compiled with -C instrument-coverage
    # (tools/coverage/config.toml) and the `coverage` feature writes raw LLVM
    # profiles to the card. It builds into target/coverage so the product
    # build's artifacts are never mixed with it. See docs/DEBUGGING.md.
    [switch]$Coverage,
    # Package destination outside the checkout (package_debug.py --output).
    [string]$Output
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
# Free disk before a build (ssbu-mod-playbook WORKFLOW, "Build caches and disk"): stop below 40 GB.
function Get-FreeBytes([string]$Path) {
    if ($IsWindows) {
        return [IO.DriveInfo]::new([IO.Path]::GetPathRoot((Resolve-Path -LiteralPath $Path).Path)).AvailableFreeSpace
    }
    return [int64]((& df -B1 --output=avail $Path | Select-Object -Last 1).Trim())
}
$freeBytes = Get-FreeBytes $root
if ($freeBytes -lt 40GB) {
    throw "Only $([math]::Round($freeBytes / 1GB, 1)) GiB free under $root; the build needs at least 40 GiB. Delete finished build products first."
}
Write-Output "free disk: $([math]::Round($freeBytes / 1GB, 1)) GiB under $root"
$cargoDirectory = Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cargo/bin'
$env:PATH = $cargoDirectory + [IO.Path]::PathSeparator + $env:PATH
Push-Location -LiteralPath $root
try {
    $buildArguments = @('skyline', 'build', '--release')
    $packageArguments = @()
    if ($Coverage) {
        $targetDirectory = Join-Path $root 'target/coverage'
        $buildArguments += @(
            '--features', 'coverage', '--',
            '--config', (Join-Path $root 'tools/coverage/config.toml'),
            '--target-dir', $targetDirectory
        )
        $packageArguments += @('--target-dir', $targetDirectory, '--variant', 'coverage')
    }
    if ($Output) { $packageArguments += @('--output', $Output) }
    & (Join-Path $cargoDirectory 'cargo') @buildArguments
    if ($LASTEXITCODE -ne 0) { throw "Skyline build failed: $LASTEXITCODE" }
    if ($Package) {
        & /usr/bin/python3 (Join-Path $PSScriptRoot 'package_debug.py') @packageArguments
        if ($LASTEXITCODE -ne 0) { throw "Debug packaging failed: $LASTEXITCODE" }
    }
} finally {
    Pop-Location
}
