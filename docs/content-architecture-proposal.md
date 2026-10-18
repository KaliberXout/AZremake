# Content architecture proposal (proposal only, not implemented in Phase 1)

Status: historical Phase 1 proposal; not an approved Phase 2 implementation plan. This document
records the original source audit and design options as they stood during Phase 1. Statements
about the then-current code were checked at the time, but may no longer describe HEAD.

**Current guidance:** See `docs/architecture/content-system.md`, `docs/roadmap.md`, and
`docs/tasks/next.md` for the current Phase 2 direction and authorised next steps. P2-001
(explicit ability targeting) was implemented in commit `0ad5a48` using Rust metadata; it did
not introduce RON/TOML loading or the composable selector/effect model proposed below.
External storage formats and a larger migration remain undecided.

## 1. How content is represented today

All survivors content is compile-time Rust data in `crates/benilla-app/src/survivors/`.

| Content | Where | Representation | Notes |
| --- | --- | --- | --- |
| Abilities | `data.rs` `ABILITIES` (73 entries) | `AbilityDef` built with `ability!`/`ab()` const constructors | Keyed by a `&'static str` (`key`), looked up by `data::ability(key)`, which panics on an unknown key. Each ability carries a `Spell.dbc` id used for icon, name and visuals. |
| Ability kinds | `data.rs` `Kind` | Closed enum, 16 variants (`Bolt`, `Strike`, `Melee`, `Nova`, `Cone`, `Ground`, `GroundSelf`, `Chain`, `Dot`, `Aura`, `Orbit`, `Shield`, `Immune`, `Thorns`, `Heal`, `Totem`) | Each kind has bespoke code in `abilities.rs`. |
| Effects | `data.rs` `Effect` | Closed enum, 8 variants, one per ability (`Slow`, `Root`, `Stun`, `Fear`, `Weaken`, `Drain`, `Execute`, `None`) | An ability has one effect slot, so "slow and burn" needs two abilities or a code change. |
| Passives | `data.rs` `PASSIVES` | `PassiveDef { key, name, spell, stat: Stat, per_level }` | `Stat` is a closed enum of 10 values, and `Run::stats()` folds them with a `match`. |
| Classes | `data.rs` `CLASSES` | `ClassDef { id, name, color, power, start, pool, looks, weapons, sheath }` | `id` is the WoW class number, and the pool is a list of ability keys. |
| Class-specific rules | scattered | `match self.class.id { 1 \| 2 => +40 HP, 7 \| 11 \| 3 \| 4 => +15 HP, _ => +0.5 speed }` in `mod.rs`; `mobility::MOBILITY` keyed by class id; `pyroblast \| aimed_shot \| exorcism \| starfire` string match in `abilities.rs` for "toughest target" selection; `"lightning_shield"` and `"renew"` string matches in `hero.rs` for visuals | Rules that are data in principle are code in practice. |
| Enemies and bosses | `data.rs` `ENEMIES`, `BOSSES`, `MAPS[].foes`, `MAPS[].bosses` | `EnemyDef` and `Foe` structs. Rosters are generated at runtime by `ladder()` from the Barrens' tuned table. | The Barrens uses hand-tuned values; the other 15 maps use the ladder. |
| Upgrades and cards | `hud.rs` `choices_for`, `cards.rs` `rarity` | `Choice` enum (5 variants), weights hard-coded in `choices_for` (1.2, 1.0, 0.8, 0.5), rarity thresholds hard-coded in `cards.rs` (0.66, 0.33) | Weights and thresholds are tuning data stuck in code. |
| Level curve and wave clock | `data.rs` | `xp_to_next`, `wave_pool`, `BOSS_EVERY_SECS`, `VICTORY_SECS` | Good candidates for data. |
| Mobility | `mobility.rs` | `Move` enum (5 variants) and a `MOBILITY` table | Same shape as abilities, with its own enum. |

Consequences the proposal has to fix:

1. Adding a spell usually means a new `Kind` or `Effect` variant, plus new branches in
   `abilities.rs`. Content changes need recompiles and code review.
2. A spell's behaviour, cooldown rules and balance numbers live in the same literal, so a
   balance pass touches code.
3. Panics on an unknown key make a typo a runtime crash in the menu, not a reported error.
4. Class identity is spread over numbers (`class.id`), strings (`key`) and tables, which makes a
   new class or a specialisation a code change in several files.

## 2. Requirements (from the phase brief)

Unique content identifiers; external content definitions; schema validation with helpful errors;
damage types and spell schools; reusable targeting and effect behaviours; buffs, debuffs, periodic
effects and stacking rules; cooldowns, charges and triggered abilities; summons, pets and
persistent area effects; talent and equipment modifiers; ability evolutions and cross-class
synergies; visual and audio references with fallbacks; automated content validation tests.

## 3. Format comparison

