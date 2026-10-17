<#
.SYNOPSIS
    Native graphical smoke test and performance baseline for Warcraft Survivors, on a WoW 1.12.1
    Data folder you own.

.DESCRIPTION
    Three legs. Each is a real run of the game driven by the built-in autopilot
    (WOW_SURVIVORS_AUTO), so no input is sent and no window needs focus:

      menu     Mage on The Barrens for 60 s: the window opens, the menu is captured, a run begins,
               the hero fights, and the first level-up is reached.
      death    Warrior:idle, which stands still, until the hero dies: the death, the results screen,
               and the return to the menu followed by a second run.
      victory  Mage:late, which starts two thirds of the way through the run with the full ability
               pool at top rank, until the run is won.
      perf     Mage:late for 120 s in the normal window, with the game's frame journal on: frame,
               main-thread, GPU and entity figures for the largest crowd.

    Writes screenshots, the game's logs and report.md under the output folder. The exit code is 0
    only when every automated check passes. Items listed under "Manual" need a person to look at
    the screenshots; their paths are in the report.

    Nothing here downloads anything, and the data folder is only read.

.PARAMETER DataDir
    A WoW 1.12.1 (build 5875) Data folder containing terrain.MPQ and patch.MPQ.

.PARAMETER Release
    Use the release build (the performance baseline should use this).

.PARAMETER OutDir
    Where to write the run. Defaults to target\smoke\<timestamp> in the repository.

.PARAMETER SkipVictory
    Skip the victory leg (about five minutes of game time).

.EXAMPLE
    .\scripts\windows\smoke-survivors.ps1 -DataDir "D:\WoW 1.12.1\Data" -Release
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$DataDir,
    [switch]$Release,
    [string]$OutDir,
    [switch]$SkipVictory,
    [int]$DeathTimeoutSec = 420,
    [int]$VictoryTimeoutSec = 600
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Resolve-Path (Join-Path (Join-Path $PSScriptRoot '..') '..')
Set-Location $repo

function Resolve-Cargo {
    if (Get-Command cargo -ErrorAction SilentlyContinue) { return }
    $bin = Join-Path (Join-Path $env:USERPROFILE '.cargo') 'bin'
    if (Test-Path (Join-Path $bin 'cargo.exe')) { $env:PATH = "$bin;$env:PATH"; return }
    throw 'cargo was not found. Install Rust with: winget install --id Rustlang.Rustup -e'
}

if (-not (Test-Path $DataDir -PathType Container)) { throw "-DataDir '$DataDir' is not a folder." }
$dataFull = (Resolve-Path $DataDir).Path
if (-not ((Test-Path (Join-Path $dataFull 'terrain.MPQ')) -and (Test-Path (Join-Path $dataFull 'patch.MPQ')))) {
    throw "-DataDir '$dataFull' does not hold terrain.MPQ and patch.MPQ."
}

Resolve-Cargo
$profileDir = if ($Release) { 'release' } else { 'debug' }
$buildArgs = @('build', '-p', 'warcraft-survivors')
if ($Release) { $buildArgs += '--release' }
& cargo @buildArgs
if ($LASTEXITCODE -ne 0) { throw 'the build failed' }
$exe = Join-Path (Join-Path (Join-Path $repo 'target') $profileDir) 'warcraft-survivors.exe'

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
if (-not $OutDir) { $OutDir = Join-Path (Join-Path (Join-Path $repo 'target') 'smoke') $stamp }
New-Item -ItemType Directory -Force $OutDir | Out-Null
$OutDir = (Resolve-Path $OutDir).Path

$statusPattern = 'survivors auto: (\w+) t=(\d+)s fps=(\d+) alive=(\d+) level=(\d+) hp=(-?\d+) kills=(\d+) damage=(\d+)'

function Read-Log([string]$path) {
    if (-not (Test-Path $path)) { return '' }
    try {
        $fs = [System.IO.File]::Open($path, 'Open', 'Read', 'ReadWrite')
        try { $sr = New-Object System.IO.StreamReader($fs); return $sr.ReadToEnd() } finally { $fs.Dispose() }
    } catch { return '' }
}

