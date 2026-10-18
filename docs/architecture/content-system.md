# Azeroth Survivors — Content System Architecture

**Status:** Proposed architecture — Phase 2  
**Last updated:** 2026-10-18
**Implementation state:** P2-001 (explicit ability targeting) is implemented and committed (`0ad5a48`). The broader architecture described here remains proposed; registration, validation, and external content storage have not been established by that milestone.

## 1. Purpose

This document describes the intended architecture for defining, registering, validating, and executing gameplay content in Azeroth Survivors.

The objective is to make it possible to add new abilities, classes, enemies, bosses, equipment, talents, and evolutions without repeatedly changing unrelated engine code.

The system must support incremental development and preserve existing gameplay behaviour during migration.

**Core principle: Content describes what should happen. Runtime systems determine how it happens.**

This document is a design direction, not permission to perform a large refactor.

---

## 2. Architectural Goals

### Primary goals

1. Define content using consistent, reusable structures.
2. Keep individual spell and item identifiers out of generic execution logic wherever practical.
3. Separate static definitions from per-run state.
4. Allow gameplay systems to query content by stable identifiers.
5. Validate content before gameplay depends on it.
6. Make content additions easy to test.
7. Support future progression systems without coupling them tightly to combat execution.
8. Preserve native Windows performance.
9. Integrate with existing Rust, Bevy, and Benilla conventions.

### Non-goals for Phase 2

- Rewriting the entire combat system.
- Replacing all existing content definitions at once.
- Implementing every possible WoW spell mechanic.
- Building a visual content editor.
- Implementing multiplayer networking.
- Creating a general-purpose scripting language.
- Introducing an external database.
- Requiring external JSON/RON/TOML files immediately.

Avoid infrastructure that does not solve an existing or clearly anticipated problem.

---

## 3. High-Level Architecture

The intended system consists of five conceptual layers.

```text
┌───────────────────────────────────────────┐
│           CONTENT DEFINITIONS             │
│                                           │
│ Abilities / Classes / Enemies / Items     │
│ Talents / Evolutions / Maps / Bosses      │
└─────────────────────┬─────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────┐
│         VALIDATION & REGISTRATION         │
│                                           │
│ IDs / References / Parameters / Assets    │
│ Content Registry / Lookup                 │
└─────────────────────┬─────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────┐
│            RUNTIME GAMEPLAY               │
│                                           │
│ Casting / Targeting / Damage / Effects    │
│ AI / Spawning / Movement / Collisions     │
└─────────────────────┬─────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────┐
│              RUN STATE                    │
│                                           │
│ Owned Abilities / Levels / Cooldowns      │
│ Equipment / Talents / Temporary Effects   │
└─────────────────────┬─────────────────────┘
                      │
                      ▼
┌───────────────────────────────────────────┐
│         PRESENTATION & FEEDBACK           │
│                                           │
│ HUD / Icons / Animations / Audio / VFX    │
└───────────────────────────────────────────┘
```

This diagram represents responsibilities, not necessarily five separate Rust crates or modules.

In practice, gameplay and run-state systems will interact in both directions. The diagram is a conceptual overview rather than an execution sequence.

---

## 4. Existing Architecture

### Known implementation

Survivors-specific code is primarily located under:

`crates/benilla-app/src/survivors/`

Known relevant modules:

| Module         | Known responsibility                                                                                                |
| -------------- | ------------------------------------------------------------------------------------------------------------------- |
| `abilities.rs` | Ability definitions, casting, targeting, and effect execution                                                       |
| `data.rs`      | Gameplay data and supporting definitions                                                                            |
| `hero.rs`      | Hero-related behaviour and run outcomes                                                                             |
| `hud.rs`       | HUD and level-up choices                                                                                            |
| `tests/`       | Survivors-specific test modules (`mod.rs`, `abilities.rs`, `hero_tests.rs`, `rng.rs` at the last reviewed snapshot) |

This list is not exhaustive and must be verified against the current source when implementing changes.

### Existing ability behaviours

The current system supports multiple ability categories, including:

- Projectiles and direct attacks.
- Melee attacks.
- Area-of-effect attacks.
- Cone attacks.
- Persistent ground effects.
- Chained attacks.
- Damage-over-time effects.
- Auras.
- Orbiting effects.
- Shields and defensive effects.
- Healing.
- Totems.

