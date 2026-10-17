# Security review (Phase 1)

Scope: the repository at the start of Phase 1 (branch `phase1/windows-stability`), with the
Windows 11 native survivors build as the priority. The review is source-level and read-only. No
installer was run, no game archive was downloaded, and no build script was executed by the review.

Method:

1. A read-only pass over scripts, build scripts, CI, networking, file-write and process-launch
   code, and vendored sources (performed by a separate read-only review agent, then checked by me
   for every item that is listed below as confirmed).
2. A diff of the three vendored patches (`lua-src`, `mlua-sys`, `kira`) against their pinned
   crates.io releases. The three `.crate` files were fetched over HTTPS from `static.crates.io`
   into a temporary folder, about 1 MB in total, and deleted afterwards. This was the only network
   download in Phase 1 beyond normal cargo fetches.

## 1. Summary

| # | Sev | Confidence | Status | Finding |
| --- | --- | --- | --- | --- |
| S1 | MEDIUM | 7/10 | **Partly mitigated.** HTTPS is enforced. Checksum pinning is open. | `scripts/web-setup.sh` extracts GitHub release tarballs (wasi-sdk, binaryen) with no hash check. |
| S2 | MEDIUM | 7/10 | **Partly mitigated.** HTTPS is enforced. Checksum pinning is open. | `scripts/fetch-data.sh` downloads a 5.5 GB ZIP from archive.org and extracts it with `unzip`, with no hash check. |
| S3 | MEDIUM | 6/10 | **Open** (browser tooling only). | `crates/wenilla-host` `/ws/{port}` upgrades without an `Origin` check. `/data` has no `Host` check. The bind check only warns. |
| S4 | LOW | 6/10 | **Open** (browser dev page only). | `web/index.html` sets `dev_query_creds: '1'`, so a password can be typed into a URL. The file is copied into `web/dist/`. |
| S5 | MEDIUM | 7/10 | **Fixed.** Read-only token and pinned actions. Tag pin for `dtolnay/rust-toolchain` remains. | `.github/workflows/check.yml` had no `permissions:` block and used mutable action tags. |
| S6 | INFO | 9/10 | Not a vulnerability. Verified. | The game and the Windows script fetch no game data and no installers. Cargo fetches crates.io dependencies during a normal build, as it does for any Rust project. The WoW client is never fetched. |
| S7 | INFO | 8/10 | Verified. Not a vulnerability. | The three vendored patches differ from pinned upstream only in the files their `BENILLA.md` documents. |

No CRITICAL or HIGH findings were confirmed. No injection, path traversal, credential exposure,
telemetry or unexpected outbound connection was found in the game code.

## 2. Confirmed findings

### S1. Toolchain tarballs are extracted without a checksum (MEDIUM, 7/10)

- File: [scripts/web-setup.sh](../scripts/web-setup.sh) (wasi-sdk and binaryen steps).
- Evidence (before this change): `curl -sL "${url}" | tar xz ...`. `-s` without `-f` feeds an HTTP
  error page to `tar`. The version is overridable with `WASI_SDK_VERSION` and `BINARYEN_VERSION`.
- Impact: a compromised release asset, or a man-in-the-middle, would run code on the developer's
  machine. The wasi-sdk `clang` compiles the Lua C sources, and binaryen's `wasm-opt` rewrites the
  shipped module. `check.yml` runs this on every pull request, including forks.
- Phase 1 change: `curl -fsSL --proto '=https' --proto-redir '=https'`. HTTP errors now fail
  before `tar` runs, and non-HTTPS URLs are refused.
- Open: pin the SHA-256 of each platform tarball and verify it before extraction. The maintainer
  has to supply the hashes. They cannot be taken safely from the same host.

### S2. The game-data ZIP is downloaded and unzipped without an integrity check (MEDIUM, 7/10)

- File: [scripts/fetch-data.sh](../scripts/fetch-data.sh).
- Evidence (before this change): `curl -L -C - -o "$ZIP" "$URL"` followed by
  `unzip -o "$ZIP" 'Data/*.MPQ' -d "$DATA_DIR"`. `WOW_DATA_URL` may point to any URL.
- Impact: the MPQs feed the MPQ parser and the sandboxed Lua addon VM. The realistic exposure is
  the C `unzip` parser on a hostile archive, which needs an archive.org compromise or a
  man-in-the-middle. Path traversal is limited because `unzip` refuses `..` entries and the glob
  restricts extraction to `Data/*.MPQ`.