function Invoke-Leg {
    param(
        [string]$Name,
        [string]$Auto,
        [int]$TimeoutSec,
        [scriptblock]$Done,
        [int]$GraceSec = 0,
        [hashtable]$Env = @{}
    )
    $dir = Join-Path $OutDir $Name
    New-Item -ItemType Directory -Force $dir | Out-Null
    $log = Join-Path $dir 'stderr.log'
    $out = Join-Path $dir 'stdout.log'

    $env:WOW_DATA = $dataFull
    $env:WOW_SURVIVORS_AUTO = $Auto
    $env:WOW_SURVIVORS_MAP = '0'
    $env:WOW_SURVIVORS_SHOTS = $dir
    foreach ($key in $Env.Keys) { [System.Environment]::SetEnvironmentVariable($key, [string]$Env[$key], 'Process') }
    $proc = Start-Process -FilePath $exe -PassThru -RedirectStandardError $log -RedirectStandardOutput $out
    Remove-Item Env:WOW_SURVIVORS_AUTO, Env:WOW_SURVIVORS_MAP, Env:WOW_SURVIVORS_SHOTS -ErrorAction SilentlyContinue
    Remove-Item Env:WOW_DATA -ErrorAction SilentlyContinue
    foreach ($key in $Env.Keys) { [System.Environment]::SetEnvironmentVariable($key, $null, 'Process') }

    $started = [System.Diagnostics.Stopwatch]::StartNew()
    $peakWs = [int64]0
    $unresponsive = 0
    $reason = 'timeout'
    while ($started.Elapsed.TotalSeconds -lt $TimeoutSec) {
        Start-Sleep -Seconds 2
        $proc.Refresh()
        if ($proc.HasExited) { $reason = "exited ($($proc.ExitCode))"; break }
        if ($proc.WorkingSet64 -gt $peakWs) { $peakWs = $proc.WorkingSet64 }
        if (-not $proc.Responding) { $unresponsive++ }
        if ($Done -and (& $Done (Read-Log $log))) { $reason = 'done'; break }
    }
    if ($reason -eq 'done' -and $GraceSec -gt 0) {
        # The results screen comes a moment after the outcome line: let its screenshot save.
        Start-Sleep -Seconds $GraceSec
    }
    $wall = [Math]::Round($started.Elapsed.TotalSeconds, 1)
    if (-not $proc.HasExited) {
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }
    $text = Read-Log $log
    [pscustomobject]@{
        Name = $Name
        Dir = $dir
        Log = $text
        Reason = $reason
        WallSec = $wall
        PeakWorkingSetMB = [Math]::Round($peakWs / 1MB, 0)
        UnresponsiveSamples = $unresponsive
    }
}


function Get-Status($leg) {
    [regex]::Matches($leg.Log, $statusPattern) | ForEach-Object {
        [pscustomobject]@{
            Phase = $_.Groups[1].Value
            T = [int]$_.Groups[2].Value
            Fps = [int]$_.Groups[3].Value
            Alive = [int]$_.Groups[4].Value
            Level = [int]$_.Groups[5].Value
            Hp = [int]$_.Groups[6].Value
            Kills = [int]$_.Groups[7].Value
            Damage = [int64]$_.Groups[8].Value
        }
    }
}

$checks = New-Object System.Collections.Generic.List[object]
function Add-Check([string]$Area, [string]$Name, [bool]$Pass, [string]$Evidence) {
    $result = 'FAIL'
    if ($Pass) { $result = 'PASS' }
    $checks.Add([pscustomobject]@{ Area = $Area; Check = $Name; Result = $result; Evidence = $Evidence })
}

function Count-Matches([string]$text, [string]$pattern) {
    ([regex]::Matches($text, $pattern)).Count
}