| Criterion | RON | TOML | JSON |
| --- | --- | --- | --- |
| Rust enums and tagged variants | **Native.** `Kind::Bolt`, `Slow(0.5, 3.0)` read as written. | Awkward. Enums become tables or strings, and tuple variants need workarounds. | Awkward. Needs an external tag convention. |
| Comments | Yes | Yes | No |
| Trailing commas and nesting | Yes, and readable for nested effect lists | Limited for deep nesting; arrays of tables get verbose | Verbose, no comments |
| Serde support | `ron` crate, mature | `toml` crate, mature | `serde_json`, mature |
| Editor tooling | Good in Rust-aware editors; no schema standard | Good; a JSON Schema can be applied via taplo | **Best.** JSON Schema is the standard for editor completion and validation. |
| Human editing by non-programmers | Moderate; Rust-like syntax | **Best** for flat tables | Moderate; strict syntax causes friction |
| Diff quality | Good | Good | Good |
| Fit for the current content | Matches the closed-enum design and the `&'static` keys | Fits flat stats; struggles with effect lists | Works, but loses the enum shape |

**Recommendation: RON for behaviour, TOML for flat tuning.** Abilities, passives, effects,
targeting and triggers are trees of enums, which RON expresses directly. Flat numeric tuning
(XP curve, wave clock, rarity thresholds, card weights) fits TOML and can be edited by people who
do not read Rust. JSON is not recommended for the core format, but a JSON Schema generated from
the same definitions is the best way to give editors completion. Generating it is a later step.

## 4. Proposed model

### 4.1 Identifiers

- Every content object has a stable string id in `snake_case`, `ns:name`-style namespaces are
  optional. Example: `mage.frostbolt`, `passive.might`, `enemy.elwynn.young_wolf`.
- IDs are unique per type, checked at load. The id is the key that saves, saved references and
  evolutions use. A `Spell.dbc` id is an optional visual reference, never the identity.
- Numeric ids from WoW (`spell`, `display`, class id) stay as data fields, documented as "the
  client row this reads", and are never used as keys.

### 4.2 Definitions in files

```ron
// content/abilities/mage.ron (illustrative, not implemented)
(
    id: "mage.frostbolt",
    school: Frost,
    damage_type: Magic,
    kind: Bolt( count: 1, range: 20.0, speed_from_spell: true ),
    cooldown: 2.0,
    damage: 8.0,
    effects: [
        Slow( factor: 0.5, secs: 3.0, stacking: RefreshLonger ),
    ],
    visuals: ( spell: 116, mark: 0 ),
    fallbacks: ( icon: "frostbolt", cast_sound: Some("spell.cast.frost") ),
)
```

Flat tuning, for example `content/tuning/curve.toml`:

```toml
[xp]
base = 5.0
step = 10.0
step_after_20 = 13.0
```

### 4.3 Damage types and spell schools

- `School` (Arcane, Fire, Frost, Nature, Holy, Shadow, Physical, ...) is data, not code, and is
  used for display, for passives that affect a school, and for synergy rules.
- `DamageType` is separate from school. Phase 1 needs one type (direct damage), so the proposal
  reserves the field without changing behaviour until a mitigation rule needs it.

### 4.4 Targeting and effect behaviours

Replace the closed `Kind` with composable parts:

- `Selector`: `Nearest(n)`, `Toughest(n)`, `Densest(radius)`, `Self_`, `AllInRadius(r)`.
- `Shape`: `Point`, `Cone(angle, range)`, `Circle(radius)`, `Line(length, width)`.
- `Delivery`: `Instant`, `Projectile(speed)`, `Ground(duration, tick)`, `Aura(interval)`.
- `Effect`: a list, not one slot. Each entry is damage, heal, a status (see 4.5), a summon, or a
  trigger.

The current `Kind` values map onto these combinations, so the migration is a translation table,
not a behaviour change. For example `Bolt` is `Nearest(count) + Projectile(speed) + Damage`.

### 4.5 Buffs, debuffs, periodic effects and stacking

- `Status { id, duration, tick: Option<f32>, stacking, max_stacks, dispel_group }`.
- `Stacking`: `RefreshDuration`, `RefreshLonger`, `AddStacks(max)`, `Independent`. The current
  DOT list (cap of 6 entries, oldest removed) becomes `AddStacks(6)` with an explicit rule.
- Periodic damage is a status with a tick. Current `Dot` and `Dot` ticks become that.
- Slow, root, stun, fear and weaken become statuses with an explicit boss modifier, instead of the
  constants written inline in `apply_hits` (`m.max(0.75)`, `t * 0.3`).

### 4.6 Cooldowns, charges and triggers

- `Cooldown { secs, scaling: Rank | Haste }`, and `Charges { max, recharge }`.
- `Trigger`: `OnHit`, `OnKill`, `OnLevelUp`, `OnCrit`, `OnTimer(secs)` with a `Chance`. Triggered
  abilities cast another ability by id, which keeps the cast path shared.
- The current cooldown function (`base * 0.93^(rank-1) * max(1-haste, 0.35)`, floor 0.2 s) stays as
  the default formula, and becomes a named rule that data can override per ability.

### 4.7 Summons, pets and persistent areas

- `Summon { unit: "enemy.wolf" | "summon.totem.frost", count, lifetime, follow: Hero | Point }`.
- `Area { shape, duration, tick, statuses }`. The current `GroundArea` and `Totem` components are
  the runtime form of this.
