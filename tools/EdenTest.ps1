param(
    [ValidateSet('Start', 'Stop', 'Status')]
    [string]$Action = 'Status',
    [string]$EdenPath = '/home/intpa/Downloads/Eden-Linux-v0.2.1-amd64-clang-pgo.AppImage',
    [string]$GamePath
)

$ErrorActionPreference = 'Stop'
$unit = 'ssbu-deluxe-eden-test.service'

if ($Action -eq 'Stop') {
    # Only this test's cgroup; never stop another emulator or the desktop session.
    & systemctl --user stop $unit
    exit $LASTEXITCODE
}

if ($Action -eq 'Start') {
    $existing = @(Get-Process -Name eden -ErrorAction SilentlyContinue)
    if ($existing.Count -gt 0) {
        throw "Eden is already running (PID $($existing.Id -join ', ')). Exit it before starting another test."
    }
    & systemctl --user is-active --quiet $unit
    if ($LASTEXITCODE -eq 0) {
        throw "The test service is already active. Use -Action Stop before restarting."
    }
    if (-not (Test-Path -LiteralPath $EdenPath -PathType Leaf)) {
        throw "Eden executable not found: $EdenPath"
    }
    if ($GamePath -and -not (Test-Path -LiteralPath $GamePath -PathType Leaf)) {
        throw "Game file not found: $GamePath"
    }
    $runArgs = @(
        '--user', '--unit=ssbu-deluxe-eden-test', '--collect', '--service-type=exec',
        '--property=MemoryAccounting=yes', '--property=MemoryHigh=8G',
        '--property=MemoryMax=12G', '--property=MemorySwapMax=1G',
        '--property=KillMode=control-group', '--property=OOMPolicy=kill',
        '--property=TimeoutStopSec=10s'
    )
    foreach ($name in @('DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'XAUTHORITY', 'DBUS_SESSION_BUS_ADDRESS')) {
        $value = [Environment]::GetEnvironmentVariable($name)
        if ($value) { $runArgs += "--setenv=$name=$value" }
    }
    $edenArgs = @()
    if ($GamePath) { $edenArgs += @('-g', $GamePath) }
    & systemd-run @runArgs -- $EdenPath @edenArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

& systemctl --user show $unit --property=ActiveState,SubState,MainPID,ControlGroup,MemoryCurrent,MemoryPeak,MemoryHigh,MemoryMax,MemorySwapMax,Result
exit $LASTEXITCODE
