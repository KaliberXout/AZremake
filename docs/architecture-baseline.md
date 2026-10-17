# Architecture baseline (Phase 1)

Status: written at the start of Phase 1 from source inspection, then updated as the phase
progressed. Every claim here was checked against the code or a command, not against the README.
Where the README and the code disagree, the code is recorded here.

## 1. What the repository is

Azeroth Survivors is a single-player, offline survivors game built on a from-scratch WoW 1.12.1
client engine (Rust, Bevy 0.18). The client engine (`benilla-*`) reads a legally owned 1.12.1
`Data` folder at runtime and ships no Blizzard content. The survivors mode is layered on top of
that engine, in `crates/benilla-app/src/survivors/`.

The workspace has 20 members (`crates/*`) and one pinned toolchain (`rust-toolchain.toml`,
Rust 1.98.1 with clippy and rustfmt).

## 2. Entry points

| Binary / library | Crate | Role | Native Windows |
| --- | --- | --- | --- |
| `warcraft-survivors.exe` | `crates/warcraft-survivors` | Survivors launcher. `main.rs` stamps the build and calls `benilla_app::run_survivors`. | **Yes. This is the Phase 1 target.** |
| `benilla.exe` | `crates/benilla` | Full WoW client launcher (FrameXML, networking, login). | Builds; not the target. Needs a realm server for its main flow. |
| `wenilla-host` | `crates/wenilla-host` | Loopback dev server for the browser build (static files, `/data`, `/ws` relay). | Builds; browser tooling only. |
| `wenilla` | `crates/wenilla` | wasm32 entry crate (`#[wasm_bindgen(start)]`). | Browser only (`scripts/web-build.sh`). |
| `benilla-app` (lib) | `crates/benilla-app` | The client app, including `run_survivors`, `run`, `run_with`. | Library. |

Other binaries and examples (`blpx`, `mpqx`, `benilla-worldview` shim, `examples/`) are developer
tools and are out of Phase 1 scope.

## 3. Survivors mode: module map

Source size is about 317 KB across 13 modules. Nothing in this directory had tests before Phase 1.

| Module | Size | What it owns |
| --- | --- | --- |
| `mod.rs` | 13.6 KB | `SurvivorsPlugin`, `Phase` state machine, `Run`, `Owned`, `Stats`, `Rng`, `Selection`. |
| `data.rs` | 61.8 KB | Classes, abilities, passives, enemies, bosses, maps, rosters, wave clock, XP curve. Pure data and pure functions. |
| `abilities.rs` | 27.6 KB | Auto-cast: cooldowns, target selection, hit scheduling, ground areas, totems, `apply_hits` (damage, DOT, effects, death). |
| `enemies.rs` | 16.6 KB | Horde spawning on a ring, boss and ring schedule, chase, contact damage, XP gems, loot. |
| `hero.rs` | 18.1 KB | Hero entity, third-person / top-down camera, movement input, `hurt_hero`, `hero_vitals` (regen, buffs, death, victory). |
| `hud.rs` | 79.9 KB | HUD, level-up offer (`choices_for`, `pick`), results screen, icon and tooltip plumbing. |
| `cards.rs` | 20.5 KB | Card visuals, rarity (`power`, `spell_tier`, `rarity`), deal and pick animation. |
| `menu.rs` | 25.6 KB | Pick screen (race, sex, class, zone), `survivors-pick.txt` persistence. |
| `minimap.rs` | 20.3 KB | Minimap composition from the zone's art. |
| `mobility.rs` | 8.9 KB | Class movement spell on F from level 5. |
| `units.rs` | 12.0 KB | Spawning local units through the same attach/animation path a streamed unit uses. |
| `grade.rs` | 3.4 KB | Fullscreen grade pass (CAS, vibrance, S-curve). |
| `auto.rs` | 9.6 KB | `WOW_SURVIVORS_AUTO` autopilot for hands-off runs and screenshots. |

