# Azeroth Survivors — Development Roadmap

**Status:** Active  
**Current phase:** Phase 2 — Data-Driven Content System  
**Last updated:** 2026-10-18

## 1. Project Vision

Azeroth Survivors is a World of Warcraft-inspired survival roguelite built with Rust, Bevy, and the existing Benilla engine.

The objective is to create a highly replayable single-player game combining:

- The accessible, escalating combat of Vampire Survivors.
- Recognisable World of Warcraft classes, spells, enemies, and environments.
- Meaningful build choices and powerful ability combinations.
- Itemisation, talents, and progression inspired by WoW.
- Distinct maps, bosses, and challenges.
- A content architecture that allows new gameplay content to be added efficiently.

The game should be enjoyable as a standalone Survivors experience, even for players unfamiliar with World of Warcraft.

### Core design principles

1. **Gameplay first.** A fun, responsive combat loop takes priority over the quantity of available content.
2. **Build diversity.** Different classes and ability combinations should produce meaningfully different playstyles.
3. **Meaningful choices.** Level-ups, equipment, talents, and evolutions should influence how a run develops.
4. **Escalating spectacle.** Abilities should become more visually and mechanically impressive as a run progresses.
5. **Data-driven expansion.** Adding content should require as little new engine code as reasonably possible.
6. **Incremental development.** Build and verify small systems before expanding them.
7. **Native Windows first.** Browser compatibility and multiplayer must not delay core gameplay development.

---

## 2. Development Phases

| Phase | Focus | Status |
|---|---|---|
| 1 | Stability, testing, and development foundation | Complete |
| 2 | Data-driven content system | Current |
| 3 | Class identity and ability expansion | Planned |
| 4 | Ability upgrades, combinations, and evolutions | Planned |
| 5 | Equipment, talents, and build progression | Planned |
| 6 | Enemies, bosses, and map progression | Planned |
| 7 | Presentation, feedback, and polish | Planned |
| 8 | Balance, optimisation, and release preparation | Planned |

These phases describe the primary focus of development, not strict boundaries. Small supporting changes may occur earlier when necessary.

Major changes to scope or phase ordering should be documented before implementation.

---

## 3. Phase 1 — Stability and Testing

**Status: COMPLETE**

### Objectives

Establish a reliable, playable baseline and ensure future changes can be validated without relying exclusively on manual gameplay.

### Delivered

- Native Windows build workflow.
- Automated gameplay tests.
- Full 900-second headless simulation.
- Smoke testing for native gameplay.
- Performance measurement tools.
- CI validation.
- Additional edge-case tests.
- Development documentation.

### Completion criteria

- [x] Native Windows game builds and runs.
- [x] Core gameplay completes a full 15-minute run.
- [x] Victory and death paths are tested.
- [x] Survivors automated tests pass.
- [x] Native smoke tests pass.
- [x] GitHub Actions checks pass.
- [x] Phase 1 merged into `main`.

See `docs/PROJECT_STATUS.md` for the recorded test and performance results.

---

## 4. Phase 2 — Data-Driven Content System

**Status: CURRENT**

**Implemented milestone:** P2-001 (explicit ability targeting) was committed as `0ad5a48`. Subsequent commits `447bdfd`, `c87cd7c`, `755acb2`, and `c19ef97` reorganised Survivors tests into modules. These changes do not complete the broader Phase 2 architecture.

### Goal

Separate gameplay content definitions from the systems that execute them, making it easier to add abilities, enemies, classes, and other content without repeatedly modifying engine logic.

This is the foundation for subsequent gameplay expansion.

### 2.1 — Audit existing content

Identify where gameplay content is currently defined and how it reaches the runtime.

Initial areas to investigate:

- Ability definitions.
- Ability casting and targeting.
- Damage and status effects.
- Hero/class definitions.
- Enemy definitions and spawning.
- Upgrade and level-up choices.
- Boss definitions and progression.

Document the existing behaviour before changing it.

**Deliverable:** A concise map of current content ownership and hardcoded dependencies.

### 2.2 — Ability metadata

Introduce explicit configuration for behaviours that are currently determined by particular spell identifiers.

First implemented step: ability targeting strategy (`Targeting::Nearest` / `Targeting::Toughest`, P2-001).

