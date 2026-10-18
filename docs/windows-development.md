# Windows development (native, Windows 11 x64)

The native survivors game (`warcraft-survivors.exe`) is the Phase 1 target. This guide covers the
toolchain, the commands, how the WoW 1.12.1 data is configured, and what was verified on this
machine during Phase 1. Anything not run in that historical verification is marked as such.
Later Phase 1 results and Phase 2 checks are distinguished below; this is not a record of a new full validation run.

## 1. Summary

- Build: **passes** (dev and release, Rust 1.98.1, MSVC, Windows 11 SDK).
- Survivors unit, install and headless simulation tests: **passed** (69 tests in the later Phase 1 run; no game data needed).
- Launch without game data: **refuses to start**, exits non-zero, and names the folders it
  searched. Before Phase 1 it opened a window and hung on a loading screen.
- Launch with the user's WoW 1.12.1 data and the native smoke test: **passed**, 17 of 17 automated
  checks (run `20261017-165007`). See [survivors-smoke-test.md](survivors-smoke-test.md) and
  [testing.md](testing.md).
- In the later Phase 1 verification, workspace Clippy with `-D warnings` **passed**
  (`cargo clippy --workspace --all-targets -- -D warnings`). The recorded
  `cargo fmt --all -- --check` run failed on 13 files outside Survivors.
  The later full-library run recorded 4,293 passed and 1 failed (the `SweepUiMaterial`
  warm-up coverage test). These are historical results, not newly rerun checks.

## 2. Prerequisites (verified)

| Requirement | Version verified | How to get it |
| --- | --- | --- |
| Windows 11 x64 | 10.0.26200 (Windows 11 Pro) | — |
| Rust (rustup) | rustup 1.29.1 | `winget install --id Rustlang.Rustup -e` |
| Rust toolchain | 1.98.1 (pinned by `rust-toolchain.toml`, with clippy and rustfmt) | Installed automatically by the first `rustup`/`cargo` command in the repository |
| MSVC C++ build tools | MSVC 14.44 and 14.51 (Visual Studio 2026 Community, "18") | Visual Studio installer, workload **Desktop development with C++** |
| Windows SDK | 10.0.26100 | Included with the workload |
| PowerShell | Windows PowerShell 5.1 (`powershell.exe`) and PowerShell 7 both work | Built in |
| Disk | About 15 GB for `target/` (debug and release) | — |

Notes:

- Rust found `link.exe` without running `vcvars64.bat` in this shell. If linking fails with
  "linker `link.exe` not found", start the shell from **Developer PowerShell for VS**.
- The survivors build does **not** need Git Bash, `make`, `curl`, `unzip`, Node.js or WSL.
- Git for Windows is optional. Without it the build stamp reports no git information.

## 3. Commands

All commands run from the repository root in PowerShell:

```powershell
.\scripts\windows\survivors.ps1 -Action Check     # toolchain report, cargo check, WoW data status
.\scripts\windows\survivors.ps1 -Action Test      # survivors unit tests, then the no-data launch check
.\scripts\windows\survivors.ps1 -Action Lint      # rustfmt + clippy for the survivors crates (see section 6)
.\scripts\windows\survivors.ps1 -Action Build     # dev build: target\debug\warcraft-survivors.exe
.\scripts\windows\survivors.ps1 -Action Build -Release   # target\release\warcraft-survivors.exe
.\scripts\windows\survivors.ps1 -Action Run -DataDir "<your WoW 1.12.1 Data folder>"
.\scripts\windows\survivors.ps1 -Action Run -Release -DataDir "<your WoW 1.12.1 Data folder>"
```

The script:

- finds `cargo` on `PATH`, or in `%USERPROFILE%\.cargo\bin` if a new install is not on `PATH` yet;
- reports the MSVC installation found by `vswhere`;
- never downloads anything and never changes a machine setting;
- restores `WOW_DATA` to its previous state after its check;
- stops with a non-zero exit code if any step fails.

## 4. WoW 1.12.1 data

The game reads the client's `Data` folder at runtime. It contains the MPQ archives, and the game
cannot start without `terrain.MPQ` and `patch.MPQ`.

Where the game looks, in order (`crates/benilla-formats/src/install.rs`):

1. `WOW_DATA`. If it is set and empty, there is no install.
2. `<repository>\WoW\Data` (dev builds only).
3. `<exe folder>\Data`, then `<exe folder>\WoW\Data`.

`-DataDir` sets `WOW_DATA` for one run. No path in the repository is hardcoded to a machine.

Constraints:

- The client is supplied by you. Do not commit it. `/WoW`, `/wow-data` and `/extracted` are
  gitignored. Do not copy MPQ files into the repository.