### Runtime shape

- A Bevy `States` machine (`Phase`): `Menu`, `Playing`, `LevelUp`, `Paused`, `GameOver`.
  `pause` and `unpause` drive `Time<Virtual>`.
- Systems are chained in sets after `WorldStage::Input`. The gameplay chain runs only in
  `Phase::Playing` and only while the loading screen is not covering the world.
- Per-run state lives in the `Run` resource and is reset by `reset_run` on return to the menu.
  Everything spawned for a run carries `RunEntity` and is despawned there.
- Cross-system messages: `abilities::Hit` (one blow on one enemy) and `enemies::Loot`
  (an enemy died here). Delayed blows wait in `PendingHits`.
- Randomness is a single xorshift64* `Rng` resource seeded from the clock.

### Dependencies on the rest of the client

The survivors mode is essential in the sense that it runs on the engine, but it is entangled with
parts of the client that exist for the full game:

- **Engine core, required:** `benilla-world` (`WorldStage`, `WorldCamera`, `WorldPoint`,
  `rig_anim`, `world_map`), `benilla-formats` (`wow_data`, `candidates`, `tile_to_world`,
  minimap translate), `benilla-assets` (`WorldAssets`, `MapCatalogRes`), `benilla-protocol`
  (`ObjectFields`, `ObjectType`, used to fake streamed-object fields), `crate::net` (`ObjectStore`,
  `Guid`, `FieldChanged`), `crate::creature_anim` (movement, cast and swing messages),
  `crate::combat_text`, `crate::run_mode::CaptureMode`, `crate::loading_screen`,
  `crate::screen_fade`, `crate::local_state`.
- **Client UI pieces reused for presentation, not gameplay:** `crate::glue` (glue art and
  backdrops, used by the pick screen), `crate::char_select::race_name` (race strings),
  `benilla_ui::script::nameplate` (nameplate text), `crate::ui_action::Spells` (spell catalogue
  for projectile speed), `crate::area::AreaTableRes`.
- **Unrelated to survivors, present because it is the full client:** login and SRP
  (`benilla-srp`, `login`, `realmlist`, `realm_select`, `transport`), chat and FrameXML
  (`ui_chat`, `ui_script`, `ui_*` panels, the Lua addon VM in `benilla-ui`), the protocol session
  (`benilla-protocol::world::session`), the wasm bridge (`webbridge`) and the realm-side host.
  These are compiled into `benilla-app` for every binary. Survivors disables FrameXML and the
  player controller with `CaptureMode` but does not remove the code.

Phase 1 does **not** remove these. Splitting the engine is an architectural change outside scope.
The recommendation for Phase 2 is recorded in section 9.

## 4. Native launch path (as observed)

1. `warcraft-survivors::main` builds a `BuildId` and calls `benilla_app::run_survivors`.
2. Phase 1 added `benilla_formats::set_project_folder` and a start check
   (`survivors::install::check_wow_data`) before any window. Without a complete Data folder the
   process prints the folders it looked in and exits non-zero.
3. Before the change, the same launch opened a window and sat on a loading screen indefinitely,
   logging `Asset Source 'AssetSourceId::Name(mpq)' does not exist`. That is the defect the check
   fixes (see the status table in `docs/windows-development.md`).

Data resolution (`crates/benilla-formats/src/install.rs`): `$WOW_DATA` first (set and empty means
"no install"), then `<repo>/WoW/Data` for dev builds, then `<exe dir>/Data` and
`<exe dir>/WoW/Data`. No path is hardcoded.

Local state (`crates/benilla-app/src/local_state.rs`): `$BENILLA_HOME`, else
`<repo>/benilla-config/` for dev builds, else `<exe dir>/benilla-config/`. Survivors writes
`survivors-pick.txt` there. `benilla-config/` and `/.probe-identity` are gitignored.

## 5. Existing tests, utilities and logging

