# Azeroth Survivors — Project Status

**Current phase:** Phase 2 — Data-Driven Content System  
**Phase 1:** Complete and merged into `main`  
**Last updated:** 2026-10-18

## 1. Project Overview

Azeroth Survivors is a World of Warcraft-inspired, single-player survival roguelite in the style of Vampire Survivors.

The project is based on a fork of the Benilla engine and uses Rust, Bevy, and assets from the World of Warcraft 1.12.1 client.

Repository: https://github.com/neily2nd/azeroth-survivors-re

### Primary development target

- Native Windows desktop application.
- Single-player gameplay.
- Development environment: Windows 11, VS Code, PowerShell.
- Browser/WASM compatibility is secondary.
- Multiplayer is a possible future feature, not a current development priority.

### Design direction

Build a replayable survival roguelite using World of Warcraft's classes, abilities, enemies, environments, equipment, and progression systems as inspiration.

Priorities include:

- Distinct playable classes and builds.
- Abilities that behave differently rather than simply dealing different amounts of damage.
- Interesting ability combinations and evolutions.
- Meaningful progression and player choices.
- Data-driven content that can be expanded without repeatedly modifying combat engine code.

## 2. Current Playable State

The game has a functioning native Windows build and a playable survival run.

Confirmed systems include:

- Player movement and combat.
- Automatic ability casting.
- Multiple ability behaviours and targeting mechanisms.
- Enemy spawning, combat, and scaling.
- Experience and level-up choices.
- Boss encounters.
- Run completion, including victory and death outcomes.
- A 15-minute survival run.

The existing ability implementation supports several behaviour categories, including projectiles, melee attacks, area effects, damage-over-time effects, defensive effects, and healing.

**Important:** Existing gameplay functionality does not imply that every system is fully data-driven or production-ready.

## 3. Phase 1 — Complete

Phase 1 focused on stabilising the existing game and establishing a reliable development and testing baseline.

### Completed work

- Native Windows build and development workflow.
- Automated testing improvements.
- Additional edge-case tests.
- Full-duration, headless 900-second gameplay simulation.
- Native smoke-testing scripts.
- Performance measurement scripts.
- Rust linting and formatting checks for relevant Survivors sources.
- GitHub Actions validation for Windows and existing web-related components.
- Documentation and supporting development tooling.

### Validation results

The following results were established during Phase 1:

| Validation | Result |
|---|---|
| Survivors automated tests | 69 passed |
| Native smoke tests | 17/17 passed |
| Full headless simulation | 900 seconds completed |
| Victory path | Verified |
| Death path | Verified |
| GitHub Windows CI | Passed |
| GitHub WASM CI | Passed |
| GitHub host/UI CI | Passed |
| GitHub client-carries CI | Passed |

The native smoke run reached the 15-minute victory condition with approximately 5,881 enemy kills and six bosses encountered.

A broader Benilla test run also passed 4,293 library tests, with one pre-existing `SweepUiMaterial` warm-up test failure observed locally. This should not be represented as a completely clean full-workspace test run.

### Performance baseline

Recorded during Phase 1:

| Metric | Baseline |
|---|---|
| Mean frame time | 6.69 ms |
| 95th percentile frame time | 11.24 ms |
| Peak entity count | 15,132 |
| Peak live enemies | 253 |
| Approximate working set | 1.7 GB |

These are reference measurements, not guaranteed performance targets. Future changes should be measured under comparable conditions.

### Phase 1 integration

Phase 1 was merged through GitHub pull request #1:

https://github.com/neily2nd/azeroth-survivors-re/pull/1

All four reported required test workflows passed before merging. One CI gate was skipped.

## 4. Current Technical Architecture

### Technology

- Language: Rust.
- Game framework: Bevy.
- Existing engine foundation: Benilla.
- Primary platform: Windows.
- Assets: World of Warcraft 1.12.1 client data.

Local WoW client data is stored outside the repository and must not be committed.

### Relevant source areas

The Survivors-specific implementation is located primarily under:

`crates/benilla-app/src/survivors/`

Known relevant files include:

- `abilities.rs` — ability definitions, casting, targeting, and combat effects.
- `data.rs` — gameplay data and related definitions.
- `hero.rs` — hero-related gameplay behaviour.
- `hud.rs` — player interface and level-up choices.
- `tests/` — Survivors-specific automated tests (inspect the current module layout).

This is a starting map, not an exhaustive architectural inventory. Check the current source before making assumptions about exact ownership or module responsibilities.

### Known architectural limitation

P2-001 replaced the spell-name-specific targeting check with explicit ability metadata (commit `0ad5a48`). `AbilityDef` carries `Targeting::Nearest` or `Targeting::Toughest`; Pyroblast, Aimed Shot, Exorcism, and Starfire use `Toughest`, while the default is `Nearest`.

Other hardcoded gameplay rules remain candidates for incremental review. Do not assume they should all be removed at once.

## 5. Phase 2 — Current Focus

**Goal:** Make gameplay content easier to define, maintain, extend, and balance without embedding individual spell rules throughout the combat engine.

The intended direction includes:

- Explicit ability metadata.
- Data-driven targeting and behaviour configuration.
- Clear separation between content definitions and execution logic.
- Validation of content definitions.
- Focused automated tests for new content behaviour.
- Incremental migration of existing abilities.

### Development strategy

Prefer small, independently testable changes.

Do not begin with a large rewrite of `abilities.rs` or the entire Survivors module.

Preserve existing gameplay behaviour unless a change is explicitly intentional.

The first implementation milestone, P2-001, is complete and committed as `0ad5a48`. Subsequent commits `447bdfd`, `c87cd7c`, `755acb2`, and `c19ef97` split Survivors tests into modules. These changes do not complete the wider data-driven content architecture.

Review the current source and agree on the next small milestone before authorising implementation.

See:

- `docs/roadmap.md`
- `docs/architecture/content-system.md`
- `docs/tasks/next.md`

## 6. AI-Assisted Development

### Current coding-agent approach

Codex is the intended tool for future reviewed implementation tasks. Earlier local experiments used Continue/Ollama and Kilo Code; do not infer successful implementation or test verification from an agent's completion message.

Before assigning work, inspect the current Git status and the relevant source and tests. Preserve existing changes and regression tests; the previously planned test modularisation is already committed. Follow `AGENTS.md` for mandatory editing safeguards.

The coding agent handles narrowly scoped, approved changes and reports exact verification results. The human developer approves architecture, reviews diffs, and controls staging, commits, and merges.

## 7. Development Rules

1. Native Windows is the primary target.
2. Keep changes small and scoped.
3. Prefer existing patterns and project tooling.
4. Do not introduce unnecessary dependencies.
5. Do not perform broad refactors without an agreed plan.
6. Avoid unrelated formatting changes.
7. Use the repository's targeted lint and test scripts.
8. Preserve existing functionality unless explicitly changing it.
9. Keep WoW client assets and proprietary data out of Git.
10. Update project documentation when architecture or milestones change.

## 8. Immediate Next Steps

1. Finish the Phase 2 documentation consistency audit while preserving existing source and uncommitted work.
2. Treat P2-001 as implemented and committed (`0ad5a48`), with seven targeted tests previously reported as passed. The broader test suite and native lint were not rerun for that change; confirm current Git status before new work.
3. Treat the Survivors test extraction as completed through `c19ef97`; inspect the current test modules before proposing any further restructuring.
4. Agree on the next small coding task, review its source before assignment, and verify its results before approving any commit.

**Current priority:** Establish a maintainable data-driven content foundation before significantly expanding the number of playable abilities, classes, or progression systems.