For example:

- Nearest enemy.
- Toughest enemy.
- Random valid enemy.
- Area centred on the player.
- Directional targeting.

Not every targeting strategy needs to be implemented immediately.

**Deliverable:** Existing abilities retain their behaviour while targeting becomes explicitly configurable.

### 2.3 — Content definitions and validation

Establish a consistent representation for content definitions.

Potential requirements:

- Stable identifiers.
- Display names and descriptions.
- Gameplay parameters.
- Behaviour configuration.
- Asset references.
- Unlock requirements.
- Validation of missing or invalid references.

The implementation format is not yet decided.

Possible approaches include Rust-defined data, external configuration files, or a hybrid.

**Do not introduce a new serialization framework or content-loading system until its requirements are clear.**

### 2.4 — Content loading and registration

Provide a predictable mechanism for registering and accessing gameplay definitions.

Desired properties:

- Definitions can be looked up by stable identifier.
- Invalid or duplicate definitions produce useful errors.
- Gameplay code does not depend on scattered spell-name checks.
- New definitions can be added without modifying unrelated systems.

External file loading is desirable but is not automatically required for the first implementation.

### 2.5 — Prove the architecture

Add a small amount of new content using the resulting system.

The goal is to demonstrate that the architecture actually reduces implementation effort.

### Phase 2 completion criteria

- [ ] Existing content architecture documented.
- [x] Initial targeting hardcoding replaced with explicit configuration (P2-001, `0ad5a48`).
- [ ] Content-definition conventions established.
- [ ] Content validation covered by automated tests.
- [ ] At least one new ability added through the improved architecture.
- [ ] Existing gameplay and 15-minute simulation remain functional.
- [ ] Native tests, lint, and relevant CI checks pass.
- [ ] Architecture documentation updated.

**Out of scope:** Rewriting the entire combat engine, implementing every possible ability type, or creating a full visual content editor.

---

## 5. Phase 3 — Classes and Ability Expansion

**Status: PLANNED**

### Goal

Make playable classes feel mechanically distinct, with recognisable abilities and different approaches to survival.

### Class identity

Potential class archetypes include:

| Class | Potential gameplay identity |
|---|---|
| Warrior | Close-range combat, armour, retaliation, weapon attacks |
| Paladin | Holy damage, auras, shields, sustain |
| Hunter | Ranged attacks, traps, pets |
| Rogue | Fast attacks, poison, critical strikes |
| Priest | Holy and shadow magic, healing, defensive effects |
| Shaman | Elemental spells, totems, weapon enhancements |
| Mage | Powerful spells, area control, elemental effects |
| Warlock | Damage over time, curses, demons |
| Druid | Nature magic, shapeshifting, hybrid gameplay |

These are design directions, not commitments to implement all classes immediately.

### Ability expansion

Add recognisable abilities with distinct mechanics rather than creating large numbers of nearly identical attacks.

Potential examples:

- Chain Lightning jumping between enemies.
- Blizzard creating a persistent damaging area.
- Consecration controlling space around the player.
- Whirlwind rewarding close-range positioning.
- Corruption spreading damage over time.
- Multi-Shot attacking groups of enemies.
- Totems providing persistent effects.
- Pets acting as independent combat entities.

Existing abilities must be audited before any are described as new.

### Completion criteria

- [ ] A small representative set of classes has clearly different playstyles.
- [ ] New abilities use the content architecture.
- [ ] Each implemented class has viable offensive and defensive options.
- [ ] Class-specific mechanics have automated tests where practical.
- [ ] Full runs remain playable across the supported classes.

**Scope rule:** Develop a few strong class identities before attempting complete coverage of every WoW class.

---

## 6. Phase 4 — Ability Upgrades and Evolutions

**Status: PLANNED**

### Goal

Make the player's build transform meaningfully over the course of a run.

### Ability upgrades

Potential upgrade dimensions:

- Damage.
- Attack frequency.
- Projectile count.
- Range and area.
- Duration.
- Chain targets.
- Critical chance.
- Additional effects.

Upgrades should have clear limits and predictable stacking behaviour.

### Ability evolutions

Introduce major transformations that alter how abilities behave.

Examples for design exploration:

