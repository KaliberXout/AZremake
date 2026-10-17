# Azeroth Survivors — Project Status

**Current phase:** Phase 2 — Data-Driven Content System  
**Phase 1:** Complete and merged into `main`  
**Last updated:** 2026-10-17

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
- `tests.rs` — Survivors-specific automated tests.

This is a starting map, not an exhaustive architectural inventory. Check the current source before making assumptions about exact ownership or module responsibilities.

### Known architectural limitation

Some ability behaviour is selected using hardcoded spell identifiers.

For example, the casting implementation has special targeting behaviour for particular abilities such as Pyroblast, Aimed Shot, Exorcism, and Starfire.

This is a candidate for replacement with explicit ability data, such as a targeting strategy.

Do not assume that all existing hardcoded logic should be removed at once.

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

The first implementation candidate is replacing spell-name-specific targeting checks with an explicit targeting property in ability definitions.

This candidate must be reviewed against the actual source before implementation.

See:

- `docs/roadmap.md`
- `docs/architecture/content-system.md`
- `docs/tasks/next.md`

## 6. AI-Assisted Development

### Local development assistant

Continue (VS Code extension), connected to Ollama.

Primary local model:

- Qwen3 14B.
- Running locally on an AMD Radeon RX 9070 XT through ROCm.
- Approximately 32K configured context for coding tasks.

The local assistant has successfully performed an initial read-only analysis of the ability dispatch code.

Its implementation reliability has not yet been established.

### Task allocation

**Local AI / Continue**

- Small, well-defined code changes.
- Focused tests.
- Single-module improvements.
- Read-only source analysis.
- Routine maintenance.

**Cloud Codex**

- More difficult Rust and Bevy problems.
- Complex cross-module changes.
- Architectural work requiring substantial repository context.

**Human review**

- Architectural decisions.
- Scope control.
- Code review.
- Test verification.
- Gameplay and design decisions.

AI-generated changes must be reviewed and tested before being accepted.

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

1. Finalise the Phase 2 roadmap.
2. Document the proposed content-system architecture.
3. Define the first small implementation task.
4. Use Continue/Qwen3 to attempt that task.
5. Review the code changes and run targeted tests.
6. Commit only after verification.

**Current priority:** Establish a maintainable data-driven content foundation before significantly expanding the number of playable abilities, classes, or progression systems.