# ── Menu, run start, combat, first level-up ──────────────────────────────────────────────────────
Write-Host '== leg: menu (Mage, 60 s)'
$menu = Invoke-Leg -Name 'menu' -Auto 'Mage' -TimeoutSec 60 -Done $null
$menuStatus = @(Get-Status $menu)
Add-Check 'Launch' 'Window opens' ($menu.Log -match 'Creating new window Warcraft Survivors') 'bevy_winit: Creating new window Warcraft Survivors'
Add-Check 'Launch' 'Process stays responsive' ($menu.UnresponsiveSamples -eq 0) "unresponsive samples: $($menu.UnresponsiveSamples)"
Add-Check 'Menu' 'Menu screenshot captured' (Test-Path (Join-Path $menu.Dir 'menu.png')) (Join-Path $menu.Dir 'menu.png')
Add-Check 'Run' 'Run begins as Mage on The Barrens' ($menu.Log -match 'run begins as Mage at The Barrens') 'survivors auto: run begins as Mage at The Barrens'
Add-Check 'Combat' 'Hero deals damage and enemies spawn' (@($menuStatus | Where-Object { $_.Damage -gt 0 -and $_.Alive -gt 0 }).Count -gt 0) "status lines with damage>0: $(@($menuStatus | Where-Object { $_.Damage -gt 0 }).Count)"
Add-Check 'Levelling' 'Level-up reached, with its card screenshot' ((@($menuStatus | Where-Object { $_.Level -ge 2 }).Count -gt 0) -and (Test-Path (Join-Path $menu.Dir 'levelup-2.png'))) "max level: $(($menuStatus | Measure-Object Level -Maximum).Maximum)"
Add-Check 'Stability' 'No ERROR lines in the menu leg' ((Count-Matches $menu.Log ' ERROR ') -eq 0) "ERROR lines: $(Count-Matches $menu.Log ' ERROR ')"

# ── Death, results, return to the menu ───────────────────────────────────────────────────────────
Write-Host '== leg: death (Warrior:idle)'
$deathDone = {
    param($text)
    ($text -match 'run over: died') -and ((Count-Matches $text 'run begins as') -ge 2)
}
$death = Invoke-Leg -Name 'death' -Auto 'Warrior:idle' -TimeoutSec $DeathTimeoutSec -Done $deathDone
$deathStatus = @(Get-Status $death)
$diedLine = [regex]::Match($death.Log, 'run over: died at t=(\d+)s, level (\d+), kills (\d+)')
Add-Check 'Death' 'Hero dies: run over, died' $diedLine.Success $(if ($diedLine.Success) { $diedLine.Value } else { "no death line within $DeathTimeoutSec s ($($death.Reason))" })
Add-Check 'Death' 'Health reaches zero in the status lines' (@($deathStatus | Where-Object { $_.Hp -le 0 }).Count -gt 0) "status lines at hp<=0: $(@($deathStatus | Where-Object { $_.Hp -le 0 }).Count)"
Add-Check 'Death' 'Results screenshot captured' (Test-Path (Join-Path $death.Dir 'over.png')) (Join-Path $death.Dir 'over.png')
Add-Check 'Death' 'Returns to the menu and starts a new run' ((Count-Matches $death.Log 'run begins as') -ge 2) "run begins lines: $(Count-Matches $death.Log 'run begins as')"
Add-Check 'Stability' 'No ERROR lines in the death leg' ((Count-Matches $death.Log ' ERROR ') -eq 0) "ERROR lines: $(Count-Matches $death.Log ' ERROR ')"

# ── Victory, bosses, results ─────────────────────────────────────────────────────────────────────
$victory = $null
$victoryStatus = @()
if (-not $SkipVictory) {
    Write-Host '== leg: victory (Mage:late)'
    $victoryDone = {
        param($text)
        $text -match 'run over: won'
    }
    $victory = Invoke-Leg -Name 'victory' -Auto 'Mage:late' -TimeoutSec $VictoryTimeoutSec -Done $victoryDone -GraceSec 8
    $victoryStatus = @(Get-Status $victory)
    $wonLine = [regex]::Match($victory.Log, 'run over: won at t=(\d+)s, level (\d+), kills (\d+)')
    Add-Check 'Victory' 'Run is won' $wonLine.Success $(if ($wonLine.Success) { $wonLine.Value } else { "no victory line within $VictoryTimeoutSec s ($($victory.Reason))" })
    $bossLines = Count-Matches $victory.Log 'survivors: boss .* spawned'
    Add-Check 'Bosses' 'Bosses appear on their clock (late start: 3 remain)' ($bossLines -ge 2) "boss spawn lines: $bossLines"
    Add-Check 'Victory' 'Results screenshot captured' (Test-Path (Join-Path $victory.Dir 'over.png')) (Join-Path $victory.Dir 'over.png')
    Add-Check 'Stability' 'No ERROR lines in the victory leg' ((Count-Matches $victory.Log ' ERROR ') -eq 0) "ERROR lines: $(Count-Matches $victory.Log ' ERROR ')"
}

