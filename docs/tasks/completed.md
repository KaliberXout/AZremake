# Azeroth Survivors — Completed Development Tasks

**Last updated:** 2026-10-17

This document records completed development work, implementation decisions, and verification results.

Its purpose is to preserve project history and prevent developers or AI assistants from repeatedly investigating problems that have already been solved.

## Task Completion Rules

A task is considered complete when:

- The agreed implementation is finished.
- Relevant tests and validation have passed.
- Changes have been reviewed.
- The implementation has been committed.
- Related documentation has been updated where appropriate.

Record meaningful implementation details and any known limitations.

Do not mark tasks complete merely because an AI assistant reports that it has finished.

---

## Phase 1 — Stability and Testing

**Status: Complete**  
**Merged:** 2026-10-17  
**Pull request:** https://github.com/neily2nd/azeroth-survivors-re/pull/1

### P1-001 — Native Windows Development Foundation

**Status:** Complete

Established and validated the native Windows development workflow for Azeroth Survivors.

**Results:**
- Native application builds and runs.
- Windows development scripts are available.
- Rust/MSVC toolchain is working.
- Local World of Warcraft 1.12.1 assets can be used without committing them to the repository.

### P1-002 — Automated Gameplay Testing

**Status:** Complete

Expanded the Survivors automated tests and improved coverage of gameplay edge cases.

**Results:**
- 69 Survivors tests passing.
- Additional tests for gameplay boundary conditions.
- Existing game behaviour preserved.

### P1-003 — Full-Run Headless Simulation

**Status:** Complete

Implemented a headless gameplay simulation covering the full 15-minute survival duration.

**Results:**
- 900-second simulation completes.
- Provides a repeatable regression test without manually playing an entire run.

### P1-004 — Native Smoke Testing

**Status:** Complete

Added native smoke-testing support for important gameplay paths.

**Results:**
- 17/17 smoke checks passed locally.
- Victory outcome verified.
- Death outcome verified.
- A successful victory run reached approximately 5,881 kills and six bosses.

### P1-005 — Performance Baseline

**Status:** Complete

Established an initial performance reference for future optimisation and regression testing.

| Metric | Recorded value |
|---|---|
| Mean frame time | 6.69 ms |
| P95 frame time | 11.24 ms |
| Peak entities | 15,132 |
| Peak live enemies | 253 |
| Approximate working set | 1.7 GB |

These measurements are historical reference values, not guaranteed performance targets.

### P1-006 — Continuous Integration

**Status:** Complete

Established and validated the relevant GitHub Actions workflows.

**Results:**
- Windows native CI passed.
- WASM CI passed.
- Host/UI CI passed.
- Client-carries CI passed.
- Phase 1 pull request merged successfully.

**Notable fixes:**
- Corrected JavaScript test mocks to support required DOM operations.
- Corrected Rust formatting in a Survivors test.
- Addressed Windows-specific source-scanning test behaviour.

### P1-007 — Documentation and Development Tooling

**Status:** Complete

Added supporting development documentation, scripts, and testing guidance during Phase 1.

The more detailed Phase 2 planning documents are tracked separately and are not part of this completed task.

### Known Outstanding Issue

A broader local Benilla test run passed 4,293 library tests but encountered one pre-existing `SweepUiMaterial` warm-up test failure.

This issue did not prevent the Phase 1 pull request from passing its required CI checks.

It remains a known limitation rather than a completed fix.

---

## Phase 2 — Data-Driven Content System

**Status: In progress**

### P2-001 — Explicit Ability Targeting Configuration

**Status:** Not completed — tracked in `docs/tasks/next.md`

This task will be added to the completed history only after implementation, review, testing, and commit.

---

## Completion Entry Template

For future tasks, use the following format:

### P2-XXX — Task Title

**Status:** Complete  
**Completed:** YYYY-MM-DD  
**Implementation:** Continue / Codex / Manual  
**Commit:** `<commit hash>`

**Objective:** Briefly describe the problem.

**Changes:** Explain what was implemented and which areas were affected.

**Verification:** Record tests and validation results.

**Decisions:** Record architectural choices that may affect future development.

**Limitations:** Record anything deliberately deferred.

---

## Maintenance

- Keep completed tasks in chronological order within each phase.
- Do not remove historical entries when implementation details change.
- Link to commits or pull requests where practical.
- Avoid duplicating large sections of architecture documentation.
- Update `docs/PROJECT_STATUS.md` when completed work changes the current project state.