param([Parameter(Mandatory)][string]$Runs, [switch]$Json)
$o = 'out/post-run-diag/user/sdmc/ultimate/ssbu_online_deluxe_rewrite/hang-trace.log'
$argv = foreach ($r in $Runs.Split(',')) {
    $label, $run = $r.Trim().Split('=', 2)
    if (-not $run) { $run = $label }
    "$label=/home/intpa/eden-runs/$run/$o"
}
Set-Location /home/intpa/deluxe-unicode-names-work/parity/ssbu-online-deluxe-rewrite/tools
if ($Json) { /usr/bin/python3 -B -s -E heartbeat_memory.py @argv --json } else { /usr/bin/python3 -B -s -E heartbeat_memory.py @argv }
