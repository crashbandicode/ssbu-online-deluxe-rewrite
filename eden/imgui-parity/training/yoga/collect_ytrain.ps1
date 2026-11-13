param([string[]]$Labels)
# Copies the Yoga Training runs' RESULTS.md and Deluxe files into the artifacts, then compares memory per phase with
# Deluxe's tools/heartbeat_memory.py (hang-card-r5, the integration lane's Yoga run of variant b, as reference).
# Every scp and the analysis run under the playbook's memory/time guard (../../guard.ps1).
. "$PSScriptRoot/../../guard.ps1"
$y = 'memento-claw-yoga-ubuntu'
$dest = '/home/intpa/smash-training-lobby/artifacts/imgui-parity/eden'
$sub = 'out/post-run-diag/user/sdmc/ultimate/ssbu_online_deluxe_rewrite'
$argv = @()
foreach ($label in @('hang-card-r5') + $Labels) {
    $to = "$dest/$label"
    New-Item -ItemType Directory -Force "$to/post-run" | Out-Null
    Invoke-Guarded 512M 120 /usr/bin/scp -q -o BatchMode=yes "${y}:/home/intpa/eden-runs/$label/RESULTS.md" "$to/RESULTS.md"
    foreach ($f in 'hang-trace.log', 'renderer-status.txt', 'install.log') {
        Invoke-Guarded 512M 120 /usr/bin/scp -q -o BatchMode=yes "${y}:/home/intpa/eden-runs/$label/$sub/$f" "$to/post-run/ssbu_online_deluxe_rewrite-$f"
    }
    if (Test-Path "$to/post-run/ssbu_online_deluxe_rewrite-hang-trace.log") {
        $argv += "$label=$to/post-run/ssbu_online_deluxe_rewrite-hang-trace.log"
    }
    "$label rows: " + ((Get-Content "$to/RESULTS.md" -ErrorAction SilentlyContinue | Select-String 'Rows:') -replace '\s+', ' ')
}
$tool = Join-Path (Resolve-Path "$PSScriptRoot/../../../..").Path 'tools/heartbeat_memory.py'
Invoke-Guarded 1G 300 /usr/bin/python3 -B -I $tool @argv