# ── Performance baseline: uncapped, the crowd at its largest ────────────────────────────────────
Write-Host '== leg: perf (Mage:late, vsync off, 120 s)'
$perfDir = Join-Path $OutDir 'perf'
$journal = Join-Path $perfDir 'journal.csv'
$perf = Invoke-Leg -Name 'perf' -Auto 'Mage:late' -TimeoutSec 150 -Done $null -Env @{ WOW_BG = '0'; WOW_NOVSYNC = '1'; WOW_FPS_JOURNAL = $journal }
$perfStatus = @(Get-Status $perf)
$journalRows = @()
if (Test-Path $journal) {
    $csvLines = @(Get-Content $journal | Where-Object { $_ -and -not $_.StartsWith('#') })
    if ($csvLines.Count -gt 1) { $journalRows = @($csvLines | ConvertFrom-Csv) }
}
Add-Check 'Performance' 'Frame journal recorded the perf run' ($journalRows.Count -ge 30) "journal rows: $($journalRows.Count)"

# ── Performance baseline ─────────────────────────────────────────────────────────────────────────
# The first rows are start-up (loading, pipeline warm-up); the steady state starts at row 6.
$steady = @($journalRows | Select-Object -Skip 5)
$allStatus = @($menuStatus) + @($deathStatus) + @($victoryStatus)
$fpsMeanOf = {
    param($rows)
    $values = @($rows | ForEach-Object { $_.Fps } | Where-Object { $_ -gt 0 })
    if ($values.Count) { [Math]::Round(($values | Measure-Object -Average).Average, 1) } else { 0 }
}
$fpsVsyncOn = & $fpsMeanOf $allStatus
$fpsVsyncOff = & $fpsMeanOf $perfStatus
$peakAlive = if ($allStatus.Count) { ($allStatus | Measure-Object Alive -Maximum).Maximum } else { 0 }
$frameMean = 0; $frameP95 = 0; $mainMean = 0; $gpuMean = 0; $entitiesMax = 0; $cpuMean = 0
if ($steady.Count) {
    $frameMean = [Math]::Round((@($steady | ForEach-Object { [double]$_.mean_ms }) | Measure-Object -Average).Average, 2)
    $frameP95 = [Math]::Round((@($steady | ForEach-Object { [double]$_.p95_ms }) | Measure-Object -Maximum).Maximum, 2)
    $mainMean = [Math]::Round((@($steady | ForEach-Object { [double]$_.main_ms }) | Measure-Object -Average).Average, 2)
    $gpuMean = [Math]::Round((@($steady | ForEach-Object { [double]$_.gpu_ms }) | Measure-Object -Average).Average, 2)
    $cpuMean = [Math]::Round((@($steady | ForEach-Object { [double]$_.cpu_ms }) | Measure-Object -Average).Average, 2)
    $entitiesMax = (@($steady | ForEach-Object { [int]$_.entities }) | Measure-Object -Maximum).Maximum
}
$liveLabels = @(([regex]::Matches(($menu.Log + $death.Log + $(if ($victory) { $victory.Log } else { '' }) + $perf.Log), 'compiled LIVE \[\d+\] label=(\S+)') | ForEach-Object { $_.Groups[1].Value }) | Group-Object | ForEach-Object { "$($_.Name) x$($_.Count)" })
$liveTotal = @($liveLabels).Count
$peakWs = @($menu.PeakWorkingSetMB, $death.PeakWorkingSetMB, $perf.PeakWorkingSetMB, $(if ($victory) { $victory.PeakWorkingSetMB } else { 0 })) | Measure-Object -Maximum
$gpu = (@(Get-CimInstance Win32_VideoController) | ForEach-Object { $_.Name }) -join '; '
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name
$ram = [Math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 0)
$sha = (git rev-parse --short HEAD 2>$null)
$branch = (git branch --show-current 2>$null)
$profile = if ($Release) { 'release' } else { 'dev' }