- These are data over existing ECS components, so no new runtime model is needed.

### 4.8 Talent and equipment modifiers

- A `Modifier` is a typed change to a stat or an ability field, for example
  `AbilityField { id: "mage.frostbolt", field: Damage, op: Mul(1.2) }`.
- Passives and talents are modifiers. Equipment is a set of modifiers and grants.
- `Run::stats()` becomes a fold over modifiers. The current `Stat` enum is the first set of
  allowed targets.
- Ordering is explicit: additive, then multiplicative, then overrides. This makes the current
  mixed `+=` and `*=` in `Run::stats()` a documented rule.

### 4.9 Evolutions and cross-class synergies

- `Evolution { from: "mage.frostbolt", requires: [Rank(8), Passive("mage.intellect", 3)],
  becomes: "mage.frost_nova_bolt" }`. The evolved ability is a normal ability with its own id.
- Synergies are modifiers with a `requires` clause that names two ids. They are checked at offer
  time, so `choices_for` can offer a synergy card with a clear reason.

### 4.10 Visual and audio references with fallbacks

- A reference is a list: `icon: ["spell:116", "fallback:frostbolt.png"]`. The loader tries each
  entry, and if all fail it records a warning and uses a built-in placeholder.
- The current code uses `Spell.dbc` ids for icons and visual kits, and `GameObjectDisplayInfo` ids
  for pickups. Those become typed references, validated against the loaded client data when it is
  present and skipped (with a logged placeholder) when it is not.
- Audio follows the same rule. A missing sound is a warning, not a crash.

### 4.11 Validation and errors

- Loading happens once, at start-up, and produces a `ContentReport` with file, line, column, id and
  message. The game refuses to start on errors and shows the report.
- Checks: unique ids; every referenced id exists; every number is in range (`cooldown > 0`,
  `count >= 1`, `0 <= rank <= max`); every class pool entry resolves; every class has a start
  ability that is in its own pool; every map has six bosses; roster ladders are non-decreasing
  (the property `ladder_rosters_never_soften_as_they_rise` already tests).
- Phase 1 already has the first of these as Rust unit tests (`data` invariants in
  `survivors/tests.rs` at the time; Survivors tests have since been split into
  `survivors/tests/`). These tests could inform future validation if external data is adopted.

### 4.12 Automated content tests

- A test per content file that loads it, validates it, and prints the report on failure.
- A golden test per class: the offer for a fixed seed does not change unless the content changes.
  This uses the seeded `Rng` introduced in Phase 1.

## 5. Migration plan (incremental)

The following is the original proposed sequence, not a currently approved task list. None
was started in Phase 1; P2-001 later took a smaller Rust-metadata-first approach. Any further
migration requires fresh review against the current source and Phase 2 plan.

1. **Freeze the baseline.** Phase 1 tests cover the rules that the migration must keep. Add
   golden offers and damage numbers for one class before anything moves.
2. **Move tuning first.** Extract the XP curve, wave clock, rarity thresholds and card weights to
   a TOML file loaded at start-up, with the current values as defaults. Lowest risk, and it
   validates the loader and error reporting.
3. **Move class and passive tables.** Classes and passives become RON files. Their Rust structs
   stay, filled by the loader. Ids replace `class.id` matches in `Run::stats()` and `mobility`.
4. **Move abilities behind a translation layer.** Each existing `Kind` is expressed as
   selector + shape + delivery. Abilities are loaded from RON, and the kind-specific code in
   `abilities.rs` is unchanged until every ability is translated. The golden tests prove the
   numbers match.
5. **Move enemies and rosters.** The Barrens' tuned table and the ladder inputs become data. The
   ladder stays code, because it is an algorithm.
6. **Introduce statuses and modifiers** for the effects currently inline in `apply_hits` and
   `Run::stats()`.
7. **Add new content** only after 1 to 6 are merged. This is Phase 2 or later.

Rules for the migration:

- No step changes a gameplay number unless a test shows a demonstrated defect.
- Every step adds tests for the behaviour it moves.
- The Rust structs stay the runtime model. The loader is the only new component.

## 6. What this proposal does not decide

- The final file layout. Proposed: `content/` at the repository root, not inside `crates/`, so a
  designer does not need a Rust toolchain.
- Whether the content loader lives in `benilla-app` or a new crate. A new crate avoids pulling
  Bevy into the validator, but adds a crate to the workspace.
- Whether RON or TOML wins for abilities. The recommendation above is a starting point to test in
  step 2, before the larger migration.

## 7. Original proposed Phase 2 priorities (historical)

These priorities were drafted before the Phase 1 baseline was completed and before P2-001.
The Windows smoke baseline subsequently passed its 17 automated checks; manual visual checks
remain open. Do not treat the following list as current work authorisation.

1. Finish the Windows build and smoke baseline (see `docs/survivors-smoke-test.md`).
2. Move tuning to TOML with the loader and `ContentReport` (migration step 2).
3. Add golden tests for one class (migration step 1).
4. Then the class and passive tables.
