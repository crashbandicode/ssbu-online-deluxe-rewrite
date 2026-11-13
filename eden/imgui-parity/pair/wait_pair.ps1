param([Parameter(Mandatory)][string]$Suffix, [int]$Minutes = 75)
# Wait until both the XPS (instance a) and the Yoga (instance b) pass a read-only lifecycle preflight, then start the
# pair. Polls every 60 s. Every scp/ssh runs under the playbook's memory/time guard (../guard.ps1), the remote
# preflight under the same guard on its host.
$ErrorActionPreference = 'Continue'
. "$PSScriptRoot/../guard.ps1"
$root = '/tmp/opencode/imgui-parity/split'
$log = "$root/pair-$Suffix.log"
function Note([string]$text) { "$(Get-Date -AsUTC -Format s)Z $text" | Add-Content $log }
Invoke-Guarded 256M 120 /usr/bin/scp -q -o BatchMode=yes "$PSScriptRoot/preflight_ok.py" 'intpa@ubuntu-xps.lan:/home/intpa/eden-runs/imp-split/preflight_ok.py'
Invoke-Guarded 256M 120 /usr/bin/scp -q -o BatchMode=yes "$PSScriptRoot/preflight_ok.py" 'memento-claw-yoga-ubuntu:/home/intpa/eden-runs/imp-split/preflight_ok.py'
$deadline = (Get-Date).AddMinutes($Minutes)
while ((Get-Date) -lt $deadline) {
    $x = Invoke-Guarded 256M 180 /usr/bin/ssh -o BatchMode=yes intpa@ubuntu-xps.lan (Format-RemoteGuarded 1G 120 '/usr/bin/python3 -I /home/intpa/eden-runs/imp-split/preflight_ok.py /home/intpa/ssbu-mod-playbook/tools/eden-lifecycle/lifecycle.py /home/intpa/eden-runs/imp-split/base-a-xps.json')
    $xok = $LASTEXITCODE -eq 0
    $y = Invoke-Guarded 256M 180 /usr/bin/ssh -o BatchMode=yes memento-claw-yoga-ubuntu (Format-RemoteGuarded 1G 120 '/usr/bin/python3 -I /home/intpa/eden-runs/imp-split/preflight_ok.py /home/intpa/pb-imp-13b2e83/tools/eden-lifecycle/lifecycle.py /home/intpa/eden-runs/imp-split/base-b.json')
    $yok = $LASTEXITCODE -eq 0
    Note "xps: $x"
    Note "yoga: $y"
    if ($xok -and $yok) {
        Note 'both hosts admit; starting the pair'
        Invoke-Guarded 1G 4800 /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File "$PSScriptRoot/run_pair_xy.ps1" -Suffix $Suffix
        exit 0
    }
    Start-Sleep 60
}
Note 'gave up waiting for both hosts'
exit 1
