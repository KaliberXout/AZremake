# Testing

This document records the Phase 1 native Windows 11 x64 testing baseline for Azeroth Survivors:
what each layer proves, the commands used, and the open items recorded at that time. Results
below are historical, not claims that the same tests have been rerun on the current Phase 2 HEAD.
Every recorded number comes from the repository history or `target\smoke\` (not committed).

## 1. Layers

| # | Layer | Where | Needs WoW data | What it proves |
|---|---|---|---|---|
| 1 | Rule unit tests | `crates/benilla-app/src/survivors/tests/` (split into modules after Phase 1), and in-file `mod tests` in `abilities.rs`, `cards.rs` | No | Damage, cooldowns, stats and passives, rarity, offers, slot limits, schedules, scaling, XP, outcome, targeting, dice |
| 2 | Launch check tests | `survivors/install.rs` | No | The start check names the folders it searched, and names missing files |
| 3 | Headless simulation | `survivors/sim.rs` (test-only) | No | A whole 900-second run on the game's schedule: bosses, cards, pause, waves, slots, victory, defeat, determinism |
| 4 | Source guards | `ui_script/verb_event_gate.rs`, `ui_session.rs`, `pipe_warm/menagerie.rs` | No | Upstream invariants about event wiring and warm-up coverage |
| 5 | Lint | clippy and rustfmt | No | Correctness and style gate for the workspace and survivors crates |
| 6 | Native smoke | `scripts/windows/smoke-survivors.ps1` | **Yes** (your install, read-only) | The real game launches, shows its menus, runs, levels, bosses, death, victory and returns to the menu |
| 7 | Performance baseline | the perf leg of layer 6 | **Yes** | Per-frame work and memory for the largest crowd |

Layers 1 to 5 run in CI without any game asset. Layers 6 and 7 are local, and they read the Data
folder you point them at. Nothing is downloaded, and no game file is copied into the repository.

## 2. Commands

Run from the repository root in PowerShell.

```powershell
# Layers 1-3: the survivors unit, install and simulation tests (no data).
cargo test -p benilla-app --lib survivors

# Layers 1-3 plus the no-data launch check, as CI runs them.
.\scripts\windows\survivors.ps1 -Action Test

# Layer 5, survivors crates (rustfmt on the survivors sources, clippy with -D warnings).
.\scripts\windows\survivors.ps1 -Action Lint

# Layer 5, the whole workspace, as ci.yml runs it.
cargo clippy --workspace --all-targets -- -D warnings

# Every library test, to see the full picture (the one known failure is documented in section 5).
cargo test -p benilla-app --lib

