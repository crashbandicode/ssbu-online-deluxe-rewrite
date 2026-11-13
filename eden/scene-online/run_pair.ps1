param(
    [Parameter(Mandatory)][ValidateSet('prepost', 'fix')][string]$Pair,
    [Parameter(Mandatory)][string]$Suffix,
    [Parameter(Mandatory)][string]$LogDir
)
# deluxe-scene-online split pair: instance a on the XPS, b on the Yoga (no Surface slot), each `eden_test.py all` on
# its own host through its own lifecycle claim. The XPS boots first: the first LAN broker leads the coordinator, and
# inbound connections to the Yoga timed out before (imgui-parity run_pair_xy.ps1, eden-pair e01).
# Every ssh runs under systemd-run MemoryMax + timeout.
$ErrorActionPreference = 'Continue'
$app = '/home/intpa/Applications/Eden-GIF/20261113-973abf9c2e3f-unmapped-stop-lan2c-svc-x8-x17-gdbstub-2f315fc4b7d9/Eden.AppImage'
$d = '/home/intpa/eden-runs/scene-online'
$log = "$LogDir/pair-$Pair-$Suffix.log"
function Note([string]$text) { "$(Get-Date -AsUTC -Format s)Z $text" | Add-Content $log }
$label = @{ a = "scn-$Pair-a$Suffix"; b = "scn-$Pair-b$Suffix" }
$xcmd = "/usr/bin/python3 -B -I /home/intpa/ssbu-mod-playbook/tools/eden-test/eden_test.py all --from $d/base-$Pair-a.json --label $($label.a) --rows $d/rows/rows-$Pair-a --app $app --owner 'OpenCode deluxe-scene-online' > $d/run-$($label.a).log 2>&1; echo EXIT:`$? >> $d/run-$($label.a).log"
$ycmd = "/usr/bin/python3 -B -I /home/intpa/pb-scene-13b2e83/tools/eden-test/eden_test.py all --from $d/base-$Pair-b.json --label $($label.b) --rows $d/rows/rows-$Pair-b --app $app --owner 'OpenCode deluxe-scene-online' > $d/run-$($label.b).log 2>&1; echo EXIT:`$LASTEXITCODE >> $d/run-$($label.b).log"
Note "start ${Pair}: xps $($label.a), then yoga $($label.b)"
$guard = @('--user', '--scope', '-q', '-p', 'MemoryMax=1G', '/usr/bin/timeout', '4800', '/usr/bin/ssh', '-o', 'BatchMode=yes', '-o', 'ServerAliveInterval=30')
$xps = Start-Process -PassThru -NoNewWindow -FilePath /usr/bin/systemd-run -ArgumentList ($guard + @('intpa@ubuntu-xps.lan', $xcmd)) -RedirectStandardOutput "$LogDir/pair-$Pair-$Suffix-xps.out" -RedirectStandardError "$LogDir/pair-$Pair-$Suffix-xps.err"
Start-Sleep 75
$yoga = Start-Process -PassThru -NoNewWindow -FilePath /usr/bin/systemd-run -ArgumentList ($guard + @('memento-claw-yoga-ubuntu', $ycmd)) -RedirectStandardOutput "$LogDir/pair-$Pair-$Suffix-yoga.out" -RedirectStandardError "$LogDir/pair-$Pair-$Suffix-yoga.err"
$xps.WaitForExit(4900000) | Out-Null
Note "xps ssh exit $($xps.ExitCode)"
$yoga.WaitForExit(4900000) | Out-Null
Note "yoga ssh exit $($yoga.ExitCode)"
