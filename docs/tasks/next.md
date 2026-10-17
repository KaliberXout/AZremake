# Azeroth Survivors — Next Development Task

**Phase:** 2 — Data-Driven Content System  
**Task ID:** P2-001  
**Title:** Replace hardcoded ability targeting with explicit configuration  
**Status:** Ready for source inspection  
**Assigned assistant:** Continue / Qwen3 14B  
**Last updated:** 2026-10-17

## 1. Objective

Remove the existing dependency on specific spell names when deciding whether an ability targets the nearest or toughest enemy.

Represent this choice explicitly in the ability definition while preserving current gameplay behaviour.

This is the first small migration towards the data-driven content architecture.

## 2. Background

The existing casting code in:

`crates/benilla-app/src/survivors/abilities.rs`

appears to contain logic equivalent to:

```rust
if matches!(
    def.key,
    "pyroblast" | "aimed_shot" | "exorcism" | "starfire"
) {
    toughest(&foes, reach, p.count)
} else {
    nearest(&foes, reach, p.count)
}
```

The exact source must be inspected before making changes.

The problem is that targeting behaviour depends on the spell identifier rather than a property of the ability.

This makes it harder to introduce new abilities that reuse the same targeting behaviour.

## 3. Desired Behaviour

An ability definition should explicitly determine whether it targets:

- The nearest valid enemy.
- The toughest valid enemy.

Existing abilities must continue using the same targeting rules.

In particular:

- Pyroblast should retain its existing targeting behaviour.
- Aimed Shot should retain its existing targeting behaviour.
- Exorcism should retain its existing targeting behaviour.
- Starfire should retain its existing targeting behaviour.
- Other abilities should retain their existing default targeting behaviour.

Do not change targeting range, target count, damage, cooldowns or other ability mechanics.

## 4. Proposed Implementation

The following is illustrative, not mandatory:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Targeting {
    Nearest,
    Toughest,
}
```

Add a targeting property to the existing ability definition.

Configure the four existing special-case abilities to use `Toughest`.

Other abilities should use `Nearest` where that matches existing behaviour.

Replace the spell-name check in the casting code with a check against the configured targeting strategy.

Use the simplest approach that fits the current Rust structures.

## 5. Strict Scope

Expected primary file:

`crates/benilla-app/src/survivors/abilities.rs`

A focused existing test file may also be changed if required.

Allowed changes:

- A small targeting enum or equivalent representation.
- A targeting property in the existing definition.
- Updates to affected ability definitions.
- Replacement of the existing spell-name targeting condition.
- Focused tests.

Not allowed:

- Rewriting the casting system.
- Changing other ability mechanics.
- Moving ability definitions into external files.
- Introducing new crates or dependencies.
- Refactoring unrelated modules.
- Renaming existing ability identifiers.
- Reformatting unrelated files.
- Adding unused targeting strategies.
- Changing UI or graphics.
- Changing the game's balance intentionally.

If the task requires a larger change than expected, stop and report the reason before proceeding.

## 6. Required Inspection Before Editing

Before making any changes:

1. Locate the ability definition structure.
2. Identify how ability definitions are constructed.
3. Find all references to the ability definition.
4. Confirm the exact targeting condition in the casting logic.
5. Identify the four affected ability definitions.
6. Check whether existing tests cover targeting behaviour.

Report findings before editing.

Do not assume the proposed structure matches the current code.

## 7. Acceptance Criteria

- [ ] Targeting strategy is explicitly represented in ability configuration.
- [ ] The four special-case abilities retain toughest-target selection.
- [ ] All other relevant abilities retain nearest-target selection.
- [ ] The identified hardcoded spell-name check is removed.
- [ ] No gameplay parameters are changed unintentionally.
- [ ] No new dependency is introduced.
- [ ] Focused tests verify targeting configuration or selection.
- [ ] Existing Survivors tests pass.
- [ ] Relevant formatting and lint checks pass.
- [ ] Only task-related files are modified.

## 8. Verification

Run from the repository root in PowerShell.

### Inspect the diff

```powershell
git diff --check
git diff --stat
git diff -- crates/benilla-app/src/survivors/abilities.rs
```

Inspect any changed test files separately.

### Run project validation

```powershell
.\scripts\windows\survivors.ps1 -Action Lint
```

Run the relevant Survivors tests using the repository's established test workflow.

If the exact test command is unclear, inspect the existing script or documentation rather than guessing.

### Review

Confirm:

- All existing ability definitions compile.
- The intended four abilities use toughest targeting.
- Default targeting is unchanged.
- No unrelated files were modified.
- No unrequested architectural changes were introduced.

Do not commit automatically.

## 9. Completion Procedure

After the implementation has been reviewed and verified:

1. Commit the focused code change.
2. Record the completed task in `docs/tasks/completed.md`.
3. Update `docs/architecture/content-system.md` to reflect the actual implementation.
4. Update `docs/PROJECT_STATUS.md` if the verified project state changes.
5. Select the next small Phase 2 task.

Do not begin another implementation task automatically.

## 10. Success Definition

**The task is successful when adding another ability that targets the toughest enemy requires changing its definition, not adding its name to the casting logic.**

Existing gameplay must remain unchanged.