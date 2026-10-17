# Windows development (native, Windows 11 x64)

The native survivors game (`warcraft-survivors.exe`) is the Phase 1 target. This guide covers the
toolchain, the commands, how the WoW 1.12.1 data is configured, and what was verified on this
machine. Anything not run is marked as such.

## 1. Summary

- Build: **passes** (dev and release, Rust 1.98.1, MSVC, Windows 11 SDK).
- Unit tests for the survivors rules: **pass** (62 tests, no game data needed).
- Launch without game data: **refuses to start**, exits non-zero, and names the folders it
  searched. Before Phase 1 it opened a window and hung on a loading screen.
- Launch with game data and the gameplay smoke test: **Blocked**. No WoW 1.12.1 client is on this
  machine, and the client is not downloaded automatically. See
  [survivors-smoke-test.md](survivors-smoke-test.md).
- The repository-wide gates (`cargo fmt --all --check`, workspace clippy with `-D warnings`) **fail
  on pre-existing code**. Section 6 lists the exact items.

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

Each row is a command that was actually run in this Phase 1 session, with its result.

| # | Command | Result | Notes |
| --- | --- | --- | --- |
| 1 | `winget install --id Rustlang.Rustup -e` | **Passed** | rustup 1.29.1, installer hash verified by winget |
| 2 | `rustup show` (in repo) | **Passed** | Installed toolchain 1.98.1 with clippy and rustfmt |
| 3 | `cargo fmt --all -- --check` | **Failed** (pre-existing) | 13 files outside survivors: `ui_chat/settings.rs`, `benilla-assets/src/lib.rs`, `benilla-protocol/` (5 files), `benilla-world/` (3), `benilla-ui/src/script/tick.rs`. Phase 1 did not reformat them, because that is unrelated work. |
| 4 | `rustfmt --edition 2021 --check crates\benilla-app\src\survivors\mod.rs` | **Passed** | Covers every survivors module, including the new `install.rs` and `tests.rs`. |
| 5 | `cargo check -p warcraft-survivors` | **Passed** | 2m 58s from cold |
| 6 | `cargo build -p warcraft-survivors` (dev) | **Passed** | 10m from cold |
| 7 | `scripts\windows\survivors.ps1 -Action Build -Release` (first) | **Passed** | 6m 33s |
| 8 | `cargo test -p benilla-app --lib survivors` | **Passed** | 62 passed, 0 failed (`survivors::tests` and `survivors::install::tests`) |
| 9 | Launch with `WOW_DATA=` (before the fix) | Observed | Opened a window titled "Warcraft Survivors", then stayed on `loading screen: waiting` with no error to the user. The log repeated `Asset Source 'AssetSourceId::Name(mpq)' does not exist`. |
| 10 | Launch with `WOW_DATA=` (after the fix), via `-Action Test` | **Passed** | Exit code 1, message printed, no window |
| 11 | `cargo test -p benilla-app --lib` (whole library) | **Failed**, 4 of 4287 | See below. None is caused by Phase 1. |
| 12 | `cargo clippy --no-deps -p benilla-app -p warcraft-survivors --all-targets -- -D warnings` | **Failed** (pre-existing) | 7 lints in `benilla-app`, none in Phase 1 code (see below) |
| 13 | Same clippy command with the three pre-existing lint names allowed, for verification only | **Passed** | Confirms Phase 1 code is lint-clean. Not part of any configuration. |
| 14 | `cargo clippy --workspace --all-targets -- -D warnings` (what `ci.yml` runs) | **Not run to completion** | Fails first in `benilla-world` (section below), the same pre-existing lint |
| 15 | `scripts\windows\survivors.ps1 -Action Test` (final) | **Passed** | 62 unit tests and the no-data launch check (exit 1 with the message) |
| 16 | Patch diff: `lua-src` 550.0.0, `mlua-sys` 0.10.0, `kira` 0.12.4 against pristine crates.io copies | **Passed** | Differences match the `BENILLA.md` files. See [security-review.md](security-review.md) section 4. |
| 17 | `bash -n scripts/fetch-data.sh` and `bash -n scripts/web-setup.sh` (Git Bash) | **Passed** | Syntax only. Not executed. |
| 18 | `scripts\windows\survivors.ps1 -Action Build -Release` (second, from final code) | **Passed** | Cargo reports the release artifact fresh (0.46 s), so it was built from the final sources during the earlier release build. |

