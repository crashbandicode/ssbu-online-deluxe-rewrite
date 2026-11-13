# The playbook's memory and time guard (README "Commands": systemd-run --user --scope -p MemoryMax=... timeout ...)
# for every local command these scripts run, ssh and scp included. Dot-source it: . "$PSScriptRoot/../guard.ps1".
# $LASTEXITCODE is the command's own exit code, or 124 when the timeout ended it.
function Invoke-Guarded {
    param(
        [Parameter(Mandatory)][string]$Memory,
        [Parameter(Mandatory)][int]$Seconds,
        [Parameter(Mandatory, ValueFromRemainingArguments)][string[]]$Argv
    )
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