- A projectile spell evolving into a multi-projectile barrage.
- Chain Lightning evolving to jump between larger groups.
- A damage-over-time ability gaining the ability to spread.
- A defensive aura evolving into an offensive retaliation effect.
- A totem evolving into a more powerful persistent summon.

Evolution requirements could involve ability levels, complementary abilities, equipment, or other build conditions.

The exact system must be designed and tested before content production.

### Completion criteria

- [ ] Ability upgrades have consistent rules.
- [ ] Upgrade descriptions reflect actual behaviour.
- [ ] Evolution requirements are represented explicitly.
- [ ] Several abilities have meaningful evolutions.
- [ ] Evolution interactions are tested.
- [ ] Players can pursue different viable build paths.

---

## 7. Phase 5 — Equipment, Talents, and Build Progression

**Status: PLANNED**

### Goal

Introduce additional build-defining decisions inspired by World of Warcraft without overwhelming the fast-paced Survivors gameplay loop.

### Equipment

Potential systems:

- Weapons.
- Armour.
- Trinkets.
- Item rarity.
- Stat bonuses.
- Unique equipment effects.
- Equipment synergies with particular abilities or classes.

The number of equipment slots and how items are acquired remain undecided.

### Talents

Explore a simplified talent system that offers significant gameplay choices.

Potential examples:

- Increased critical damage.
- Improved damage-over-time effects.
- Stronger shields.
- Additional projectiles.
- Improved pet effectiveness.
- Ability-specific transformations.

Talents should complement, rather than duplicate, normal ability upgrades.

### Progression boundaries

Decide which systems apply:

- During an individual run.
- Between runs.
- Across permanent account/profile progression.

Permanent progression should enhance replayability without making early runs meaningless.

### Completion criteria

- [ ] Equipment rules and acquisition are defined.
- [ ] Items have meaningful gameplay effects.
- [ ] Talent choices create build diversity.
- [ ] Run-specific and persistent progression are clearly separated.
- [ ] Progression data can be validated and tested.
- [ ] Core combat remains readable and balanced.

---

## 8. Phase 6 — Enemies, Bosses, and Maps

**Status: PLANNED**

### Goal

Expand the game beyond a single survival experience into distinct encounters and environments.

### Enemy variety

Introduce enemy behaviours that require different player responses.

Potential archetypes:

- Fast but fragile enemies.
- Slow, heavily armoured enemies.
- Ranged attackers.
- Charging enemies.
- Summoners.
- Support or healing enemies.
- Enemies that leave hazardous ground effects.

### Boss encounters

Bosses should provide more than additional health and damage.

Potential mechanics:

- Telegraph attacks.
- Summoned reinforcements.
- Area denial.
- Movement phases.
- Distinct attack patterns.
- Escalating difficulty.

### Maps

Explore environments inspired by Azeroth, each with its own atmosphere and encounter design.

Potential examples:

- Elwynn Forest.
- Duskwood.
- The Barrens.
- Westfall.
- Stranglethorn Vale.
- Eastern Plaguelands.

These are candidates, not a confirmed production schedule.

### Map progression

Possible systems:

- Different enemy rosters.
- Unique bosses.
- Map-specific events.
- Unlockable stages.
- Increasing difficulty.
- Distinct environmental hazards.

### Completion criteria

- [ ] Multiple environments provide different gameplay experiences.
- [ ] Enemies exhibit meaningfully different behaviours.
- [ ] Bosses have recognisable mechanics.
- [ ] Map and encounter configuration is maintainable.
- [ ] Progression between maps is understandable.
- [ ] Performance remains acceptable at high enemy counts.

---

## 9. Phase 7 — Presentation and Gameplay Polish

**Status: PLANNED**

### Goal

Improve the clarity, impact, and overall feel of the game.

### Areas of focus

- Ability visual effects.
- Hit feedback.
- Damage numbers.
- Audio and spell effects.
- Enemy death effects.
- Level-up presentation.
- Ability and evolution descriptions.
- Class selection.
- Run summary screens.
- Menu and settings improvements.
- Accessibility and readability.

### Design requirements

- Combat must remain readable when many effects overlap.
- Important enemy attacks should be visually distinguishable.
- Players should understand why they received damage.
- Upgrade descriptions should accurately communicate gameplay changes.
- Visual effects should not cause unacceptable performance regressions.