Some of these behaviours may share internal execution paths.

### Existing hardcoding

The initial spell-name-specific targeting check was replaced in P2-001 (`0ad5a48`). `AbilityDef` now carries a `Targeting` value; `Nearest` is the default and `Toughest` is configured for `pyroblast`, `aimed_shot`, `exorcism`, and `starfire`. The casting path reads this configuration rather than matching those spell identifiers.

This is one completed migration, not evidence that all identifier-dependent gameplay logic has been removed. Inspect the current source before identifying another candidate.

---

## 5. Content Identity

Every independently referenced content definition should have a stable identifier.

Examples:

```text
ability:fireball
ability:chain_lightning
ability:consecration

class:mage
class:paladin

enemy:ghoul
enemy:scarlet_crusader

item:arcanite_reaper

talent:improved_fireball

map:elwynn_forest
```

These are illustrative identifiers, not a required naming convention or existing content inventory.

### Identifier requirements

- Unique within the relevant content domain.
- Stable across balancing changes.
- Independent of display names.
- Suitable for saving and loading.
- Easy to reference from other definitions.
- Validated for duplicates and missing references.

### Initial implementation guidance

Do not immediately replace existing string identifiers with a complex ID framework.

Existing identifiers may be sufficient for Phase 2.

A dedicated identifier type should be introduced only when it provides a clear advantage, such as preventing domain confusion or supporting reliable serialization.

---

## 6. Ability Definitions

Abilities should describe their intended behaviour without containing the execution logic itself.

### Conceptual ability properties

| Property            | Purpose                                     |
| ------------------- | ------------------------------------------- |
| Identifier          | Stable reference                            |
| Display name        | UI text                                     |
| Class association   | Optional class relationship                 |
| Behaviour kind      | Projectile, melee, area, aura, etc.         |
| Targeting strategy  | How targets are selected                    |
| Damage parameters   | Base damage and scaling                     |
| Timing              | Cooldown, duration, tick interval           |
| Geometry            | Range, radius, cone angle, projectile count |
| Effects             | Damage, healing, shields, status effects    |
| Visual references   | Icons, animations, VFX                      |
| Audio references    | Spell sounds                                |
| Upgrade information | Future level scaling and evolution hooks    |

Not every ability needs every property.

### Design requirement

The engine should not need to recognise a particular spell name to determine behaviour that can be expressed through reusable configuration.

For example:

**Undesirable**

```rust
if ability.key == "pyroblast" {
    // Select a tougher enemy.
}
```

**Preferred direction**

```rust
match ability.targeting {
    Targeting::Nearest => { /* ... */ }
    Targeting::Toughest => { /* ... */ }
}
```

This example reflects the implemented P2-001 direction. Check the current source for exact types and call sites before extending it.

### Avoid excessive generalisation

Not every ability needs to fit into a universal collection of arbitrary effects.

If a spell introduces a genuinely new mechanic, adding a new runtime behaviour may be appropriate.

The goal is to avoid duplicating existing mechanics, not to eliminate all special-purpose code.

---

## 7. Targeting Architecture

**First Phase 2 implementation milestone: completed in commit `0ad5a48`.**

Target selection should be explicit rather than inferred from a spell identifier.

### Initial targeting strategies

Only implement strategies required by existing abilities.

Implemented initial strategies:

- `Nearest`
- `Toughest`

Potential future strategies:

- Random valid enemy.
- Lowest-health enemy.
- Highest-health enemy.
- Nearest group.
- Directional target.
- Player-centred area.
- Ground position.

These future strategies are design possibilities, not an implementation checklist.

### Important distinction

**Target selection and ability geometry are separate concerns.**

For example:

- Targeting determines which enemy or location an ability aims at.
- Geometry determines which entities are affected after the ability executes.

A cone spell might select a direction using one enemy, then affect multiple enemies inside that cone.

An area spell might centre on the player without selecting an individual enemy.

Do not force every ability into an enemy-targeting model.

### Completed P2-001 migration approach

The following steps describe the completed targeting migration and remain a useful pattern for future incremental changes:

1. Confirm existing targeting rules.
2. Identify which ability definitions need special targeting.
3. Add the smallest suitable targeting configuration.
4. Preserve current default behaviour.
5. Replace the hardcoded identifier check.
6. Add tests for ordinary and special targeting.
7. Verify that existing abilities behave identically.

