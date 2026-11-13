param([Parameter(Mandatory)][string]$Suffix)
# imgui-parity split pair on the XPS (instance a) and the Yoga (instance b), started together; no Surface slot. Each
# ssh runs under the playbook's memory/time guard here (../guard.ps1), and each eden_test.py run under the same guard
# on its host.
$ErrorActionPreference = 'Continue'
. "$PSScriptRoot/../guard.ps1"
$root = '/tmp/opencode/imgui-parity/split'
$app = '/home/intpa/Applications/Eden-GIF/20261113-973abf9c2e3f-unmapped-stop-lan2c-svc-x8-x17-gdbstub-2f315fc4b7d9/Eden.AppImage'
$log = "$root/pair-$Suffix.log"
function Note([string]$text) { "$(Get-Date -AsUTC -Format s)Z $text" | Add-Content $log }
$xrun = Format-RemoteGuarded 3G 4200 "/usr/bin/python3 -B -I /home/intpa/ssbu-mod-playbook/tools/eden-test/eden_test.py all --from /home/intpa/eden-runs/imp-split/base-a-xps.json --label imp-oppa$Suffix --rows /home/intpa/eden-runs/imp-split/rows-a --app $app --owner 'OpenCode deluxe-imgui-parity'"
$xcmd = "cd /home/intpa && $xrun > /home/intpa/eden-runs/imp-split/run-oppa$Suffix.log 2>&1; echo EXIT:`$? >> /home/intpa/eden-runs/imp-split/run-oppa$Suffix.log"
$yrun = Format-RemoteGuarded 3G 4200 "/usr/bin/python3 -B -I /home/intpa/pb-imp-13b2e83/tools/eden-test/eden_test.py all --from /home/intpa/eden-runs/imp-split/base-b.json --label imp-oppb$Suffix --rows /home/intpa/eden-runs/imp-split/rows-b --app $app --owner 'OpenCode deluxe-imgui-parity'"
$ycmd = "$yrun > /home/intpa/eden-runs/imp-split/run-oppb$Suffix.log 2>&1; echo EXIT:`$LASTEXITCODE >> /home/intpa/eden-runs/imp-split/run-oppb$Suffix.log"
$guard = @('--user', '--scope', '--quiet', '-p', 'MemoryMax=256M', '/usr/bin/timeout', '4500', '/usr/bin/ssh', '-o', 'BatchMode=yes', '-o', 'ServerAliveInterval=30')
Note 'start xps a + yoga b'
$xps = Start-Process -PassThru -NoNewWindow -FilePath /usr/bin/systemd-run -ArgumentList ($guard + @('intpa@ubuntu-xps.lan', $xcmd)) -RedirectStandardOutput "$root/pair-$Suffix-xps.ssh.out" -RedirectStandardError "$root/pair-$Suffix-xps.ssh.err"
# The first broker leads the LAN coordinator and inbound connections to the Yoga time out (eden-pair/pair.ps1), so
# the XPS boots first; in imp-oppa8/b8 the Yoga's broker started 43 s before the XPS's.
Start-Sleep 75
$yoga = Start-Process -PassThru -NoNewWindow -FilePath /usr/bin/systemd-run -ArgumentList ($guard + @('memento-claw-yoga-ubuntu', $ycmd)) -RedirectStandardOutput "$root/pair-$Suffix-yoga.ssh.out" -RedirectStandardError "$root/pair-$Suffix-yoga.ssh.err"
$xps.WaitForExit(4600000) | Out-Null
Note "xps ssh exit $($xps.ExitCode)"
$yoga.WaitForExit(4600000) | Out-Null
Note "yoga ssh exit $($yoga.ExitCode)"
