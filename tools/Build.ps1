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