Do not add unused targeting strategies merely to anticipate future content.

---

## 8. Runtime Execution

The runtime is responsible for interpreting content definitions and applying gameplay behaviour.

### Runtime responsibilities

- Check whether an ability can activate.
- Resolve effective ability parameters.
- Select targets.
- Execute the appropriate behaviour.
- Apply damage, healing, or status effects.
- Create and update relevant entities.
- Handle cooldowns and durations.
- Emit gameplay feedback where appropriate.

### Separation of responsibilities

**Definitions contain configuration.**

**Runtime systems contain execution logic.**

**Run state contains mutable values.**

For example, a Fireball definition might contain its base damage, targeting rule, and projectile parameters.

The runtime determines which enemy is targeted and how the projectile behaves.

The run state records the ability's current level, cooldown, and relevant modifiers.

### Performance considerations

Avoid designs that require:

- Repeated parsing of external files during combat.
- Frequent heap allocations in hot loops.
- Repeated string comparisons for every enemy.
- Expensive global content lookups per collision.
- Unnecessary duplication of immutable definitions.

Prefer loading and validating definitions before gameplay, then using efficient runtime references or identifiers.

Any optimisation should be informed by profiling rather than speculation.

---

## 9. Static Definitions Versus Run State

Static content and mutable gameplay state must remain conceptually distinct.

### Static definition example

```text
Fireball
  Base damage: 20
  Cooldown: 2.0 seconds
  Targeting: Nearest
  Behaviour: Projectile
```

### Run state example

```text
Owned Fireball
  Level: 4
  Remaining cooldown: 0.8 seconds
  Damage modifier: +25%
  Projectile count bonus: +1
```

These are conceptual examples, not confirmed existing game values.

### Benefits

This separation allows:

- Multiple runs to use the same definitions.
- Upgrades without mutating global content.
- Clear save-data boundaries.
- More reliable tests.
- Easier future class and equipment modifiers.
- Potential future multiplayer support without designing multiplayer now.

---

## 10. Gameplay Modifiers

Future abilities, talents, equipment, buffs, and upgrades will need to modify gameplay values.

Avoid introducing separate, incompatible modifier implementations for each system.

### Potential modifier sources

- Ability level.
- Class passive.
- Talent.
- Equipment.
- Temporary buff.
- Evolution.
- Map or challenge modifier.

### Potential modified properties

- Damage.
- Cooldown.
- Area.
- Range.
- Duration.
- Projectile count.
- Critical chance.
- Critical damage.
- Defensive values.
- Healing.

### Design questions to resolve later

- Which modifiers stack additively?
- Which stack multiplicatively?
- In what order are modifiers applied?
- Are there minimum and maximum values?
- How are integer values rounded?
- How are temporary modifiers removed?
- Can modifiers affect targeting or behaviour type?

These rules must be explicit before implementing a general modifier engine.

**Phase 2 should not introduce a comprehensive modifier framework prematurely.**

The completed targeting migration did not require one.

---

## 11. Classes

Class definitions should describe class identity and available gameplay content.

Potential properties:

- Stable identifier.
- Display name.
- Starting statistics.
- Starting ability or abilities.
- Available ability pool.
- Passive traits.
- Visual and audio references.
- Unlock requirements.

### Separation from abilities

Abilities should not automatically be restricted to a class merely because they are thematically associated with it.

A class may define which abilities it can access, while an ability definition describes how the ability behaves.

This allows flexibility for future game modes, talents, equipment, and special unlocks.

### Future considerations

- Class-specific ability upgrades.
- Passive abilities.
- Resource systems.
- Shapeshifting.
- Pets and summons.
- Specialisations.

Implement these only when a concrete class design requires them.

---

## 12. Enemies and Bosses

Enemy definitions should describe combat characteristics and behaviour configuration.

Potential properties:

- Stable identifier.
- Display name.
- Base statistics.
- Movement behaviour.
- Attack behaviour.
- Spawn conditions.
- Rewards.
- Asset references.

### Bosses

Bosses may require additional behaviour configuration:

- Attack patterns.
- Phases.
- Summons.
- Telegraphs.
- Arena effects.
- Phase transitions.

The architecture should support reusable behaviour patterns without assuming every boss can be described entirely through generic data.

Complex boss mechanics may require dedicated runtime implementations.

---

## 13. Equipment and Talents

