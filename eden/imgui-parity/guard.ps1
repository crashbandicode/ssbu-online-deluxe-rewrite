# The playbook's memory and time guard (README "Commands": systemd-run --user --scope -p MemoryMax=... timeout ...)
# for every local command these scripts run, ssh and scp included. Dot-source it: . "$PSScriptRoot/../guard.ps1".
# $LASTEXITCODE is the command's own exit code, or 124 when the timeout ended it.
# Usage: Invoke-Guarded <memory> <seconds> <command> [arguments...]. A plain function over $args, not an advanced
# one: the command's own switches (python3 -E, -I, ...) must not be bound as PowerShell's common parameters.
function Invoke-Guarded {
    $Memory, $Seconds, $Argv = $args
    $Argv = @($Argv | ForEach-Object { $_ })
    if (-not $Memory -or -not $Seconds -or $Argv.Count -eq 0) { throw 'Invoke-Guarded <memory> <seconds> <command> [arguments...]' }
    & /usr/bin/systemd-run --user --scope --quiet -p "MemoryMax=$Memory" /usr/bin/timeout $Seconds @Argv
}

# The same guard on the remote host, for a command line that ssh hands to the remote shell.
function Format-RemoteGuarded([string]$Memory, [int]$Seconds, [string]$Command) {
    return "systemd-run --user --scope --quiet -p MemoryMax=$Memory /usr/bin/timeout $Seconds $Command"
}

# The Deluxe checkout these scripts live in (eden/imgui-parity/<dir>/ -> repo root).
function Get-DeluxeRoot([string]$ScriptRoot) {
    return (Resolve-Path (Join-Path $ScriptRoot '../../..')).Path
}