- Phase 1 change: `--proto '=https' --proto-redir '=https'`, so redirects cannot leave HTTPS.
- Open: pin the archive SHA-256. The archive is about 5.5 GB, and it was not downloaded in Phase 1,
  so the hash was not computed.
- The Windows native path does not run this script. The PowerShell workflow takes an
  already-present folder through `-DataDir` or `WOW_DATA` and never fetches data.

### S3. The browser dev relay accepts any origin (MEDIUM for browser tooling, 6/10, suspected impact)

- File: `crates/wenilla-host/src/ws.rs` (`upgrade`) and `crates/wenilla-host/src/main.rs`
  (`binds_loopback`).
- Confirmed in code: there is no `Origin` check anywhere in `wenilla-host`. `binds_loopback`
  only logs a warning when the bind address is not loopback. The process does not refuse to start.
- Suspected impact: a page the operator visits could open `ws://127.0.0.1:8090/ws/3724` and send
  login-protocol bytes to the configured upstream. This could enable password guessing against a
  local realm. Browser Private Network Access rules may block some of these paths, so
  exploitability is not confirmed.
- Survivors impact: none. Survivors does not start `wenilla-host`.
- Status: open. Recommended fix is an `Origin` allow-list and refusing non-loopback binds unless a
  flag is passed. Not in Phase 1 scope because it is browser tooling.

### S4. The shipped dev page enables credentials in the query string (LOW, 6/10)

- File: [web/index.html](../web/index.html) line 66, `window.__wenilla_env = { dev_query_creds: '1' }`.
  The gate is in `crates/benilla-app/src/webenv.rs` (`CREDENTIAL_KEYS`, `DEV_QUERY_CREDS`).
- Confirmed: `scripts/web-build.sh` copies this file verbatim into `web/dist/`.
- Impact: `?user=&pass=` links put a password into browser history and proxy logs. The realm
  service that would replace this page is not in this checkout.
- Status: open. Recommended fix is to default `dev_query_creds` off and enable it through a
  dev-only build step.

### S5. GitHub Actions hardening in `check.yml` (MEDIUM, 7/10). Fixed in Phase 1.

- File: [.github/workflows/check.yml](../.github/workflows/check.yml).
- Before: no `permissions:` block (the token scope fell back to the repository default), and
  `actions/checkout@v4` and `Swatinem/rust-cache@v2` were mutable tags. `ci.yml` already pinned
  these to commit SHAs.
- Phase 1 change: added `permissions: contents: read`. Pinned `actions/checkout` and
  `Swatinem/rust-cache` to the same SHAs `ci.yml` uses, and set `persist-credentials: false`.
- Open: `dtolnay/rust-toolchain@stable` is still a mutable tag in three jobs. Pin it by SHA when
  the maintainer agrees to the SHA update.

## 3. Verified as clean

- **No telemetry, analytics, update checks or outbound endpoints** in game code. The only
  outbound connections are the explicit ones in the download scripts above and the realm protocol
  in `benilla-protocol`, which the survivors mode does not use.
- **No git dependencies.** `Cargo.lock` has only `registry+` sources. The only `git =` strings are
  in the doc comment of `crates/benilla-app/examples/extended_launcher.rs`.
- **No procedural-macro crates** of our own.
- **Process launching.** `Command::new` appears only in `crates/benilla-buildstamp` (fixed `git`
  arguments, no user input).
- **Local state paths.** `crates/benilla-app/src/local_state.rs` is the only module that computes
  paths into `benilla-config/`. Realm and character names pass through `file_token` before they
  become file names.
- **MPQ parser.** `crates/benilla-mpq` bounds header-driven allocations and checks sector offsets.
  It contains no `unsafe` in the parsing code. All `unsafe` in the repository is platform FFI for
  audio, clipboard, QoS and timing.
- **SRP.** `benilla-srp` verifies the server's M2 value and uses `thread_rng` for ephemeral keys.
- **Credentials.** `.probe-identity` (`WOW_USER`, `WOW_PASS`, `WOW_CHAR`) is gitignored. The
  `pprobeN` values in the repository are documented dummies. No committed key or secret was found.
- **Web boot.** `web/boot.js` injects only static markup, and the `?bridge=` value is restricted
  to `^[a-z]+$`.
