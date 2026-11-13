param([Parameter(Mandatory)][string]$Suffix, [int]$Minutes = 75)
# Wait until both the XPS (instance a) and the Yoga (instance b) pass a read-only lifecycle preflight, then start the
# pair. Polls every 60 s.
$ErrorActionPreference = 'Continue'
$root = '/tmp/opencode/imgui-parity/split'
$log = "$root/pair-$Suffix.log"
function Note([string]$text) { "$(Get-Date -AsUTC -Format s)Z $text" | Add-Content $log }
& scp -q -o BatchMode=yes "$root/preflight_ok.py" 'intpa@ubuntu-xps.lan:/home/intpa/eden-runs/imp-split/preflight_ok.py'
& scp -q -o BatchMode=yes "$root/preflight_ok.py" 'memento-claw-yoga-ubuntu:/home/intpa/eden-runs/imp-split/preflight_ok.py'
$deadline = (Get-Date).AddMinutes($Minutes)
while ((Get-Date) -lt $deadline) {
    $x = & ssh -o BatchMode=yes intpa@ubuntu-xps.lan '/usr/bin/python3 -I /home/intpa/eden-runs/imp-split/preflight_ok.py /home/intpa/ssbu-mod-playbook/tools/eden-lifecycle/lifecycle.py /home/intpa/eden-runs/imp-split/base-a-xps.json'
    $xok = $LASTEXITCODE -eq 0
    $y = & ssh -o BatchMode=yes memento-claw-yoga-ubuntu '/usr/bin/python3 -I /home/intpa/eden-runs/imp-split/preflight_ok.py /home/intpa/pb-imp-13b2e83/tools/eden-lifecycle/lifecycle.py /home/intpa/eden-runs/imp-split/base-b.json'
    $yok = $LASTEXITCODE -eq 0
    Note "xps: $x"
    Note "yoga: $y"
    if ($xok -and $yok) {
        Note 'both hosts admit; starting the pair'
        & /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File "$root/run_pair_xy.ps1" -Suffix $Suffix
        exit 0
    }
    Start-Sleep 60
}
Note 'gave up waiting for both hosts'
exit 1