- **Test counts before Phase 1:** the `benilla-app` lib test binary has 4,297 tests. Survivors had
  none. Many existing tests skip when no WoW install is present (`wow_data_or_skip!`, logged
  through `BENILLA_SKIP_LOG`). `BENILLA_REQUIRE_DATA=1` turns those skips into failures.
- **Test utilities:** `crate::local_state::test_env::{EnvGuard, ENV_LOCK}` for environment
  isolation, `crate::test_support`, `wow_data_or_skip!`.
- **Logging:** Bevy's tracing subscriber, written to stderr. Panics leave
  `benilla-config/Diagnostics/crash-<unix>.txt` (`crash.rs`).
- **Scripted gates:** `scripts/check.sh` (scoped, in-loop), `scripts/gates.sh` (full land chain),
  `scripts/smoke.sh` (live logout/re-login against a server), `scripts/doc-links.py`,
  `scripts/pass-span-lint.py`. All are bash or Python. `make` and Git Bash are not part of the
  Windows toolchain.
- **Autopilot:** `WOW_SURVIVORS_AUTO=<class>[:idle|:all|:late]` with optional
  `WOW_SURVIVORS_MAP` and `WOW_SURVIVORS_SHOTS`. It is the only automated way to exercise a full
  survivors run, and it needs WoW data and a display.

## 6. Build configuration

- Workspace `Cargo.toml`: `[profile.dev]` opt-level 1, dependencies opt-level 3 (avian/parry
  collision drops below 1 fps at opt-level 0). `[profile.ship]` fat LTO. `[profile.play]`
  incremental release for the run loop.
- `.cargo/config.toml`: aliases `cargo play` (benilla) and `cargo survivors` (warcraft-survivors).
- `[patch.crates-io]`: `lua-src`, `mlua-sys` and `kira` are vendored with patches. Verified in
  Phase 1 against pinned registry copies; see `docs/security-review.md` section 4.
- `Cargo.lock` has only crates.io sources. There are no git dependencies and no procedural-macro
  crates of our own.
- Makefile: `native` runs `fetch-data.sh` (bash, curl, unzip) and then
  `cargo run --release -p warcraft-survivors` with `WOW_DATA=wow-data/Data`. It sets
  `SHELL := /bin/bash`, so it does not run on Windows without GNU make and bash.

## 7. CI

| Workflow | Runner | Runs | Needs WoW data |
| --- | --- | --- | --- |
| `ci.yml` | ubuntu | fmt, clippy `--workspace`, `cargo test --workspace`, doc-links, pass-span-lint, player tests | No (`WOW_DATA=""`) |
| `check.yml` | ubuntu | wasm32 `cargo check`, host and UI tests, client-carry tests | No |
| `survivors-windows.yml` (new in Phase 1) | windows-latest | survivors unit tests, no-data launch check | No |

## 8. Status of the existing gates on the Windows baseline

Measured on Windows 11 x64 with MSVC 14.5x, Windows SDK 10.0.26100, and Rust 1.98.1:

- `cargo check -p warcraft-survivors`: passes.
- `cargo build -p warcraft-survivors` (dev): passes, about 10 minutes from cold.
- `cargo fmt --all -- --check`: **fails** on 13 pre-existing files outside survivors
  (`ui_chat/settings.rs`, `benilla-assets/src/lib.rs`, `benilla-protocol/*`, `benilla-world/*`,
  `benilla-ui/src/script/tick.rs`). Survivors files are clean.
- `cargo clippy --no-deps -p benilla-app -p warcraft-survivors -- -D warnings`: **passes**.
- `cargo clippy --workspace --all-targets -- -D warnings`: **passes** after the `benilla-world`
  `needless_borrow` fix (`AsRef` on `write_rig_worlds`, checked on Windows and wasm; macOS not
  buildable here) and an example type alias in `benilla-formats`.

Details and decisions are in `docs/windows-development.md`.
