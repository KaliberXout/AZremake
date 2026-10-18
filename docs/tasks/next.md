# Azeroth Survivors — Next Development Task

**Phase:** 2 — Data-Driven Content System  
**Task ID:** Pending — next scoped task not yet agreed
**Status:** Read-only source review and task selection
**Assigned assistant:** Not assigned
**Last updated:** 2026-10-18

## 1. Current verified repository history

P2-001 (data-driven ability targeting) is implemented in commit `0ad5a48` on `phase2/content-system`. The earlier implementation review recorded a successful `cargo check -p benilla-app --tests` and `cargo test -p benilla-app --lib targeting_` (7 passed). These were focused checks, not a clean full-suite result.

Subsequent committed test-organisation work:

- `447bdfd` — split Survivors tests into modules.
- `c87cd7c` — extract ability tests.
- `755acb2` — extract hero stats and passives tests.
- `c19ef97` — extract RNG tests.

At the last Git inspection, `HEAD` and `origin/phase2/content-system` both pointed to `c19ef97`. This is a snapshot, not a guarantee about future repository state. Do **not** repeat P2-001 or restart the test extraction from the obsolete `survivors/tests.rs` layout.

## 2. Next action — read-only inspection

Before assigning another coding task:

1. Inspect `git status --short` and the relevant current source and test modules. Preserve all pre-existing modifications and untracked files.
2. Inspect `crates/benilla-app/src/survivors/tests/mod.rs` and its existing child modules, and confirm what tests remain in `mod.rs` before proposing further extraction.
3. Inspect `crates/benilla-app/src/survivors/data.rs` and its dependencies if considering content-definition modularisation. Do not assume its size or boundaries from older planning notes.
4. Review the active design in `docs/architecture/content-system.md` and the recorded history in `docs/tasks/completed.md`.
5. Propose **one** small, behaviour-preserving next task with exact files, acceptance criteria and narrow verification commands. Obtain user approval before implementation.

The next task ID, ownership and scope are **not yet decided**. Do not create a speculative P2-002 implementation or start a broad refactor.

## 3. Safety and verification rules

- Follow `AGENTS.md` and the user's explicit approval workflow.
- Inspect existing tests before adding tests; avoid duplicates and changes to established targeting semantics.
- Preserve unrelated code, tests, documentation and local changes. Do not run `cargo fmt --all`.
- Use native Windows PowerShell commands and the narrowest relevant Cargo test or check command.
- Distinguish compilation, linking, executed tests and passed assertions. Report failures and incomplete verification accurately.
- Do not stage, commit, push, reset, restore, clean, delete or otherwise alter Git state without explicit permission.
- After an approved edit, inspect the affected region, `git --no-pager diff -- <path>`, `git diff --check`, and `git status --short` before reporting completion.

## 4. Task-selection checkpoint

**Current priority:** Agree on the next maintainable, incremental Phase 2 improvement based on the actual code. The historical P2-001 specification has been superseded by its committed implementation; refer to commit `0ad5a48` and `docs/tasks/completed.md` for history rather than treating it as outstanding work.
