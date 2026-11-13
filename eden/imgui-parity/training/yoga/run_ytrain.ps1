param([string]$Order = 'fc,fa,fc,fa', [int]$GapSeconds = 180, [string]$AfterPair = '')
# Yoga Training memory A/B of the fixed build, one Yoga slot, one boot at a time: waits for a read-only preflight,
# runs eden_test.py all on the Yoga, keeps a gap after each release. Every ssh runs under the playbook's memory/time
# guard here (../../guard.ps1), and the remote commands under the same guard on the Yoga.
$ErrorActionPreference = 'Continue'
. "$PSScriptRoot/../../guard.ps1"
$root = '/tmp/opencode/imgui-parity/ytrain'
$log = "$root/ytrain.log"
$y = 'memento-claw-yoga-ubuntu'
$app = '/home/intpa/Applications/Eden-GIF/20261113-973abf9c2e3f-unmapped-stop-lan2c-svc-x8-x17-gdbstub-2f315fc4b7d9/Eden.AppImage'
$et = '/home/intpa/pb-imp-13b2e83/tools/eden-test/eden_test.py'
$lc = '/home/intpa/pb-imp-13b2e83/tools/eden-lifecycle/lifecycle.py'
function Note([string]$text) { "$(Get-Date -AsUTC -Format s)Z $text" | Add-Content $log }
# The pair's Yoga instance goes first; keep the gap after its release too.
if ($AfterPair) {
    while (-not (Select-String -Path "/tmp/opencode/imgui-parity/split/pair-$AfterPair.log" -Pattern 'yoga ssh exit' -Quiet)) { Start-Sleep 30 }
    Note "pair $AfterPair done; gap $GapSeconds s"
    Start-Sleep $GapSeconds
}
$n = @{ fa = 1; fc = 1 }
foreach ($v in $Order.Split(',')) {
    while ($true) {
        $label = "imp-y$v$($n[$v])"
        $exists = Invoke-Guarded 256M 60 /usr/bin/ssh -o BatchMode=yes $y "Test-Path /home/intpa/eden-runs/$label"
        if ("$exists" -ne 'True') { break }
        $n[$v]++
    }
    $deadline = (Get-Date).AddMinutes(60)
    while ((Get-Date) -lt $deadline) {
        $check = Format-RemoteGuarded 1G 120 "/usr/bin/python3 -I /home/intpa/eden-runs/imp-split/preflight_ok.py $lc /home/intpa/eden-runs/imp-ytrain/base-y$v.json"
        $pf = Invoke-Guarded 256M 180 /usr/bin/ssh -o BatchMode=yes $y $check
        if ($LASTEXITCODE -eq 0) { break }
        Note "$label waits: $pf"
        Start-Sleep 60
    }
    Note "$label start (variant $v)"
    $run = Format-RemoteGuarded 3G 4200 "/usr/bin/python3 -B -I $et all --from /home/intpa/eden-runs/imp-ytrain/base-y$v.json --label $label --rows /home/intpa/eden-runs/imp-ytrain/rows --app $app --owner 'OpenCode deluxe-imgui-parity'"
    $cmd = "$run > /home/intpa/eden-runs/imp-ytrain/run-$label.log 2>&1; echo EXIT:`$LASTEXITCODE >> /home/intpa/eden-runs/imp-ytrain/run-$label.log"
    Invoke-Guarded 256M 4500 /usr/bin/ssh -o BatchMode=yes -o ServerAliveInterval=30 $y $cmd
    $tail = Invoke-Guarded 256M 60 /usr/bin/ssh -o BatchMode=yes $y "Get-Content /home/intpa/eden-runs/$label/RESULTS.md -ErrorAction SilentlyContinue | Select-String 'Rows:|Down:'"
    Note "$label end: $tail"
    $n[$v]++
    Start-Sleep $GapSeconds
}
Note 'ytrain done'