Equipment and talents should eventually reference reusable gameplay effects rather than modifying combat code through item-name checks.

### Equipment definitions

Potential properties:

- Identifier.
- Slot.
- Rarity.
- Statistics.
- Special effects.
- Restrictions.
- Icon and description.

### Talent definitions

Potential properties:

- Identifier.
- Prerequisites.
- Rank limits.
- Effects.
- Class association.
- Unlock requirements.

### Architectural requirement

A talent that increases Fireball damage should modify Fireball's effective parameters through an established mechanism.

It should not require a new hardcoded Fireball condition in the damage execution system.

Exceptions may be appropriate for genuinely unique mechanics.

---

## 14. Ability Evolutions

Evolutions should be represented as explicit relationships between content definitions.

Conceptual example:

```text
Base ability
    +
Required upgrade or item
    +
Required ability level
    ↓
Evolved ability
```

### Evolution properties

Potential requirements:

- Source ability.
- Resulting ability.
- Minimum level.
- Additional prerequisites.
- Whether the original ability is replaced.
- Whether upgrades carry over.
- Whether an evolution can occur more than once.

### Design principle

An evolution may change behaviour, not merely increase statistics.

This might involve changing the ability's behaviour kind, effect configuration, or underlying definition.

The exact representation will be designed during Phase 4.

---

## 15. Content Registration and Validation

A centralised registry is a possible direction for managing definitions.

The precise implementation is undecided.

### Potential responsibilities

- Register definitions.
- Detect duplicate identifiers.
- Resolve references.
- Report missing dependencies.
- Validate parameter ranges.
- Provide content lookup.
- Expose definitions to gameplay and UI systems.

### Validation examples

Reject or report:

- Duplicate content identifiers.
- Missing referenced abilities.
- Negative cooldowns where unsupported.
- Invalid radius or range values.
- Invalid upgrade prerequisites.
- Circular dependencies where prohibited.
- Missing required assets.

Not every missing optional asset should necessarily prevent startup.

### Error handling

Development errors should be informative.

For example:

```text
Invalid ability 'chain_lightning':
maximum target count must be greater than zero.
```

Avoid silent fallback behaviour that hides malformed content.

### Initial implementation guidance

Start with validation that protects actual current gameplay assumptions.

Do not build an elaborate validation framework before identifying the relevant invariants.

---

## 16. Content Storage Format

The long-term content format has not been selected.

### Option A — Rust definitions

Advantages:

- Strong type checking.
- Simple integration.
- No new file-loading pipeline.
- Easy refactoring.

Disadvantages:

- Content changes require recompilation.
- Less accessible to non-programmers.
- Potentially large definition files.

### Option B — External data files

Possible formats include RON, JSON, or TOML.

Advantages:

- Easier content authoring.
- Definitions can be organised independently.
- Potential support for modding.
- Balance changes may not require recompilation.

Disadvantages:

- Requires parsing and validation.
- Reference errors are discovered at runtime or during validation.
- Asset-loading and packaging complexity.
- Version compatibility concerns.

### Option C — Hybrid

Use Rust for behaviour implementations and typed schemas, with external files for content instances.

This may eventually offer a useful balance between extensibility and safety.

### Current decision

**Deferred.**

Phase 2 begins by improving the existing representation rather than introducing external files immediately.

The first targeting migration is complete. Revisit this decision only after reviewing actual content-authoring requirements and the current source; no external format has been approved.

---

## 17. Asset References

The content system must eventually reference visual and audio assets consistently.

Potential references include:

- Spell icons.
- Character models.
- Creature models.
- Spell effects.
- Animations.
- Sounds.
- Environment assets.

### Requirements

- Keep content identifiers separate from asset paths.
- Avoid duplicating asset references across unrelated systems.
- Provide useful diagnostics for missing assets.
- Respect existing Benilla asset-loading mechanisms.
- Avoid loading assets repeatedly during combat.

### Legal and repository boundary

World of Warcraft client assets are external dependencies and must not be committed to the repository.

Development and distribution must account for the rights and licensing associated with those assets.

---

## 18. Testing Strategy

Content-system changes should be tested at multiple levels.

### Definition tests

Verify that:

- Identifiers are valid.
- Required values are present.
- References resolve.
- Invalid values are rejected.

### Behaviour tests

Verify that:

- Targeting rules select the intended targets.
- Ability execution uses the correct parameters.
- Upgrades modify values as expected.
- Existing abilities preserve their previous behaviour.

