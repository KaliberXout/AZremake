# Warcraft Survivors: smoke-test checklist

This checklist covers the native Windows build (`warcraft-survivors.exe`). It separates checks the
automated autopilot can make from checks that need a person looking at the screen.

**Historical status (2026-10-17): automated smoke checks passed with data; manual visual checks remain open.** The launch with the
user's WoW 1.12.1 Data folder ran the autopilot legs and passed 17 of 17 automated checks
(run `20261017-165007`, release build). Steps 1 to 12 are covered below. Visual checks that a
person must confirm are listed in section 4. The results and performance figures are in
[testing.md](testing.md).

## 1. Prerequisites

- Windows 11 x64, a desktop session (the window must be on screen), and the toolchain from
  [windows-development.md](windows-development.md).
- A WoW 1.12.1 (build 5875) `Data` folder that you own and that contains `terrain.MPQ` and
  `patch.MPQ`. Point the game at it with `-DataDir` or `$env:WOW_DATA`. Nothing downloads it.
- Automated runs use the autopilot, which needs no input focus.

## 2. Automated checks

The autopilot is set with environment variables. It starts a run by itself, picks a random card
at every level-up, and returns to the menu after each game over. It logs one line every ten
seconds during a run, and a line at the start of each run.

```powershell
# Pick the class, zone (index or name fragment), and screenshot folder for this run.
$env:WOW_SURVIVORS_AUTO = 'Mage'          # or 'Warrior:idle', 'Mage:late' (see below)
$env:WOW_SURVIVORS_MAP  = '0'             # index into the zone list, or a name fragment
$env:WOW_SURVIVORS_SHOTS = "$PWD\smoke-shots"
.\scripts\windows\survivors.ps1 -Action Run -DataDir 'D:\WoW-1.12.1\Data'
```

Modes: `<class>` plays normally; `<class>:idle` stands still, so the hero dies and the run reaches
the results screen quickly; `<class>:late` starts at two thirds of the way through the run with
the full ability pool at its top level, so the run reaches victory in about five minutes.

Log lines to match (from `survivors/auto.rs` and the outcome handler in `survivors/hero.rs`):

| Line | Meaning |
| --- | --- |
| `survivors auto: run begins as <class> at <zone>` | A run started. |
| `survivors auto: Playing t=<s>s fps=<n> alive=<n> level=<n> hp=<n> kills=<n> damage=<n> build=[...]` | Periodic status during play. |
| `survivors auto: LevelUp ...` / `phase` in the status line | Level-up screen reached. |
| `survivors: boss <name> spawned at t=<s>s` | A boss appeared. |
| `survivors: run over: died at t=<s>s, level <n>, kills <n>` | Defeat. |
| `survivors: run over: won at t=<s>s, level <n>, kills <n>` | Victory. |

Screenshots (saved every six seconds, and once for the menu, first level-ups and the results
screen) go to `WOW_SURVIVORS_SHOTS`.

For the repeatable automated smoke checks, run from the repository root:

```powershell
.\scripts\windows\smoke-survivors.ps1 -DataDir "<your WoW 1.12.1 Data folder>" -Release
```

The following is a historical, manual debugging example rather than the automated checks script:

```powershell
$log = Join-Path $env:TEMP 'survivors-smoke.log'
$p = Start-Process -FilePath .\target\debug\warcraft-survivors.exe -PassThru -RedirectStandardError $log -WindowStyle Hidden
Start-Sleep -Seconds 120
$alive = Get-Process -Id $p.Id -ErrorAction SilentlyContinue
# ...then check the log lines in the table above, and stop the process by its id:
Stop-Process -Id $p.Id
```

## 3. Steps