# ── Report ───────────────────────────────────────────────────────────────────────────────────────
$passed = @($checks | Where-Object { $_.Result -eq 'PASS' }).Count
$failed = @($checks | Where-Object { $_.Result -eq 'FAIL' }).Count
$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("# Survivors smoke test: $stamp")
$lines.Add('')
$lines.Add("Build: $profile, commit $sha on $branch. Data: the folder given to the script (not recorded here).")
$lines.Add("Machine: $cpu; $ram GB RAM; GPU $gpu.")
$lines.Add('')
$lines.Add("## Automated checks: $passed passed, $failed failed")
$lines.Add('')
$lines.Add('| Area | Check | Result | Evidence |')
$lines.Add('|---|---|---|---|')
foreach ($c in $checks) { $lines.Add("| $($c.Area) | $($c.Check) | $($c.Result) | $($c.Evidence) |") }
$lines.Add('')
$lines.Add('## Performance baseline')
$lines.Add('')
$lines.Add('Steady state of the perf leg (Mage:late, the largest crowd), from the game''s frame journal.')
$lines.Add('The perf leg runs with vsync off (WOW_NOVSYNC=1), so its frame time is the game''s own pace.')
$lines.Add('The vsync-on legs (menu, death, victory) are limited by the display, not by the game, so their')
$lines.Add('fps is shown for comparison only. The adapter that drives the game window is not recorded.')
$lines.Add('')
$lines.Add('| Measure | Value | Source |')
$lines.Add('|---|---|---|')
$lines.Add("| Frame time, mean | $frameMean ms | perf leg, journal mean_ms |")
$lines.Add("| Frame time, worst p95 | $frameP95 ms | perf leg, journal p95_ms |")
$lines.Add("| Main-thread work per frame, mean | $mainMean ms | perf leg, journal main_ms |")
$lines.Add("| GPU work per frame, mean | $gpuMean ms | perf leg, journal gpu_ms |")
$lines.Add("| CPU time per interval, mean | $cpuMean ms | perf leg, journal cpu_ms |")
$lines.Add("| Peak entities | $entitiesMax | perf leg, journal entities |")
$lines.Add("| Mean FPS, vsync off | $fpsVsyncOff | perf leg, status lines (every 10 s) |")
$lines.Add("| Mean FPS, vsync on | $fpsVsyncOn | menu, death and victory legs, status lines |")
$lines.Add("| Peak live enemies | $peakAlive | status lines |")
$lines.Add("| Peak working set | $($peakWs.Maximum) MB | process sampled every 2 s |")
$lines.Add("| Pipelines compiled live during play | $liveTotal kinds | $(if ($liveTotal) { ($liveLabels -join '; ') } else { 'none' }) |")
$lines.Add('')

$lines.Add('## Manual checks (look at the screenshots)')
$lines.Add('')
$lines.Add("- Menu art and pick screen: $(Join-Path $menu.Dir 'menu.png')")
$lines.Add("- Level-up cards, rarity colours and text: $(Join-Path $menu.Dir 'levelup-2.png')")
$lines.Add("- Combat, spell visuals and enemy movement: $(Join-Path $menu.Dir 'shot-002.png')")
$lines.Add("- Death results screen: $(Join-Path $death.Dir 'over.png')")
if ($victory) { $lines.Add("- Victory results screen: $(Join-Path $victory.Dir 'over.png')") }
$lines.Add('')
$lines.Add('## Logs')
$lines.Add('')
$lines.Add("- $(Join-Path $menu.Dir 'stderr.log')")
$lines.Add("- $(Join-Path $death.Dir 'stderr.log')")
if ($victory) { $lines.Add("- $(Join-Path $victory.Dir 'stderr.log')") }
$lines.Add("- $(Join-Path $perf.Dir 'stderr.log')")
$lines.Add("- $journal (frame journal, perf leg)")

$report = Join-Path $OutDir 'report.md'
Set-Content -Path $report -Value ($lines -join "`r`n") -Encoding UTF8
Write-Host ''
Write-Host "Report: $report" -ForegroundColor Cyan
Write-Host "Automated checks: $passed passed, $failed failed"
if ($failed -gt 0) { exit 1 }
exit 0