### Integration tests

Verify that:

- Content definitions are available during gameplay.
- Level-up choices reference valid abilities.
- Gameplay can complete a full run.
- No unexpected regressions occur.

### Performance tests

Compare changes against the Phase 1 baseline where practical.

Avoid adding expensive work to frequently executed combat paths.

### Test philosophy

Prefer focused deterministic tests for content logic.

Use full-run simulations and native smoke tests for broader regression detection.

---

## 19. Migration Strategy

**Incremental migration is mandatory.**

Do not replace the entire existing content system in one operation.

### Migration sequence

1. Identify one hardcoded behaviour.
2. Confirm its current semantics.
3. Introduce explicit configuration.
4. Preserve existing defaults.
5. Update the smallest number of call sites.
6. Add or adjust focused tests.
7. Run relevant validation.
8. Review the resulting code.
9. Commit the isolated change.
10. Continue to the next dependency.

### Compatibility requirements

During migration:

- Existing abilities should continue to work.
- Existing game balance should not change unintentionally.
- Existing tests should remain meaningful.
- Save-data compatibility must be considered if persistent data is affected.
- Public interfaces should not change without a reason.

### Rollback

Small changes should be independently revertible.

Avoid combining architecture changes with large batches of new content.

---

## 20. Architectural Decision Process

Before introducing a new abstraction, answer:

1. What current problem does it solve?
2. Which existing code will use it?
3. Will at least one real gameplay example exercise it?
4. Does it reduce duplication or coupling?
5. Is it simpler than the existing alternative?
6. Can it be tested independently?
7. Does it create unnecessary work for future content?
8. Does it fit existing Rust and Bevy patterns?

If these questions cannot be answered, defer the abstraction.

### Decision recording

Significant decisions should be recorded in this document or a dedicated architecture decision record.

Include:

- Decision.
- Reason.
- Alternatives considered.
- Consequences.
- Implementation status.

---

## 21. First Implementation Milestone — P2-001

### Explicit ability targeting

**Implementation status:** Implemented and committed as `0ad5a48` (`refactor(survivors): make ability targeting data-driven`).

**Delivered**

- Added `Targeting::{Nearest, Toughest}` metadata to ability definitions in `data.rs`.
- Kept `Nearest` as the default; configured `pyroblast`, `aimed_shot`, `exorcism`, and `starfire` to use `Toughest`.
- Updated casting in `abilities.rs` to use configured targeting rather than the four spell identifiers.
- Preserved the existing `nearest()` and `toughest()` selection algorithms.
- Added focused configuration regression tests.

**Verification recorded during implementation**

- `cargo check -p benilla-app --tests` succeeded.
- `cargo test -p benilla-app --lib targeting_` passed: 7 tests, 0 failed. The name filter included tests beyond the two new configuration tests.
- The complete test suite, native lint workflow, and every original acceptance criterion were **not** independently confirmed by those results; do not infer broader verification.

**Related follow-up work:** Subsequent commits `447bdfd`, `c87cd7c`, `755acb2`, and `c19ef97` extracted Survivors tests into modules. This improves test organisation but does not itself implement the proposed content registry or storage system.

See `docs/tasks/completed.md` for recorded completed tasks and `docs/tasks/next.md` for the current planning checkpoint. No further implementation milestone is approved by this section.

---

## 22. Future Architectural Questions

These questions remain open and should be answered when their associated work becomes relevant.

- Should definitions live in Rust, external files, or both?
- How should content identifiers be represented?
- Should class restrictions live on abilities or class definitions?
- How should modifier stacking work?
- How should status effects be represented?
- How should pets and summons be modelled?
- How should ability evolutions interact with upgrades?
- How should save files reference content versions?
- How should content be validated during development?
- How should visual effects be configured?
- How should unique boss behaviours be registered?
- Should community content packs be supported?
- What should be the policy for backwards compatibility?

Avoid resolving these prematurely.

---

## 23. Guiding Principle

**Adding a new Fireball-like spell should primarily involve defining content, not modifying the combat engine.**

Adding a genuinely new mechanic may require engine code, but that mechanic should become reusable by subsequent abilities.

The architecture succeeds when new gameplay content becomes easier to add, test, balance, and maintain — without sacrificing performance or the flexibility to implement unusual WoW abilities.
