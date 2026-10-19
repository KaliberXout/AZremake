# Working in this repository

This is Azeroth Survivors: a single-player, offline, WoW-inspired survivors game built on a
from-scratch WoW 1.12.1 client engine in Rust + Bevy. It forks
[samwhosung/benilla](https://github.com/samwhosung/benilla) (the engine) and
[Arnesen/wenilla](https://github.com/Arnesen/wenilla) (the browser build). Upstream is merged in,
not copied; see [docs/UPSTREAM.md](docs/UPSTREAM.md). Upstream's rules for the client crates are in
`docs/METHOD.md`, `docs/MAP.md` and `docs/CONTRIBUTING.md`. Read them before changing engine code.
This file is authoritative for this fork's goals, crates and process.

## Goals (read these first)

1. **Native Windows 11 x64 first.** The survivors game (`warcraft-survivors.exe`) is the product.
   Build and run it with `scripts/windows/survivors.ps1`. Keep the browser build working where
   practical, but do not let it drive decisions for the native path.
2. **Fully offline, single-player Survivors.** No server, no login, no account, no network
   gameplay. The survivors mode does not use the realm protocol.
3. **Keep the Rust/Bevy architecture.** Gameplay rules are plain Rust. Presentation is Bevy
   systems. Keep the two separable: pure rules get unit tests.
4. **Incremental changes.** Small, reviewable commits. No engine rewrites, no speculative
   abstractions. Phase 2 content-system work follows the incremental design in
   `docs/architecture/content-system.md`. The older `docs/content-architecture-proposal.md`
   is historical context, not the active implementation plan.
5. **Tests for behavioural changes.** A change to survivors rules adds or updates a test in
   `crates/benilla-app/src/survivors/` (see "Testing" below).
6. **No telemetry or unnecessary online services.** Do not add analytics, crash upload, update
   checks, or remote config. The only outbound traffic is the explicit data and toolchain downloads
   in the scripts, and those need a pinned checksum before they are trusted (see
   [docs/security-review.md](docs/security-review.md)).
7. **Never commit proprietary game assets, credentials, or large generated files.** The WoW 1.12.1
   client is supplied by the user at runtime. See "Protected content" below.

Phase planning and the current baseline are in `docs/architecture-baseline.md`,
`docs/security-review.md`, `docs/windows-development.md` and `docs/survivors-smoke-test.md`.

Survivors-specific work should stay in `crates/benilla-app/src/survivors/` where practical.
Changes to shared Benilla engine code or upstream carries require separate justification and
explicit user approval before editing; consider native and wasm compatibility.

## What lives where

| path                                                              | what                                                                                                        | whose                                       |
| ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `crates/warcraft-survivors`                                       | the survivors launcher (`warcraft-survivors.exe`)                                                           | ours                                        |
| `crates/benilla-app/src/survivors/`                               | the survivors mode: rules, HUD, menu, tests                                                                 | ours                                        |
| `crates/benilla-app/src/lib.rs` `run_survivors`                   | the survivors entry; checks the WoW install first                                                           | ours                                        |
| `crates/benilla-*`                                                | the client engine: formats, world, UI, protocol, app                                                        | upstream's, plus our `#[cfg]`-gated carries |
| `crates/benilla-app/src/webbridge/`                               | the JavaScript bridge, wasm side                                                                            | ours                                        |
| `crates/wenilla`                                                  | the wasm entry crate (`#[wasm_bindgen(start)]`)                                                             | ours                                        |
| `crates/wenilla-host`                                             | local dev server: static files, `/data/*`, `/ws/*` relay. Loopback only.                                    | ours                                        |
| `web/`                                                            | the page side: `index.html` (dev page), `boot.js`, `platform.js`, `bridge.js`, `examples/`, `wasi_stubs.js` | ours                                        |
| `scripts/windows/survivors.ps1`                                   | native Windows: check, test, lint, build, run the survivors game                                            | ours                                        |
| `scripts/windows/smoke-survivors.ps1`                             | native smoke test and performance baseline, against a Data folder you supply                                | ours                                        |
| `scripts/web-setup.sh`, `scripts/web-build.sh`                    | toolchain and build for the wasm                                                                            | ours                                        |
| `third_party/lua-src`, `third_party/mlua-sys`, `third_party/kira` | vendored and patched; each has a `BENILLA.md`                                                               | ours (vendored)                             |
| `.github/workflows/survivors-windows.yml`                         | the Windows survivors gate (tests, no-data launch check)                                                    | ours                                        |
| `.github/workflows/check.yml`                                     | the wasm check and the host and client-carry tests                                                          | ours                                        |
| `.github/workflows/ci.yml`                                        | fmt, clippy, tests and doc gates on Linux for fork pull requests                                            | upstream's, adapted                         |

`crates/wenilla-realm` (the realm service) is **not** in this checkout. Any older note that refers
to it describes a separate repository and does not apply here.

Survivors needs the engine core: `benilla-world`, `benilla-formats`, `benilla-assets`,
`benilla-protocol` (only the object-field types), `benilla-ui` (only the nameplate helper),
`benilla-mpq`, `benilla-blp`, `benilla-m2`, `benilla-wmo`, `benilla-adt`, `benilla-dbc`, and the
`net`, `creature_anim`, `glue` and `loading_screen` modules of `benilla-app`. The login, chat,
FrameXML and realm modules are compiled in but unused by survivors. Do not remove them as part of
survivors work; see `docs/architecture-baseline.md` section 3.

## Commands

Native Windows (PowerShell 5.1 or 7; no Git Bash, no `make`):

```powershell
.\scripts\windows\survivors.ps1 -Action Check                 # toolchain, MSVC, WoW data, cargo check
.\scripts\windows\survivors.ps1 -Action Test                  # survivors unit tests + no-data launch check
.\scripts\windows\survivors.ps1 -Action Lint                  # rustfmt + clippy for the survivors crates
.\scripts\windows\survivors.ps1 -Action Build -Release        # warcraft-survivors.exe (release)
.\scripts\windows\survivors.ps1 -Action Run -DataDir <Data>   # play; <Data> is a WoW 1.12.1 Data folder you own
.\scripts\windows\smoke-survivors.ps1 -DataDir <Data> -Release # native smoke test + perf baseline; writes target\smoke\<stamp>\report.md
```

Cargo directly, from the repository root:

```bash
cargo check -p warcraft-survivors                 # the native survivors binary
cargo test -p benilla-app --lib survivors         # survivors unit tests and headless simulation (no game data needed)
cargo fmt --all -- --check                        # fails on pre-existing drift in 13 files; see docs/windows-development.md
cargo clippy --workspace --all-targets -- -D warnings   # the Windows clippy gate; passes, see docs/testing.md
cargo run --release -p warcraft-survivors         # needs WOW_DATA or a WoW/Data folder in the repo
cargo survivors                                   # alias for the dev-profile run (.cargo/config.toml)
```

The workspace clippy gate passes on Windows and needs no game data. Do not add a platform split
to pass a lint: a platform-dependent fix must be checked on macOS and wasm too (see Invariants).

Browser build (bash: Linux, macOS, or Git Bash on Windows; this path is secondary):

```bash
scripts/web-setup.sh        # once per machine: wasm target, wasm-bindgen-cli at Cargo.lock's exact version, wasi-sdk, binaryen. No sudo.
scripts/web-build.sh        # → web/dist (WebGPU build). WEB_DEBUG=1 keeps symbols for readable stack traces.
cargo run --release -p wenilla-host -- --www web/dist --data /path/to/WoW/Data --upstream 127.0.0.1
                            # http://127.0.0.1:8090/ — needs a realmd:3724 and mangosd:8085 at --upstream
cargo check --target wasm32-unknown-unknown -p wenilla --no-default-features --features webgpu
                            # the wasm build without linking; what check.yml runs
cargo test -p wenilla-host                # the host's cache and relay tests
node --test web/tests/*.test.mjs          # browser startup and journal regressions
```

`make` targets (`make build`, `make start`, `make native`) are for Linux, macOS and Git Bash. The
`native` target needs GNU make, bash, curl and unzip, and it downloads the game data. On Windows,
use `scripts/windows/survivors.ps1`, which downloads nothing.

A browser for testing needs WebGPU; the world does not run without it. Linux Chrome needs
`--enable-unsafe-webgpu`. Without an adapter the client aborts in
`wgpu::create_bind_group_layout` at world entry: that is the missing adapter, not your change.
A throwaway profile with a devtools port, for driving it from a script:

```bash
google-chrome --user-data-dir=/tmp/wenilla-chrome --enable-unsafe-webgpu --remote-debugging-port=9333 http://127.0.0.1:8090/
```

## Invariants

- **Protected content.** Never commit WoW client files, MPQs, extracted assets, `WoW/`, `wow-data/`,
  `wow-addons-vanilla/`, `.probe-identity` (credentials), or `benilla-config/` (local state). All
  are gitignored. Never add a download that fetches the client. The user supplies it with
  `WOW_DATA` or `-DataDir`. On Windows, the launcher also reads `WOW_DATA` from the ignored
  repository-root `.env.local` when the process environment variable is unset; `-DataDir` takes
  precedence. Keep machine-specific paths in that file, never in tracked source.
- **Local state.** Only `crates/benilla-app/src/local_state.rs` computes paths into
  `benilla-config/`. New persistent state goes through it.
- **Dependencies.** Do not add a dependency without a reason in the commit message. Do not add a
  git dependency. Keep `Cargo.lock` committed. Do not add a procedural macro of our own without a
  review.
- **Warnings are not suppressed globally.** A lint is fixed, or allowed on the one item where the
  allow is justified and commented. Never add `#![allow]` at crate or workspace level to pass a gate.
- **Carries are `#[cfg]`-gated or in our own files.** A change inside an upstream file goes
  under `#[cfg(target_arch = "wasm32")]` or `cfg(not(target_os = "macos"))`, or into a new
  file. The next upstream merge conflicts on every upstream line a carry touches, so touch few.
- **Upstream is merged, never copied.** A sync is a real merge commit with `upstream/main` as
  its second parent. A single-parent content copy loses the ancestry and re-raises every
  conflict at the next sync. See docs/UPSTREAM.md.
- **Never press GitHub's "Sync fork" button.** On a diverged fork its "discard commits" path
  hard-resets `main`. Sync through a PR.
- **Mind the platform-dependent code.** A change that is correct only on Windows can break macOS
  or wasm (for example `rig_anim/compose.rs` returns a different type per target). Check the other
  targets' `cfg` branches before "fixing" a lint on one target.
- **What ships is the `cp` line in `scripts/web-build.sh`.** A page-side file that is not on it
  never reaches `web/dist`. `play.html` imports page modules dynamically and is never
  load-bearing, so a missing file is a console warning, not a black canvas.
- **Two pages, not one.** `web/index.html` is the dev page (`wenilla-host`). A page feature exists
  only where it was added.
- **`wasm-bindgen` crate and CLI versions must match exactly**; `web-setup.sh` enforces it.
- **No threads, no std time on wasm.** `std::time::Instant`, `SystemTime` and `env::vars_os`
  panic there; use `bevy::platform::time::Instant` or `web-time`. Upstream has no CI and never
  builds the wasm target, so a wasm-only break in an upstream change is caught only by
  `check.yml` or your own `cargo check --target wasm32…`.
- **Survivors rules are deterministic.** Randomness comes from the mode's `Rng` resource, which
  tests seed with `Rng::seeded`. Do not read the clock inside a rule.
- **`ring_reaction` returns 0..7**; every consumer adds `+ 1` for the Lua 1..8 scale.
- **`gh pr create` needs `-R <owner>/<repo>` for this fork.** Without it `gh` targets the parent
  repository, where pull requests may be closed.

## Testing

Full layers, commands and acceptance criteria are in `docs/testing.md`. The rules for a change:

- A change to survivors rules adds a test. Pure rules (damage, cooldowns, XP, rarity, offers,
  slots, schedules, scaling, targeting) live in `crates/benilla-app/src/survivors/tests/` or in
  an in-file `#[cfg(test)] mod tests` next to private helpers.
- A change to the run loop, the clock, the boss schedule, or the victory and death paths also keeps
  the headless simulation passing (`survivors/sim.rs`, the `scenarios` module). The simulation runs the
  production schedule (`add_run_loop`) on a manual clock, so a whole 900-second run takes about 2
  seconds and needs no assets. Do not write a second model of the game engine to test it.
- Tests use `Rng::seeded` and fixed values. They must not need WoW data, a window, or the clock.
- Prefer extracting a pure function from a Bevy system over testing through the ECS. Keep the
  extraction behaviour-preserving, and keep the call site as the only change in the system.
- Run `.\scripts\windows\survivors.ps1 -Action Test` before a survivors change is pushed.
- Data-gated tests use `wow_data_or_skip!` and skip without data. `BENILLA_REQUIRE_DATA=1` turns a
  skip into a failure on a machine that has the data.
- The native smoke test (`scripts/windows/smoke-survivors.ps1`) needs a Data folder you own. It is
  not part of CI. Run it on a Windows machine before changing combat, levelling, bosses or the
  results screens, and read its `report.md` for the manual checks it lists.
- Performance runs must not set a `WOW_FPS_*` variable without `WOW_BG=0`. Any `WOW_FPS_*` variable
  marks the run as a background run, which forces a 640x360 unfocused window and measures the wrong
  thing. `smoke-survivors.ps1` sets both.

## How a change ships

1. Branch from the current phase branch (or `main` once the phase lands). Open the pull request
   against this repository.
2. The PR gates run: `survivors-windows.yml` (Windows survivors tests and the no-data launch
   check), `check.yml` (wasm check, host and UI tests, client-carry tests) and `ci.yml` (fmt,
   clippy, tests and doc links on Linux, for fork pull requests).
3. Keep the commit message explicit about any dependency, any warning fixed or allowed, and any
   change to a platform branch.
4. Nothing deploys itself. The native game runs from a local build.

## Where to look when

| symptom                                                       | look at                                                                                        |
| ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| the game exits at start with "needs a WoW 1.12.1 Data folder" | point `WOW_DATA` or `-DataDir` at a folder that holds `terrain.MPQ` and `patch.MPQ`            |
| the game opens and sits on a loading screen                   | the install is incomplete or missing; check the stderr for `Asset Source 'mpq' does not exist` |
| a survivors rule seems wrong                                  | the rule's test in `survivors/tests/`; the numbers are in `survivors/data.rs`                  |
| `cargo build` fails to link on Windows                        | run `-Action Check`; confirm Visual Studio has the C++ workload and a Windows SDK              |
| the wasm panics at boot with `RuntimeError: unreachable`      | rebuild with `WEB_DEBUG=1`; check for a WebGPU adapter first; `?bridge=0` rules the bridge out |
| a page feature works on the dev page and not on the realm     | the `cp` line in `web-build.sh`, then `play.html` vs `index.html`                              |
| wasm-bindgen glue errors at load                              | version drift: rerun `scripts/web-setup.sh`                                                    |
| an upstream merge broke only the wasm build                   | `cargo check --target wasm32…`; the `cvars.rs` struct-vs-tuple case is the pattern             |
| no sound in the browser                                       | both pages resume AudioContext on the first gesture; mixer carry in `sound/mixer.rs`           |
| the world freezes at zone lines                               | wasm soundscape carry in `sound/zone.rs` and `sound/web_load.rs`                               |
| browser stalls when first playing a sound                     | deferred SFX carry in `sound/kit/deferred.rs` and related glue/cinematic code                  |
| the prod image differs from `main`                            | `wenilla-realm/docs/RELEASE.md` (separate repository), "When it breaks"                        |

## AI agent operating rules

These rules apply to every automated coding agent. **Preserving existing work takes priority over completing a task.** Instructions to stop, ask for approval, or avoid commands are mandatory, not suggestions.

### Scope and preparation

1. Read `docs/PROJECT_STATUS.md` and `docs/tasks/next.md` before implementation. Read other documentation only when relevant. For engine changes, also follow the upstream documents identified at the top of this file.
2. Inspect the specific source files and exact sections involved before editing. Identify the intended files and edits; do not conduct broad repository scans without a reason.
3. Work only within the user's requested scope. If the user requires source review or approval first, wait for it.
4. Preserve unrelated code, tests, comments, imports, formatting, and local changes. Never assume the working tree is clean.
5. Do not modify `AGENTS.md`, project status, or task backlog unless the task explicitly permits documentation updates. After verified implementation, **propose** any relevant status/backlog updates rather than making them automatically.

### Safe file editing

1. **Use `write`/create-file tools only for files verified not to exist.** A tool that replaces the complete contents of a path must never be used on an existing file for a small edit. Check file existence first.
2. For existing files, use a targeted edit/patch tool with a small, unique, exact-match context. Read the target section immediately before editing.
3. Never replace or reconstruct an entire existing file to insert, remove, or move a small section. Never recreate source from conversation history, model memory, or a partial tool response.
4. For a move/extraction, preserve test bodies and unrelated code verbatim except for necessary import/path changes. Verify that moved items appear once, not zero or twice.
5. After each edit, inspect the affected region. Before declaring completion, inspect the final diff and check for unexpected deletions or changes.
6. Do not use shell scripts, redirection, search-and-replace across entire files, or alternate tools to bypass a failed targeted edit.

### Failure limits and emergency stop

1. If a targeted edit fails, reread the current file and identify the mismatch. Make **at most one** corrected retry. After two failed attempts, **stop** and report the failure. Do not switch to a wholesale rewrite.
2. If an existing file is accidentally truncated, overwritten, deleted, or substantially changed outside scope, **stop all editing immediately**. Report the path, the operation and its observed effect. **Do not attempt recovery yourself.**
3. In particular, never reconstruct an accidentally overwritten file from tool output or chat history. Git or backups may contain the correct original, but only the user decides how to restore it.
4. Never run `git restore`, `git checkout`, `git reset`, `git clean`, `git stash`, or any other command that discards or replaces working-tree changes without explicit permission.
5. Do not repeat a failing command unchanged. A new attempt requires a specific, stated hypothesis and must respect any retry limit.
6. If blocked, stop and report what succeeded, what failed, and what remains unfinished. **Do not improvise an increasingly invasive workaround.**

### Commands, Git, and verification

1. Use Windows PowerShell commands on the native development machine. Do not stage, commit, push, amend, merge, rebase, or delete files without explicit user approval.
2. Never run `cargo clean`, delete build caches, or change `CARGO_TARGET_DIR` without permission. Do not run Cargo when the task forbids it.
3. Avoid unnecessary long-running builds. After a timeout or linker failure, report it; do not launch repeated expensive builds without a new hypothesis and approval where required.
4. A test passes only when the runner explicitly reports success and at least one relevant test executed. A timeout, partial output, compile failure, or linker error is **not** a pass.
5. Report exactly which verification commands were run, their outcomes, and anything not verified. Never claim success based only on editing-tool approval.

### Completion and handoff

1. Review the changed files and diff. Report the exact files created or modified and any unexpected changes.
2. State what was verified and what still needs human testing or review.
3. Stop after the assigned task. Do not start follow-up work independently.

**Emergency rule:** If uncertain whether an action could destroy existing work, do not perform it. Ask the user first.

### Mandatory completion report

At the end of every coding task, report:

- **Modified files:** Every existing file changed during this task.
- **Created files:** Every new file created during this task, including untracked files.
- **Deleted files:** Every file deleted during this task, or explicitly state "None".
- **Verification:** Exact commands run and their outcomes. Distinguish passed, failed, skipped and not run.
- **Remaining work:** Anything incomplete, unverified or requiring user review.

Record the initial Git status before editing so pre-existing changes are not mistaken for changes made during the task.

Do not rely on `git diff` alone to identify new files: untracked files are excluded.

Never claim that a task is complete without reporting its full file impact.