| # | Step | How it is checked | Status |
| --- | --- | --- | --- |
| 1 | Native game launch | **Automated:** `warcraft-survivors.exe` starts with complete data, opens a window titled `Warcraft Survivors`, and still responds after 30 s. Without data it refuses to start and exits non-zero (automated and **Passed**, see below). | **Passed** with data; no-data refusal also passed. |
| 2 | Main menu rendering | **Automated:** `menu.png` captured. **Manual:** confirm the pick screen draws its art and text. | **Screenshot captured**; manual visual review open. |
| 3 | Character and map selection | **Automated:** the autopilot selects `WOW_SURVIVORS_MAP` and the class, and the log line `run begins as <class> at <zone>` matches. **Manual:** the picks appear on screen. | **Automated passed**; manual visual review open. |
| 4 | Starting a run | **Automated:** `run begins` line appears, then a `Playing` status line. | **Passed**. |
| 5 | Enemy spawning and movement | **Automated:** `alive=` is greater than zero in status lines and rises over time. **Manual:** enemies visibly move toward the hero in the screenshots. | **Spawning passed**; movement visual review open. |
| 6 | Automatic ability casting | **Automated:** `damage=` increases in status lines, and `build=` lists the owned abilities. **Manual:** spell visuals play. | **Casting passed**; spell visual review open. |
| 7 | Damage, health and death | **Automated:** `hp=` falls over time in `Warrior:idle`, and the line `run over: died` appears. | **Passed**; death results screen visual review open. |
| 8 | XP collection and level-ups | **Automated:** `level=` rises in status lines. | **Passed**. |
| 9 | Card selection | **Automated:** the LevelUp phase is reached and a pick is made, which the `level=` line confirms after the pick. **Manual:** card art, rarity colours and text are checked in `levelup-*.png`. | **Automated passed**; card visual review open. |
| 10 | Boss spawning | **Automated:** `survivors: boss <name> spawned` appears at about 135 s, then every 135 s. Use `<class>:late` to reach boss time quickly. | **Passed**. |
| 11 | Victory and defeat | **Automated:** `run over: won` in `:late` mode after about 5 minutes; `run over: died` in `:idle` mode. | **Passed** for logged outcomes; death results screen visual review open. |
| 12 | Returning to the menu | **Automated:** after a game over, the next `run begins` line appears about six seconds later, which means the menu ran and a new run started. | **Passed**. |

## 4. Results

Run `20261017-165007` (release build, commit `1f1cbbb`, `scripts/windows/smoke-survivors.ps1`):

| Check | Result | Evidence |
| --- | --- | --- |
| Launch refuses without data (step 1, no-data half) | **Passed** | `survivors.ps1 -Action Test`: exit code 1 with `Warcraft Survivors needs a WoW 1.12.1 Data folder`, from the dev binary with `WOW_DATA` empty. |
| Launch with data, window opens, stays responsive (step 1) | **Passed** | Window `Warcraft Survivors` created; 0 unresponsive samples. |
| Menu renders (step 2) | **Passed, screenshot saved**; art not yet viewed by the author | `menu\menu.png`. Visual check open. |
| Character and zone selection (step 3) | **Passed** | `run begins as Mage at The Barrens`. The pick is made by the autopilot, not by a click. |
| Run starts (step 4) | **Passed** | Status lines with `Playing` and `t` advancing. |
| Enemies spawn and move (step 5) | **Passed** for spawning (`alive` up to 253); movement visible in screenshots only | Status lines; `menu\shot-002.png`. |
| Automatic casting (step 6) | **Passed** | `damage` rises every status line; `build=` lists the owned spells. |
| Damage, health and death (step 7) | **Passed** | `run over: died at t=78s, level 7, kills 192`; hp reaches 0. The author did not see this run; the user reports a manual death in the same build. |
| XP and level-ups (step 8) | **Passed** | Level-up reached; `levelup-2.png` saved. |
| Card selection (step 9) | **Passed** for the pick path; card art needs a visual check | Autopilot picks at each `LevelUp`. |
| Boss spawns (step 10) | **Passed** | `survivors: boss ... spawned` lines in the victory leg (4 lines). |
| Victory (step 11) | **Passed** | `run over: won at t=900s, level 177, kills 5881`. The results screen was viewed: "Victory! The Barrens are yours.", 15:00 survived, 6 bosses. |
| Return to menu (step 12) | **Passed** | Second `run begins` line after the death. |
| Stability | **Passed** | 0 `ERROR` lines in all legs. |

Manual checks still open, for a person: menu art and pick screen, card art and rarity colours,
spell visuals and enemy movement, and the death results screen.

## 5. Automated tests versus manual checks

| Kind | What it covers | Where it runs |
| --- | --- | --- |
| Automated, no data | Historical Phase 1 result: 69 Survivors tests, including the four headless whole-run scenarios, plus the no-data launch refusal; not a current test count | `scripts/windows/survivors.ps1 -Action Test`, `.github/workflows/survivors-windows.yml` |
| Automated, data needed | Autopilot legs: steps 1 to 12 through the log, screenshots, and the perf journal | Local, `scripts/windows/smoke-survivors.ps1 -DataDir <Data>` |
| Manual | Visual checks: menu art, card visuals, spell effects, camera feel, audio | Local, with a screen |
