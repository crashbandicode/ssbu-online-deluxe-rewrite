param([string]$Suffix = '9')
# Copies the split pair's receipts, row screenshots and Deluxe/detector files into the artifacts folder.
$dest = '/home/intpa/smash-training-lobby/artifacts/imgui-parity/eden'
$sets = @(
    @{ host = 'intpa@ubuntu-xps.lan'; label = "imp-oppa$Suffix"; key = 'a' },
    @{ host = 'memento-claw-yoga-ubuntu'; label = "imp-oppb$Suffix"; key = 'b' }
)
$keep = @(
    'user/sdmc/ultimate/ssbu_online_deluxe_rewrite/renderer-status.txt',
    'user/sdmc/ultimate/ssbu_online_deluxe_rewrite/install.log',
    'user/sdmc/ultimate/ssbu_online_deluxe_rewrite/hang-trace.log',
    'user/sdmc/ultimate/ssbu_online_deluxe_rewrite/events.log',
    'user/sdmc/ultimate/ssbu_online_deluxe_rewrite/matches/index.txt',
    'user/sdmc/ultimate/ssbu-desync-detector/events.log'
)
foreach ($s in $sets) {
    $run = "/home/intpa/eden-runs/$($s.label)"
    $to = "$dest/$($s.label)"
    New-Item -ItemType Directory -Force "$to/out", "$to/post-run" | Out-Null
    & scp -q -o BatchMode=yes "$($s.host):$run/RESULTS.md" "$to/RESULTS.md"
    & scp -q -o BatchMode=yes "$($s.host):$run/out/*.json" "$to/out/"
    foreach ($row in 'm-label', 'm-play', 'f1-fonts', 'n-nav', 'z', 'm-css') {
        & scp -q -r -o BatchMode=yes "$($s.host):$run/out/$row-$($s.key)" "$to/out/"
    }
    foreach ($f in $keep) {
        $name = ($f -split '/')[-2] + '-' + ($f -split '/')[-1]
        & scp -q -o BatchMode=yes "$($s.host):$run/out/post-run-$($s.key)/$f" "$to/post-run/$name"
    }
    "$($s.label): $((Get-ChildItem -Recurse -File $to | Measure-Object Length -Sum).Sum) bytes, $((Get-ChildItem -Recurse -File $to).Count) files"
}
