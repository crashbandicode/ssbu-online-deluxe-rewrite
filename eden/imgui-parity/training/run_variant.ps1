param([Parameter(Mandatory)][string]$Variant, [Parameter(Mandatory)][string]$Label, [string]$Rows, [int]$Lane = 1)
$ErrorActionPreference = 'Stop'
$root = '/tmp/opencode/imgui-parity'
$suffix = if ($Lane -eq 2) { '-l2' } else { '' }
if (-not $Rows) { $Rows = if ($Variant -in @('a', 'fa')) { "$root/stage-a$suffix/rows-hang" } else { "$root/stage-b$suffix/rows-hang" } }
$log = "$root/run-$Label.log"
"start $(Get-Date -AsUTC -Format s)Z variant=$Variant label=$Label lane=$Lane rows=$Rows" | Out-File -Append $log
& systemd-run --user --scope --quiet -p MemoryMax=3G /usr/bin/timeout 4200 /usr/bin/python3 -B -I /home/intpa/ssbu-mod-playbook/tools/eden-test/eden_test.py all --from "$root/base-$Variant$suffix.json" --label $Label --rows $Rows --owner 'OpenCode deluxe-imgui-parity' *>> $log
"exit $LASTEXITCODE $(Get-Date -AsUTC -Format s)Z" | Out-File -Append $log
