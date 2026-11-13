param(
    [Parameter(Mandatory)][string]$Suffix,
    [Parameter(Mandatory)][string]$LogDir,
    [string]$HostA = 'intpa@ubuntu-xps.lan',
    [string]$HostB = 'memento-claw-yoga-ubuntu',
    [string]$PlaybookA = '/home/intpa/ssbu-mod-playbook',
    [string]$PlaybookB = '/home/intpa/pb-scene-13b2e83',
    [int]$LeadSeconds = 75
)
# Release-candidate pair: run folders drc-a<Suffix> (host A) and drc-b<Suffix> (host B), made beforehand with
# `eden_test.py new` from make_config_rc.py's base. Each host: up (dry run + lifecycle claim) and run, then down
# whatever happened. A boots first (the first LAN broker leads), B LeadSeconds later. Every ssh runs under
# systemd-run MemoryMax + timeout.
$ErrorActionPreference = 'Continue'
$log = "$LogDir/rc-pair-$Suffix.log"
function Note([string]$text) { "$(Get-Date -AsUTC -Format s)Z $text" | Add-Content $log }
function Command([string]$pb, [string]$label) {
    $t = "/usr/bin/python3 -B -I $pb/tools/eden-test/eden_test.py"
    $r = "/home/intpa/eden-runs/$label"
    "($t up $r && $t run $r) > $r.log 2>&1; echo RUN_EXIT:`$? >> $r.log; $t down $r >> $r.log 2>&1; echo DOWN_EXIT:`$? >> $r.log"
}
$guard = @('--user', '--scope', '-q', '-p', 'MemoryMax=1G', '/usr/bin/timeout', '5400', '/usr/bin/ssh', '-o', 'BatchMode=yes', '-o', 'ServerAliveInterval=30')
Note "start: $HostA drc-a$Suffix, then $HostB drc-b$Suffix"
$a = Start-Process -PassThru -NoNewWindow -FilePath /usr/bin/systemd-run -ArgumentList ($guard + @($HostA, (Command $PlaybookA "drc-a$Suffix"))) -RedirectStandardOutput "$LogDir/rc-$Suffix-a.out" -RedirectStandardError "$LogDir/rc-$Suffix-a.err"
Start-Sleep $LeadSeconds
$b = Start-Process -PassThru -NoNewWindow -FilePath /usr/bin/systemd-run -ArgumentList ($guard + @($HostB, (Command $PlaybookB "drc-b$Suffix"))) -RedirectStandardOutput "$LogDir/rc-$Suffix-b.out" -RedirectStandardError "$LogDir/rc-$Suffix-b.err"
$a.WaitForExit(5500000) | Out-Null
Note "a ssh exit $($a.ExitCode)"
$b.WaitForExit(5500000) | Out-Null
Note "b ssh exit $($b.ExitCode)"