- **Phase 1 code.** The new PowerShell script downloads nothing, edits no machine setting, and
  writes only under `target/` and `benilla-config/`. It finds `cargo` through `PATH` and
  `%USERPROFILE%\.cargo\bin`, with no hardcoded machine path. It resets `WOW_DATA` to its previous
  state after its only environment-variable change. The launch check creates two temporary files,
  which are deleted.

## 4. Vendored patch verification

Each pinned crate was fetched from `static.crates.io` and compared with the repository copy
(line endings normalised). The results match the `BENILLA.md` documents.

| Crate (pinned) | Files that differ from upstream | Matches its BENILLA.md? |
| --- | --- | --- |
| `lua-src` 550.0.0 | `Cargo.toml`, `src/lib.rs`, `lua-5.1.5/lvm.c`, `lua-5.1.5/lparser.c`, `lua-5.1.5/luaconf.h`. Lua 5.2 to 5.5 source trees and the `BENILLA.md` file are absent or added. | Yes. `lib.rs` drops Lua 5.2 to 5.5 from `Version`, and adds a wasm32 build branch. |
| `mlua-sys` 0.10.0 | `src/lua51/lua.rs` only. | Yes. `lua_Integer` stays `i64` on wasm32. |
| `kira` 0.12.4 | `src/track/sub.rs` and `src/sound/streaming/decoder/symphonia.rs` only. | Yes. The mixer spatial-gain interpolation and the symphonia end-of-stream error. |

Remarks:

- `lua-src`'s `BENILLA.md` describes eight hunks. `git diff -U0` shows more change blocks, because
  it splits each logical change into several blocks. The file set matches exactly.
- The `lua-src` `lib.rs` wasm32 branch compiles C with `wasi-sdk` and does not run on Windows.
  It was not reviewed line by line for memory safety in the C it compiles, which is the upstream
  Lua 5.1.5 code with the eight documented hunks.
- The `lvm.c` change adds a substitution for `for k, v in table do`, which reads the global `next`.
  It is a deliberate compatibility behaviour, and `BENILLA.md` documents it.

## 5. Areas not verified

- The Python scripts (`doc-links.py`, `gen-reference-*.py`, `mixsum.py`, `pass-span-lint.py`,
  `soundprobe.py`, `vplsum.py`) were searched for `subprocess`, `urllib` and `eval`. They were not
  read in full.
- `scripts/gates.sh` beyond its first 120 lines, and `scripts/smoke.sh` beyond line 80, were not
  read in full.
- `scripts/probe-identity.sh`, `summon-live.sh`, `soundprobe.sh`, `cine.sh`, `genmap.sh` and
  `api-coverage.sh` were read at a high level. They are developer tooling.
- The `wenilla-realm` service is not in this checkout (`crates/` has no `wenilla-realm`). Its
  bind addresses, auth and secret handling could not be reviewed.
- Format parsers (DBC, BLP, M2, WMO, ADT) were not read line by line. Panics on malformed input
  are denial of service, not memory corruption.
- Lua bytecode is rejected by mlua's `ChunkMode::Text` at every `load` call site found. This is
  inferred from the call sites and was not tested with a crafted chunk.
- `dev` is a default feature in `benilla`, `warcraft-survivors` and `benilla-app`. It enables
  probes that read trusted environment variables. The browser build disables it.

## 6. Environment-variable trust

Several behaviours are controlled by environment variables: `WOW_DATA`, `BENILLA_HOME`,
`WOW_SURVIVORS_AUTO`, `WOW_SURVIVORS_SHOTS`, `WOW_SURVIVORS_MAP`, `WOW_GRADE`, and the probe
variables. These are trusted input from the person who starts the process. They do not come from
a network or from web content, except that on the web the page can set `__wenilla_env`. That path
is covered by S4.

## 7. Recommended next steps (not done in Phase 1)

1. Add SHA-256 pins for the wasi-sdk and binaryen tarballs, and for the archive.org data ZIP.
   Verify before extraction.
2. Add an `Origin` allow-list to `wenilla-host`'s `/ws` route, and refuse non-loopback binds unless
   an explicit flag is given (S3).
3. Default `dev_query_creds` to off in `web/index.html`, and enable it only through a dev-only
   build step (S4).
4. Pin `dtolnay/rust-toolchain` by SHA in `check.yml` (S5).
5. Read `scripts/gates.sh` and `scripts/smoke.sh` in full, and the Python scripts.