- The game only reads the Data folder. Local state goes to `benilla-config\`, which is also
  gitignored (in a dev build, at the repository root; in a release build, beside the exe).
- If the folder is missing or incomplete, the game prints this and exits with code 1:

  ```
  Warcraft Survivors needs a WoW 1.12.1 Data folder, and none was found.
  Looked in:
    <each path searched>
  Set WOW_DATA to the Data folder that holds terrain.MPQ and patch.MPQ. See docs/windows-development.md.
  ```

  An incomplete folder gets a different message that names the missing files.

## 5. Why the existing `make native` target does not apply

`make native` runs `scripts/fetch-data.sh` (bash, curl, unzip; it downloads about 5.5 GB from
archive.org), then `cargo run --release -p warcraft-survivors` with `WOW_DATA=wow-data/Data`.
On Windows it fails for these reasons:

- `Makefile` sets `SHELL := /bin/bash`, and GNU make is not installed by default.
- `fetch-data.sh` and `web-setup.sh` need bash, curl and unzip.
- It downloads the client automatically, which Phase 1 forbids.

The PowerShell script replaces it for the native game. The bash scripts remain for the browser
build, which is secondary on Windows (section 7).

## 6. Verification log

Each row records a command from the earlier Phase 1 verification session and its result.
Some failures were resolved in subsequent Phase 1 work; the later results are documented in
[testing.md](testing.md). Do not treat the earlier failures or test counts as current results.

| # | Command | Result | Notes |
| --- | --- | --- | --- |
| 1 | `winget install --id Rustlang.Rustup -e` | **Passed** | rustup 1.29.1, installer hash verified by winget |
| 2 | `rustup show` (in repo) | **Passed** | Installed toolchain 1.98.1 with clippy and rustfmt |
| 3 | `cargo fmt --all -- --check` | **Failed** (pre-existing) | 13 files outside survivors: `ui_chat/settings.rs`, `benilla-assets/src/lib.rs`, `benilla-protocol/` (5 files), `benilla-world/` (3), `benilla-ui/src/script/tick.rs`. Phase 1 did not reformat them, because that is unrelated work. |
| 4 | `rustfmt --edition 2021 --check crates\benilla-app\src\survivors\mod.rs` | **Passed** | Historical result: covered the then-current Survivors modules, including `install.rs` and `tests.rs`. Tests have since been split under `survivors/tests/`. |
| 5 | `cargo check -p warcraft-survivors` | **Passed** | 2m 58s from cold |
| 6 | `cargo build -p warcraft-survivors` (dev) | **Passed** | 10m from cold |
| 7 | `scripts\windows\survivors.ps1 -Action Build -Release` (first) | **Passed** | 6m 33s |
| 8 | `cargo test -p benilla-app --lib survivors` | **Passed** | 62 passed, 0 failed (`survivors::tests` and `survivors::install::tests`) |
| 9 | Launch with `WOW_DATA=` (before the fix) | Observed | Opened a window titled "Warcraft Survivors", then stayed on `loading screen: waiting` with no error to the user. The log repeated `Asset Source 'AssetSourceId::Name(mpq)' does not exist`. |
| 10 | Launch with `WOW_DATA=` (after the fix), via `-Action Test` | **Passed** | Exit code 1, message printed, no window |
| 11 | `cargo test -p benilla-app --lib` (whole library) | **Failed**, 4 of 4287 | See below. None is caused by Phase 1. |
| 12 | `cargo clippy --no-deps -p benilla-app -p warcraft-survivors --all-targets -- -D warnings` (no allow flags) | **Passed** | After the seven pre-existing lints were fixed (see item 12 below) |
| 13 | Clippy with the same command and `-A` flags for the three lint names | Not needed | Superseded by row 12 |
| 14 | `cargo clippy --workspace --all-targets -- -D warnings` (what `ci.yml` runs) | **Failed** (pre-existing) | One `needless_borrow` in `benilla-world/src/rig_anim/compose.rs:198`. Platform-dependent, not changed. |
| 15 | `scripts\windows\survivors.ps1 -Action Lint` (WOW_DATA empty, no WoW folder in the repo) | **Passed** | rustfmt on survivors sources, then clippy `--no-deps` with `-D warnings`. Shows the Windows CI lint step needs no game data. |
| 16 | `scripts\windows\survivors.ps1 -Action Test` (final) | **Passed** | 62 unit tests and the no-data launch check (exit 1 with the message) |
| 17 | Patch diff: `lua-src` 550.0.0, `mlua-sys` 0.10.0, `kira` 0.12.4 against pristine crates.io copies | **Passed** | Differences match the `BENILLA.md` files. See [security-review.md](security-review.md) section 4. |
| 18 | `bash -n scripts/fetch-data.sh` and `bash -n scripts/web-setup.sh` (Git Bash) | **Passed** | Syntax only. Not executed. |
| 19 | `scripts\windows\survivors.ps1 -Action Build -Release` (second, from final code) | **Passed** | Cargo reports the release artifact fresh (0.46 s), so it was built from the final sources during the earlier release build. |

### Item 11: full library test run

Historical result: 4,283 passed, 4 failed, 10 ignored. The same four failed before and
after the lint commit. Three of those failures were subsequently fixed; the later Phase 1
run recorded 4,293 passed, 1 failed, 10 ignored. See [testing.md](testing.md), section 6.
The table below preserves the original failure investigation.

| Test | Cause | In Phase 1 scope? |
| --- | --- | --- |
| `pipe_warm::menagerie::tests::every_material_lane_has_a_warm_contributor` | `SweepUiMaterial`, registered in `survivors/mod.rs`, has no pipe_warm contributor. The missing contributor is a static warm-up coverage failure; later trace evidence showed the sweep pipeline compiling behind the loading cover, not an observed first-sight stall. The registration predates Phase 1. | **Yes, and not fixed.** Adding a contributor means extending the menagerie warm pass, which is delicate engine code. Still open as an engine warm-up follow-up; not automatically part of Phase 2 content-system scope. See section 8. |
| `ui_script::verb_event_gate::the_declared_rows_are_still_true` | Stale declarations for `ui_auction/mod.rs` (the file is `ui_auction.rs` in this checkout) | No |
| `ui_script::verb_event_gate::every_event_the_reference_fires_from_a_verb_is_fired_by_the_verb_s_module` | `AddSkillUp` fires `SKILL_LINES_CHANGED` from another module | No |
| `ui_session::tests::every_npc_session_is_portrait_bound_or_explicitly_excluded` | An NPC session is neither portrait-bound nor excluded | No |

### Item 12: clippy under the pinned toolchain

Seven pre-existing lints in `benilla-app` were fixed in a separate commit, with no behaviour
change: a type alias for each of the five `type_complexity` sites, an orphaned doc comment removed
(`hud.rs`), and a test module moved above a helper it preceded (`realmlist.rs`). The survivors
clippy gate now passes with no allow attributes.

| Location | Lint | Status |
| --- | --- | --- |
| `benilla-world/src/rig_anim/compose.rs:198` | `needless_borrow` | **Fixed** in `08c035ae`: `write_rig_worlds` takes `AsRef<[GlobalTransform]>`, so the Windows slice and the macOS `Vec` both fit with no auto-deref. Checked on Windows (clippy) and wasm (`cargo check`); macOS could not be built on this host, and the call shapes were probed. |

## 7. The browser build on Windows (secondary)

The browser build (`scripts/web-setup.sh`, `scripts/web-build.sh`, `wenilla-host`) is bash-based.
On Windows it needs Git Bash, and it downloads wasi-sdk and binaryen (see the checksum finding in
[security-review.md](security-review.md)). It was **not run** in Phase 1. `node` is not installed,
so `web/tests` was not run either. The `wasm32` `cargo check` from `check.yml` was not run.

## 8. Blockers and decisions

1. **The WoW 1.12.1 data blocker was resolved.** The user's Data folder ran the native smoke test
   (run `20261017-165007`, 17 of 17 automated checks). See [testing.md](testing.md).
2. **`SweepUiMaterial` has no warm-up contributor** (one failing engine coverage test
   in the later recorded run). The sweep pipeline was observed compiling behind the loading
   cover; this is not evidence of an in-play stall. Follow-up task; see
   [testing.md](testing.md), section 6.
3. **Clippy in workspace mode** passed in the later recorded Phase 1 run. The survivors crates are also gated by
   `survivors-windows.yml` with `--no-deps`, which needs no game data.
4. **`cargo fmt --all -- --check`** failed on 13 files outside Survivors in the
   recorded Phase 1 run. Those files were left unchanged.
5. **Browser build** was not verified on Windows during Phase 1.
6. **Phase 2 targeted verification:** the P2-001 targeting change (`0ad5a48`) was
   checked with `cargo check -p benilla-app --tests` and
   `cargo test -p benilla-app --lib targeting_` (7 passed, 0 failed). This does not
   establish a new full-library, lint, native smoke, or release-build result.

## 9. Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `cargo` not found | Install rustup (`winget install --id Rustlang.Rustup -e`), then open a new shell. The script also checks `%USERPROFILE%\.cargo\bin`. |
| `linker link.exe not found` | Run from Developer PowerShell for VS, or confirm the C++ workload is installed. |
| `vswhere.exe not found` | Install Visual Studio or Build Tools with the C++ workload. |
| Exit code 1 with "needs a WoW 1.12.1 Data folder" | Supply `-DataDir` or set `WOW_DATA` to a folder that contains `terrain.MPQ` and `patch.MPQ`. |
| Window opens and does not progress | Usually an incomplete install. Check stderr for `Asset Source 'mpq' does not exist`. |
| Build is slow the first time | Expected. A cold dev build takes about 10 minutes, and release takes about 7. Later builds are incremental. |