### Item 11: full library test run

4,283 passed, 4 failed, 10 ignored.

| Test | Cause | In Phase 1 scope? |
| --- | --- | --- |
| `pipe_warm::menagerie::tests::every_material_lane_has_a_warm_contributor` | `SweepUiMaterial`, registered in `survivors/mod.rs`, has no pipe_warm contributor. Its pipelines would compile on first use during play, not behind the loading cover. This is a real first-sight stall risk. The registration predates Phase 1. | **Yes, and not fixed.** Adding a contributor means extending the menagerie warm pass, which is delicate engine code. Recommended for Phase 2. See section 8. |
| `ui_script::verb_event_gate::the_declared_rows_are_still_true` | Stale declarations for `ui_auction/mod.rs` (the file is `ui_auction.rs` in this checkout) | No |
| `ui_script::verb_event_gate::every_event_the_reference_fires_from_a_verb_is_fired_by_the_verb_s_module` | `AddSkillUp` fires `SKILL_LINES_CHANGED` from another module | No |
| `ui_session::tests::every_npc_session_is_portrait_bound_or_explicitly_excluded` | An NPC session is neither portrait-bound nor excluded | No |

### Item 12: clippy under the pinned toolchain

| Location | Lint | Owner |
| --- | --- | --- |
| `benilla-world/src/rig_anim/compose.rs:198` | `needless_borrow`. The fix differs by platform: on Windows `scratch.compose` returns a slice, and on macOS `rig_worlds` returns an owned value. A one-token change that is right on Windows would break macOS, which cannot be built here. Not changed. | upstream |
| `benilla-app/src/survivors/hud.rs:1648` | `empty_line_after_doc_comments` | pre-existing survivors code |
| `benilla-app/src/survivors/grade.rs:77` | `type_complexity` | pre-existing survivors code |
| `benilla-app/src/webbridge/nearby.rs:7` | `type_complexity` | webbridge |
| `benilla-app/src/webbridge/snapshot.rs:47, 61, 188` | `type_complexity` | webbridge |
| `benilla-app/src/realmlist.rs:145` | `items_after_test_module` | upstream |

These are left in place. Fixing them would refactor unrelated code, which the phase brief rules
out. The decision on whether to fix them is listed in section 8.

## 7. The browser build on Windows (secondary)

The browser build (`scripts/web-setup.sh`, `scripts/web-build.sh`, `wenilla-host`) is bash-based.
On Windows it needs Git Bash, and it downloads wasi-sdk and binaryen (see the checksum finding in
[security-review.md](security-review.md)). It was **not run** in Phase 1. `node` is not installed,
so `web/tests` was not run either. The `wasm32` `cargo check` from `check.yml` was not run.

## 8. Blockers and decisions

1. **No WoW 1.12.1 data on this machine.** This blocks steps 1 (with data) and 2 to 12 of the
   smoke test. Provide a `Data` folder and the sequence in
   [survivors-smoke-test.md](survivors-smoke-test.md) can run.
2. **`SweepUiMaterial` has no warm-up contributor** (a failing engine test and a first-sight
   stall risk). Recommended Phase 2 work.
3. **Clippy under the pinned toolchain** fails on 7 lints in `benilla-app` and 1 in `benilla-world`.
   Options: fix them (small, surgical edits across webbridge, realmlist, grade and hud), or keep
   them and let the Windows gate run clippy with `--no-deps` on the survivors-only items later.
4. **`cargo fmt --all --check`** fails on 13 files outside survivors.
5. **Browser build** was not verified on Windows.

## 9. Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `cargo` not found | Install rustup (`winget install --id Rustlang.Rustup -e`), then open a new shell. The script also checks `%USERPROFILE%\.cargo\bin`. |
| `linker link.exe not found` | Run from Developer PowerShell for VS, or confirm the C++ workload is installed. |
| `vswhere.exe not found` | Install Visual Studio or Build Tools with the C++ workload. |
| Exit code 1 with "needs a WoW 1.12.1 Data folder" | Supply `-DataDir` or set `WOW_DATA` to a folder that contains `terrain.MPQ` and `patch.MPQ`. |
| Window opens and does not progress | Usually an incomplete install. Check stderr for `Asset Source 'mpq' does not exist`. |
| Build is slow the first time | Expected. A cold dev build takes about 10 minutes, and release takes about 7. Later builds are incremental. |
