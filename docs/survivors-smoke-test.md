# Warcraft Survivors: smoke-test checklist

This checklist covers the native Windows build (`warcraft-survivors.exe`). It separates checks the
automated autopilot can make from checks that need a person looking at the screen.

**Current status (end of Phase 1): Blocked.** Every step below needs a complete WoW 1.12.1
`Data` folder, which is not on this machine. The launch check that does not need data has run and
passed. See the status column and section 4.

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

The checks script, to run once the data is in place:

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
| 1 | Native game launch | **Automated:** `warcraft-survivors.exe` starts with complete data, opens a window titled `Warcraft Survivors`, and still responds after 30 s. Without data it refuses to start and exits non-zero (automated and **Passed**, see below). | With data: **Blocked**. Without data: **Passed** (refusal). |
| 2 | Main menu rendering | **Manual** plus the `menu.png` screenshot from the autopilot. A person checks the pick screen draws its art and text. | **Blocked** (needs data and a screen) |
| 3 | Character and map selection | **Automated:** the autopilot selects `WOW_SURVIVORS_MAP` and the class, and the log line `run begins as <class> at <zone>` matches. **Manual:** the picks appear on screen. | **Blocked** |
| 4 | Starting a run | **Automated:** `run begins` line appears, then a `Playing` status line. | **Blocked** |
| 5 | Enemy spawning and movement | **Automated:** `alive=` is greater than zero in status lines and rises over time. **Manual:** enemies visibly move toward the hero in the screenshots. | **Blocked** |
| 6 | Automatic ability casting | **Automated:** `damage=` increases in status lines, and `build=` lists the owned abilities. **Manual:** spell visuals play. | **Blocked** |
| 7 | Damage, health and death | **Automated:** `hp=` falls over time in `Warrior:idle`, and the line `run over: died` appears. | **Blocked** |
| 8 | XP collection and level-ups | **Automated:** `level=` rises in status lines. | **Blocked** |
| 9 | Card selection | **Automated:** the LevelUp phase is reached and a pick is made, which the `level=` line confirms after the pick. **Manual:** card art, rarity colours and text are checked in `levelup-*.png`. | **Blocked** |
| 10 | Boss spawning | **Automated:** `survivors: boss <name> spawned` appears at about 135 s, then every 135 s. Use `<class>:late` to reach boss time quickly. | **Blocked** |
| 11 | Victory and defeat | **Automated:** `run over: won` in `:late` mode after about 5 minutes; `run over: died` in `:idle` mode. | **Blocked** |
| 12 | Returning to the menu | **Automated:** after a game over, the next `run begins` line appears about six seconds later, which means the menu ran and a new run started. | **Blocked** |

## 4. Results (Phase 1)

| Check | Result | Evidence |
| --- | --- | --- |
| Launch refuses without data (step 1, no-data half) | **Passed** | `survivors.ps1 -Action Test` ran the dev `warcraft-survivors.exe` with `WOW_DATA` empty. It exited with code 1 and printed `Warcraft Survivors needs a WoW 1.12.1 Data folder`. The output is in that run's console (Phase 1 session), and the script fails if the exit code is 0 or the message is missing. |
| Launch with data (step 1) | **Blocked** | No WoW 1.12.1 data is installed on this machine. Search of `C:\Azeroth`, `C:\Games`, `C:\Program Files`, `C:\Program Files (x86)`, `D:\`, `E:\`, `Downloads` and `Desktop` found only Battle.net's own MPQ. The client is not downloaded automatically. |
| Steps 2 to 12 | **Blocked** | Same cause. The autopilot needs the same data. |
| Before the Phase 1 fix (for the record) | Observed | Without data the old binary opened a window and logged `Asset Source 'AssetSourceId::Name(mpq)' does not exist` repeatedly, then stayed on `loading screen: waiting` with no user-facing error. |

To unblock: supply a `Data` folder with `-DataDir` and run the sequence in section 2. Record the
log lines and screenshots, and update the table above.

## 5. Automated tests versus manual checks

| Kind | What it covers | Where it runs |
| --- | --- | --- |
| Automated, no data | 62 unit tests for rules (damage, cooldowns, XP, rarity, offers, slots, spawn schedule, scaling, victory and death, targeting, seeded dice); the no-data launch refusal | `scripts/windows/survivors.ps1 -Action Test`, `.github/workflows/survivors-windows.yml` |
| Automated, data needed | Autopilot run: steps 3 to 8, 10 to 12 through the log | Local, with `-DataDir` |
| Manual | Visual checks: menu art, card visuals, spell effects, camera feel, audio | Local, with a screen |
