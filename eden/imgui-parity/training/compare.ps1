param([Parameter(Mandatory)][string]$Runs, [switch]$Json)
. "$PSScriptRoot/../guard.ps1"
$o = 'out/post-run-diag/user/sdmc/ultimate/ssbu_online_deluxe_rewrite/hang-trace.log'
$argv = foreach ($r in $Runs.Split(',')) {
    $label, $run = $r.Trim().Split('=', 2)
    if (-not $run) { $run = $label }
    "$label=/home/intpa/eden-runs/$run/$o"
}
$tool = Join-Path (Get-DeluxeRoot $PSScriptRoot) 'tools/heartbeat_memory.py'
if ($Json) { $argv += '--json' }
Invoke-Guarded 1G 300 /usr/bin/python3 -B -s -E $tool @argv