### Completion criteria

- [ ] Core combat has satisfying feedback.
- [ ] Menus and upgrade choices are understandable.
- [ ] High-density combat remains readable.
- [ ] Audio and visual settings are usable.
- [ ] Presentation has been tested across representative gameplay scenarios.

---

## 10. Phase 8 — Balance, Optimisation, and Release Preparation

**Status: PLANNED**

### Goal

Turn the expanded game into a stable, cohesive experience suitable for distribution.

### Balance

- Compare class performance.
- Identify dominant or ineffective abilities.
- Review upgrade and evolution combinations.
- Tune enemy scaling.
- Review boss difficulty.
- Assess progression pacing.
- Evaluate replayability.

### Performance

Use the Phase 1 performance measurements as an initial reference.

Monitor:

- Frame times.
- Entity counts.
- Enemy counts.
- Memory usage.
- Ability and effect processing costs.
- Performance during late-run combat.

### Release preparation

- Reliable Windows builds.
- Appropriate handling of required game assets.
- Clear installation instructions.
- Save-data compatibility.
- Error reporting and diagnostics.
- Regression testing.
- Release packaging.

### Completion criteria

- [ ] Core content is feature-complete for the initial release scope.
- [ ] Representative classes and builds are viable.
- [ ] Full-run regression tests pass.
- [ ] Performance is acceptable.
- [ ] Installation and setup are documented.
- [ ] Release artefacts can be built reproducibly.

---

## 11. Deferred and Optional Features

These ideas are not part of the current development commitment.

### Multiplayer

Potential future cooperative play, possibly over local networks or VPN connections.

Multiplayer must not influence the initial content-system architecture unless doing so provides a clear benefit to single-player development.

### Browser support

Maintain existing WASM compatibility where practical, but prioritise native Windows functionality.

### Modding and content tools

Potential future features:

- External content packs.
- Community-defined abilities.
- Content validation tools.
- Visual ability editors.
- Custom maps and encounters.

These may become easier once the underlying content system is mature.

### Additional progression systems

Possible later additions:

- Achievements.
- Challenges.
- Difficulty modifiers.
- Unlockable characters.
- Special game modes.
- Additional permanent progression.

No implementation commitment is implied by inclusion in this section.

---

## 12. Development Workflow

### Branching

- `main` should remain stable.
- Use focused feature branches for new work.
- Keep unrelated changes in separate commits.
- Merge after appropriate review and testing.

### Task sizing

Prefer tasks that:

- Have a clear expected result.
- Affect a small number of files.
- Can be reviewed independently.
- Have specific verification steps.
- Preserve existing gameplay unless explicitly changed.

Avoid large, loosely defined AI coding requests.

### AI assistance

Use Codex for scoped source inspection and, only after scope is agreed, narrowly defined implementation tasks. Local assistants remain optional and experimental rather than the default task owner.

Review the actual local source and existing tests before assigning edits. Preserve unrelated working-tree changes; do not let an assistant stage, commit, push, reset, clean, or delete files without explicit approval.

All generated changes require diff review and appropriate testing by the developer.

### Documentation

Maintain the following files:

- `docs/PROJECT_STATUS.md` — current verified project state.
- `docs/roadmap.md` — planned development milestones.
- `docs/architecture/content-system.md` — architectural decisions.
- `docs/tasks/next.md` — immediate task or planning checkpoint.
- `docs/tasks/completed.md` — completed work and verification.

Update these documents when milestones, architecture, or implementation priorities change.

---

## 13. Current Priority

**Phase 2, next milestone: select a small, evidence-based content-system improvement.**

P2-001 targeting is implemented and committed. Before authorising another code change:

1. Inspect the current content-system architecture and relevant source files.
2. Account for the recently modularised Survivors tests and existing coverage.
3. Identify a specific remaining hardcoded dependency or content-definition problem.
4. Agree on one small task, its boundaries, and its verification steps.
5. Only then assign a scoped implementation task; review the diff and targeted test results before accepting it.

No new Phase 2 implementation task is approved by this roadmap update.

**The objective is not to write more code. It is to make adding the next hundred abilities easier than adding the first hundred.**