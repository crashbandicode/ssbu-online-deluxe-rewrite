param([Parameter(Mandatory)][string]$Variants, [int]$Attempts = 2, [int]$GapSeconds = 180)
# One Surface slot only (primary's order, 13:52Z): lane 1 (display :141, gdb 6571, labels imp-*), one boot at a time,
# and a gap after every release before the next claim so a waiting lane can get in.
$ErrorActionPreference = 'Continue'
$root = '/tmp/opencode/imgui-parity'
$qlog = "$root/queue-l1.log"
function Next-Label([string]$v) {
    $n = 1
    while (Test-Path "/home/intpa/eden-runs/imp-$v$n") { $n++ }
    return "imp-$v$n"
}
function Wait-OwnBootsDone {
    while (Get-Process | Where-Object { $_.CommandLine -match 'eden_test\.py all --from /tmp/opencode/imgui-parity/' }) {
        Start-Sleep 30
    }
}
foreach ($v in $Variants.Split(',')) {
    for ($try = 1; $try -le $Attempts; $try++) {
        Wait-OwnBootsDone
        "$(Get-Date -AsUTC -Format s)Z gap $GapSeconds s before the next claim" | Add-Content $qlog
        Start-Sleep $GapSeconds
        Wait-OwnBootsDone
        $label = Next-Label $v
        "$(Get-Date -AsUTC -Format s)Z start $label (variant $v, lane 1, attempt $try)" | Add-Content $qlog
        & /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File "$root/run_variant.ps1" -Variant $v -Label $label -Lane 1
        $receipt = "/home/intpa/eden-runs/$label/out/hang-after.json"
        $ok = (Test-Path $receipt) -and ((Get-Content $receipt -Raw | ConvertFrom-Json).status -eq 'PASS')
        "$(Get-Date -AsUTC -Format s)Z end $label training=$ok" | Add-Content $qlog
        if ($ok) { break }
    }
}
"$(Get-Date -AsUTC -Format s)Z queue done" | Add-Content $qlog