# Layers 6 and 7: the native smoke test and performance baseline (needs a WoW 1.12.1 Data folder).
.\scripts\windows\smoke-survivors.ps1 -DataDir "<your WoW 1.12.1 Data folder>" -Release
```

A single simulated run, for debugging, is the `scenarios` module:

```powershell
cargo test -p benilla-app --lib survivors::sim -- --nocapture
```

## 3. The headless simulation

The harness (`survivors/sim.rs`) drives the game's own code, not a model of it:

- The schedule is `add_run_loop`, the same function the plugin calls. A change to the game's
  combat or transitions reaches the simulation with no edit to the harness.
- A run starts with `hud::start_run`, the hero is spawned with `hero::spawn_hero`, offers come from
  `hud::roll_offer`, and cards are chosen with `hud::pick`.
- The clock steps 1/20 s per frame (`TimeUpdateStrategy::ManualDuration`), so a 900-second run is
  18,000 frames. The dice are `Rng::seeded`, and the executor is single-threaded, so the run is
  repeatable frame for frame.

The harness controls only three things: the seed, the step, and the card policy (the dice choose a
card). Invulnerability (`immune_t`) is an explicit switch, used only to reach victory, since a
stationary hero would otherwise never finish a run.

Two harness stand-ins, both documented in the code:

- **Corpses.** The game removes a dead enemy through the renderer's fade teardown
  (`benilla_world::model_fade`, crate-private and render-side). No rule reads a corpse, so the
  harness despawns a corpse one second after its fade is armed. Without this the simulated crowd
  grew to 8,000 corpses in a 900-second run; with it the crowd is the one the game shows.
- **Executor.** Bevy does not order conflicting unordered systems under its multi-threaded
  executor, so the harness runs the schedule on one thread. The game does not change.

What the four scenario tests assert:

| Test | Asserts |
|---|---|
| `a_full_900_second_run_reaches_victory_on_the_game_schedule` | Victory on the first frame at or past 900 s; all six bosses in roster order, each within three frames of its 135-second mark; one card per level gained after the first, none owed; the clock stops while cards are up; the first minute draws from the opening tiers and the last minute from the top tiers; more than 1,000 kills; slot limits held; upgrades taken |
| `the_same_seed_replays_the_same_victory_frame_for_frame` | Two runs with one seed produce identical samples and outcomes |
| `a_hero_who_does_not_survive_ends_the_run_in_defeat` | Defeat before 900 s; the clock stops at the death; health reaches zero; the death plays for about three seconds; no crowd arrives after it; every boss came before it |
| `the_seed_decides_the_run` | Two seeds play two runs |

The 900-second victory run takes about 2 seconds of wall time in the test profile.

## 4. Native smoke test and performance baseline

`scripts/windows/smoke-survivors.ps1` runs four legs of the real game against your Data folder,
driven by the game's autopilot (`WOW_SURVIVORS_AUTO`), so no input is sent and no window needs
focus:

| Leg | Run | Ends when | Automated checks |
|---|---|---|---|
| menu | Mage on The Barrens, 60 s | the time is up | window opens, menu captured, run begins, damage dealt, level 2 reached, card screenshot, no errors |
| death | Warrior:idle (stands still) | the run reports a death and a second run begins | death reported, health reaches zero, results screen, return to the menu, no errors |
| victory | Mage:late (two thirds through, top-rank pool) | the run reports a win | win reported at 900 s, at least two boss spawns, results screen, no errors |
| perf | Mage:late in the normal window, 120 s, frame journal on | the time is up | journal rows recorded |

The report (`report.md`, written under `target\smoke\<timestamp>\`) lists every check with its
evidence, the performance figures, the pipelines compiled live during play, and the screenshots
for the manual checks.

The script exits non-zero when any automated check fails.

**Manual checks** (a person looks at the screenshots; the report gives the paths): menu art and
the pick screen, card art and rarity colours, spell visuals and enemy movement, and the results
screens. The script cannot judge these.

**Performance measurement.** The perf leg runs the release build with vsync off (`WOW_NOVSYNC=1`),
so its frame time is the game's own pace, and the journal's `main_ms` and `gpu_ms` are the per-frame
work. The vsync-on legs are limited by the display, so their frame rate is recorded for comparison
only. Earlier dev-profile runs on this machine sat at about 60 fps even with vsync off; that
difference was not isolated, and the baseline uses only the release figures. The adapter that drives
the game window is not recorded, since the script lists every adapter WMI reports.

The journal and `WOW_BG`: any `WOW_FPS_*` variable marks a run as an instrumented background run,
which forces a 640×360 unfocused window. The perf leg sets `WOW_BG=0` so it measures the normal
window. An earlier version of the leg measured the background window by mistake.

**Caveat on the script version.** The figures in section 5 come from run `20261017-165007`. Two
report changes were made after that run: the mean FPS is split into vsync-on and vsync-off, and the
GPU list shows every adapter. The parse check passes, but the new report code has not run end to end,
because the one run started after those edits was interrupted. Re-running the script on the next
change will exercise it.

## 5. Acceptance criteria and results

The phase is complete when each row below passes on a clean checkout, with the commands above.
Results are from the original Phase 1 verification runs (commit `1f1cbbb` and later), not assumed to describe the current Phase 2 checkout.

| Criterion | Command | Result |
|---|---|---|
| Native Windows build passes | `cargo build -p warcraft-survivors` (dev and `--release`) | **Passed** |
| Survivors unit and regression tests pass | `cargo test -p benilla-app --lib survivors` | **Passed**, 69 tests (including the 4 simulation scenarios) |
| Headless simulation passes | `survivors::sim::scenarios` | **Passed**, 4 tests, about 2 s |
| Full relevant library tests pass, or an upstream failure is documented | `cargo test -p benilla-app --lib` | **Passed** 4,293 of 4,294; **1 failed**, documented in section 6 |
| Clippy passes for the Windows targets | `cargo clippy --workspace --all-targets -- -D warnings` | **Passed** (exit 0) |
| GitHub Actions Windows CI passes | `survivors-windows.yml` | **Not run at the time of this local report**: the branch had not yet been pushed. Phase 1 was subsequently merged; consult the relevant GitHub Actions runs for CI evidence. |
| Native gameplay smoke test confirms death and victory | `smoke-survivors.ps1`, run `20261017-165007` | **Passed**: 17 of 17 automated checks. Death and victory are logged and the victory results screen was viewed. The user reports a manual death and end-of-run screen. Visual review of the other screens is open. |
| Performance baseline recorded | the perf leg, run `20261017-165007` | **Recorded**, section 4 |
| Security and development docs current | `docs/` | Updated for this phase |

### Performance baseline

Release build, perf leg (Mage:late, the largest crowd, vsync off, normal window), steady state from
the game's frame journal (`perf\journal.csv`, first five rows dropped as start-up):

| Measure | Value |
|---|---|
| Frame time, mean | 6.69 ms (about 150 fps) |
| Frame time, worst p95 | 11.24 ms |
| Main-thread work per frame, mean | 5.81 ms |
| GPU work per frame, mean | 2.86 ms |
| CPU time per interval, mean | 11.56 ms |
| Peak entities | 15,132 |
| Peak live enemies | 253 (status lines, all legs) |
| Peak working set | 1,695 MB |
| Pipelines compiled live during play | 3 kinds: `screenshot-to-screen` (our capture tool), `ui_gradient_pipeline`, `box_shadow_pipeline` |

Vsync-on legs ran at about 175 to 180 fps, matching the 180 Hz display, so the earlier note that
frames were capped at 60 Hz was wrong for this release build. The 60 Hz readings came from earlier
dev-profile runs and were not isolated.

Levels in the 150s appear in the late-start and invulnerable runs. Level has no ceiling by design
(section 6).

### Interrupted run

A second smoke run (`20261017-170621`) was started to correct the report's wording and was stopped
partway through the death leg, before its report was written. It is not used for any figure here.
Its contents are left in `target\smoke\` and are not committed.

### Later Phase 2 verification (separate from the Phase 1 baseline)

After P2-001 introduced explicit targeting metadata (commit `0ad5a48`), the recorded
`cargo test -p benilla-app --lib targeting_` run passed **7 tests**, with **0 failed** and
**4,299 filtered out**. This was a targeted test run, **not** a full-library rerun.
Subsequent commits `447bdfd`, `c87cd7c`, `755acb2`, and `c19ef97` split the Survivors
tests into modules. Verify current test paths and run validation on the actual checkout
before treating the historical results above as current.

## 6. Known open items

- **`SweepUiMaterial` has no warm-up contributor.** One test in the full library run fails for
  this reason (`pipe_warm::menagerie::tests::every_material_lane_has_a_warm_contributor`). Trace
  evidence: the sweep's pipeline is compiled behind the loading cover (covered), so the risk is a
  static coverage gap rather than an observed stall. A contributor needs a `bevy_ui` warm node that
  can only be verified on macOS, where compilation is synchronous. It is not exempted, and the
  decision is open.
- **Two `bevy_ui` pipelines compile live in play.** `ui_gradient_pipeline` and
  `box_shadow_pipeline` compile about eight seconds into the first run, after the cover, on every
  run tried. This is an engine warm-up coverage hole, distinct from the sweep. The fix is in the
  warm pass, and it is left for a decision.
- **Corpse teardown in the harness** is a stand-in (section 3).
- **Experience has no ceiling.** `spend_xp` loops while experience meets the next level's cost.
  Infinite experience would never end the loop. Gameplay cannot produce it (experience is a finite
  amount times a finite rate), so it is a hazard, not a defect. The smoke run reached level 147.
  Levels are not capped by design, and the results screen shows the number as is.
- **macOS and wasm** were not built for the `benilla-world` change. The macOS build needs an Apple
  C toolchain, which this host lacks. The call shapes were checked with a probe, and the wasm
  `cargo check` passes.
- **Linux CI** (`ci.yml`) was not run from this machine.
- **The four full-library failures from Phase 1** are now one. Two verb-gate tests and one
  `ui_session` test were Windows-only defects in the tests themselves (path separators, CRLF) and
  are fixed in `8c035ae`. The remaining failure is `pipe_warm::menagerie::tests::every_material_lane_has_a_warm_contributor`,
  which is the `SweepUiMaterial` item above. Full library run: 4,293 passed, 1 failed, 10 ignored.
- **Follow-up tasks (non-critical):**
  1. Add a warm-up contributor for `SweepUiMaterial` and investigate the two live `bevy_ui`
     pipelines (after the gameplay checks are signed off).
  2. Explain the vsync-on frame cap: vsync-on frames sit at the display's 180 Hz, but earlier dev
     runs read 60 Hz. Not isolated.
  3. Record the game's adapter in the report (WMI lists both the iGPU and the discrete GPU).
  4. Decide whether levels should have a ceiling. The harness shows levels in the 150s; the code
     has no cap. Only a `spend_xp` guard against non-finite experience exists.
  5. Historical follow-up: push the Phase 1 branch and check `survivors-windows.yml` on GitHub (Phase 1 was subsequently merged).
  6. Run the Linux `ci.yml` on a fork PR, and the wasm build (`scripts/web-setup.sh`) on a machine
     with bash.
