//! Unit tests for the mode's rules: the numbers behind the horde, the hero, the offers and the
//! run's end. Every random draw comes from a seeded [`Rng`], so a failure reproduces.

use bevy::prelude::*;

use super::data;
use super::hero::{self, Outcome};
use super::hud::{self, Choice, Offer};
use super::{Owned, Phase, Rng, Run, BASE_HP, BASE_MAGNET, BASE_SPEED};

const CASTER: u8 = 8; // Mage
const WARRIOR: u8 = 1;
const HUNTER: u8 = 3;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

fn class(id: u8) -> &'static data::ClassDef {
    data::CLASSES
        .iter()
        .find(|c| c.id == id)
        .expect("a class with that id")
}

fn run_for(id: u8) -> Run {
    Run {
        class: class(id),
        ..Run::default()
    }
}

fn owned(key: &str, level: u8) -> Owned {
    Owned {
        def: data::ability(key),
        level,
        cd: 0.0,
    }
}

fn passive(key: &str) -> &'static data::PassiveDef {
    data::PASSIVES
        .iter()
        .find(|p| p.key == key)
        .expect("a passive with that key")
}

fn start_of(id: u8) -> &'static str {
    class(id).start
}

// ── Ability ranks and cooldowns ──────────────────────────────────────────────────────────────────

#[test]
fn damage_grows_thirty_percent_a_rank() {
    assert!(close(owned(start_of(CASTER), 1).damage_mult(), 1.0));
    assert!(close(owned(start_of(CASTER), 2).damage_mult(), 1.3));
    assert!(close(owned(start_of(CASTER), 8).damage_mult(), 3.1));
}

#[test]
fn targets_arrive_at_ranks_three_five_and_seven() {
    let base = data::ability(start_of(CASTER)).count;
    for (rank, extra) in [
        (1, 0),
        (2, 0),
        (3, 1),
        (4, 1),
        (5, 2),
        (6, 2),
        (7, 3),
        (8, 3),
    ] {
        assert_eq!(
            owned(start_of(CASTER), rank).count(),
            base + extra,
            "rank {rank}"
        );
    }
}

#[test]
fn cooldown_falls_with_rank_and_with_haste() {
    let def = data::ability(start_of(CASTER));
    let stats = run_for(CASTER).stats();
    let at = |rank: u8, haste: f32| {
        owned(start_of(CASTER), rank).cooldown(&super::Stats { haste, ..stats })
    };
    assert!(close(at(1, 0.0), def.cooldown.max(0.2)));
    assert!(at(2, 0.0) < at(1, 0.0));
    // Haste shortens by its own fraction, but never past 65 %.
    assert!(close(at(1, 0.5), (def.cooldown * 0.5).max(0.2)));
    assert!(close(at(1, 0.9), (def.cooldown * 0.35).max(0.2)));
}

#[test]
fn cooldown_never_falls_below_a_fifth_of_a_second() {
    let stats = super::Stats {
        haste: 0.99,
        ..run_for(CASTER).stats()
    };
    for def in data::ABILITIES {
        let cd = owned(def.key, data::MAX_ABILITY_LEVEL).cooldown(&stats);
        assert!(cd.is_finite() && cd >= 0.2, "{}: {cd}", def.key);
    }
}

#[test]
fn radius_and_duration_scale_with_rank_and_area() {
    let def = data::ability(start_of(CASTER));
    let mut stats = run_for(CASTER).stats();
    stats.area = 1.5;
    assert!(close(owned(def.key, 1).radius(&stats), def.radius * 1.5));
    assert!(close(owned(def.key, 3).duration(), def.duration * 1.2));
}

// ── Hero stats and passives ──────────────────────────────────────────────────────────────────────

#[test]
fn classes_start_with_their_own_health_and_speed() {
    let warrior = run_for(WARRIOR).stats();
    assert!(close(warrior.max_hp, BASE_HP + 40.0));
    assert!(close(warrior.speed, BASE_SPEED));

    let hunter = run_for(HUNTER).stats();
    assert!(close(hunter.max_hp, BASE_HP + 15.0));

    let mage = run_for(CASTER).stats();
    assert!(close(mage.max_hp, BASE_HP));
    assert!(close(mage.speed, BASE_SPEED + 0.5));
}

#[test]
fn fortitude_multiplies_health_by_its_rank() {
    let mut run = run_for(CASTER);
    run.passives.push((passive("fortitude"), 2));
    assert!(close(run.stats().max_hp, BASE_HP * 1.4));
}

#[test]
fn additive_passives_stack_by_rank() {
    let mut run = run_for(CASTER);
    run.passives.push((passive("might"), 3));
    run.passives.push((passive("wild"), 2));
    run.passives.push((passive("renew"), 2));
    run.passives.push((passive("kings"), 4));
    let s = run.stats();
    assert!(close(s.might, 1.3));
    assert!(close(s.armor, 2.0));
    assert!(close(s.regen, 1.0));
    assert!(
        close(s.crit, 0.05 + 0.20),
        "base 5 % plus rank 4 of 5 % each"
    );
}

#[test]
fn magnet_and_speed_multiply_by_their_passives() {
    let mut run = run_for(CASTER);
    run.passives.push((passive("detect"), 1));
    run.passives.push((passive("pack"), 1));
    let s = run.stats();
    assert!(close(s.magnet, BASE_MAGNET * 1.4));
    assert!(close(s.speed, (BASE_SPEED + 0.5) * 1.1));
}

#[test]
fn every_guid_is_new() {
    let mut run = Run::default();
    let first = run.guid();
    let second = run.guid();
    assert!(second > first);
}

// ── Dice ─────────────────────────────────────────────────────────────────────────────────────────

#[test]
fn seeded_dice_repeat_exactly() {
    let draw = |seed| {
        let mut rng = Rng::seeded(seed);
        (0..100).map(|_| rng.next_u64()).collect::<Vec<_>>()
    };
    assert_eq!(draw(42), draw(42));
    assert_ne!(draw(42), draw(43));
}

#[test]
fn unit_draws_stay_in_zero_to_one() {
    let mut rng = Rng::seeded(7);
    for _ in 0..10_000 {
        let v = rng.f32();
        assert!((0.0..1.0).contains(&v), "{v}");
    }
}

#[test]
fn index_and_range_handle_edge_inputs() {
    let mut rng = Rng::seeded(9);
    assert_eq!(rng.index(0), 0, "an empty pick falls back to slot 0");
    for _ in 0..1_000 {
        assert!(rng.index(7) < 7);
        let r = rng.range(2.0, 3.0);
        assert!((2.0..3.0).contains(&r), "{r}");
    }
}

// ── Data invariants ──────────────────────────────────────────────────────────────────────────────

#[test]
fn ability_keys_are_unique_and_their_numbers_are_sane() {
    let mut keys: Vec<&str> = data::ABILITIES.iter().map(|a| a.key).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), data::ABILITIES.len(), "duplicate ability key");
    for def in data::ABILITIES {
        assert!(def.cooldown > 0.0, "{} cooldown", def.key);
        assert!(def.count >= 1, "{} count", def.key);
        assert!(def.damage >= 0.0 && def.dot >= 0.0, "{} damage", def.key);
        assert!(def.radius >= 0.0 && def.range >= 0.0, "{} reach", def.key);
        assert!(def.duration >= 0.0, "{} duration", def.key);
    }
}

#[test]
fn class_ids_are_unique_and_pools_resolve() {
    let mut ids: Vec<u8> = data::CLASSES.iter().map(|c| c.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), data::CLASSES.len(), "duplicate class id");
    for c in data::CLASSES {
        assert!(!c.looks.is_empty(), "{} has no looks", c.name);
        assert!(matches!(c.sheath, 1 | 2), "{} sheath", c.name);
        assert!(
            c.pool.contains(&c.start),
            "{} starts outside its pool",
            c.name
        );
        for key in c.pool {
            // `ability` panics on an unknown key, which is the failure this checks for.
            data::ability(key);
        }
    }
}

#[test]
fn passive_keys_are_unique_and_grow() {
    let mut keys: Vec<&str> = data::PASSIVES.iter().map(|p| p.key).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), data::PASSIVES.len(), "duplicate passive key");
    for p in data::PASSIVES {
        assert!(p.per_level > 0.0, "{} per_level", p.key);
    }
}

#[test]
fn enemy_tables_are_positive() {
    for e in data::ENEMIES.iter().chain(data::BOSSES) {
        assert!(e.hp > 0.0 && e.speed > 0.0 && e.damage > 0.0, "{}", e.name);
        assert!(e.scale > 0.0 && e.xp > 0 && e.level > 0, "{}", e.name);
    }
}

#[test]
fn every_battleground_has_six_bosses_and_enough_tiers() {
    let mut names: Vec<&str> = data::MAPS.iter().map(|m| m.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), data::MAPS.len(), "duplicate map name");
    for (i, map) in data::MAPS.iter().enumerate() {
        let roster = data::roster(i);
        assert_eq!(
            roster.bosses.len(),
            data::BOSSES.len(),
            "{} bosses",
            map.name
        );
        assert!(roster.enemies.len() >= 3, "{} needs three tiers", map.name);
        assert!(map.leash > 0.0, "{} leash", map.name);
    }
}

#[test]
fn ladder_rosters_never_soften_as_they_rise() {
    for (i, map) in data::MAPS
        .iter()
        .enumerate()
        .filter(|(_, m)| !m.foes.is_empty())
    {
        let roster = &data::roster(i).enemies;
        for pair in roster.windows(2) {
            assert!(pair[1].hp >= pair[0].hp, "{} hp", map.name);
            assert!(pair[1].damage >= pair[0].damage, "{} damage", map.name);
            assert!(pair[1].xp >= pair[0].xp, "{} xp", map.name);
        }
        assert!(roster
            .iter()
            .all(|e| e.hp > 0.0 && e.damage > 0.0 && e.xp > 0));
    }
}

// ── Pacing, scaling and the wave clock ───────────────────────────────────────────────────────────

#[test]
fn progress_clamps_negative_and_nan_time_to_zero() {
    assert_eq!(data::progress(-30.0), 0.0);
    assert_eq!(data::progress(f32::NAN), 0.0);
    assert!(close(data::progress(0.0), 0.0));
    assert!(close(data::progress(450.0), 0.5));
    assert!(close(data::progress(data::VICTORY_SECS), 1.0));
}

#[test]
fn enemy_health_and_damage_scale_across_the_run() {
    assert!(close(data::hp_scale(0.0), 1.0));
    assert!(close(data::hp_scale(data::VICTORY_SECS), 1.75));
    assert!(close(data::damage_scale(0.0), 1.0));
    assert!(close(data::damage_scale(data::VICTORY_SECS), 1.9));
}

#[test]
fn target_count_grows_and_is_capped() {
    assert_eq!(data::target_alive(0.0), 14);
    assert_eq!(data::target_alive(data::VICTORY_SECS), 220);
    assert_eq!(data::target_alive(10_000.0), 220);
    let mut last = 0;
    for t in (0..=1_000).map(|t| t as f32) {
        let n = data::target_alive(t);
        assert!(n >= last, "fell at {t}s");
        last = n;
    }
}

#[test]
fn wave_pool_stays_in_its_roster_and_moves_forward() {
    for i in 0..data::MAPS.len() {
        let tiers = data::roster(i).enemies.len();
        let mut last = [0usize; 3];
        for t in (0..=1_200).map(|t| t as f32) {
            let pool = data::wave_pool(t, tiers);
            assert!(pool.iter().all(|&p| p < tiers), "map {i} at {t}s: {pool:?}");
            assert!(pool[0] >= last[0], "map {i} went back at {t}s");
            last = pool;
        }
    }
    assert_eq!(data::wave_pool(0.0, 3), [0, 1, 2]);
}

#[test]
fn bosses_come_every_135_seconds_and_all_before_victory() {
    assert_eq!(data::bosses_due(0.0, 6), 0);
    assert_eq!(data::bosses_due(134.9, 6), 0);
    assert_eq!(data::bosses_due(data::BOSS_EVERY_SECS, 6), 1);
    assert_eq!(data::bosses_due(6.0 * data::BOSS_EVERY_SECS, 6), 6);
    assert_eq!(data::bosses_due(10_000.0, 6), 6, "capped by the roster");
}

const _: () = assert!(6.0 * data::BOSS_EVERY_SECS < data::VICTORY_SECS);

#[test]
fn invalid_clock_values_schedule_nothing() {
    assert_eq!(data::bosses_due(-5.0, 6), 0);
    assert_eq!(data::bosses_due(f32::NAN, 6), 0);
    assert_eq!(data::rings_due(-5.0), 0);
    assert_eq!(data::rings_due(f32::NAN), 0);
}

#[test]
fn closing_rings_fall_halfway_between_bosses() {
    let half = data::BOSS_EVERY_SECS / 2.0;
    assert_eq!(data::rings_due(0.0), 0);
    assert_eq!(data::rings_due(half - 0.1), 0);
    assert_eq!(data::rings_due(half), 1);
    assert_eq!(data::rings_due(half + data::BOSS_EVERY_SECS), 2);
}

#[test]
fn experience_curve_rises_for_every_level_in_play() {
    assert!(close(data::xp_to_next(1), 5.0));
    assert!(close(data::xp_to_next(2), 15.0));
    assert!(close(data::xp_to_next(20), 195.0));
    assert!(close(data::xp_to_next(40), 455.0));
    for level in 1..100 {
        assert!(
            data::xp_to_next(level + 1) > data::xp_to_next(level),
            "level {level}"
        );
    }
}

#[test]
fn spending_experience_carries_the_remainder() {
    let (mut xp, mut level) = (0.0, 1);
    assert_eq!(data::spend_xp(&mut xp, &mut level), 0);
    assert_eq!((xp, level), (0.0, 1));

    let (mut xp, mut level) = (5.0, 1);
    assert_eq!(data::spend_xp(&mut xp, &mut level), 1);
    assert_eq!((xp, level), (0.0, 2));

    let (mut xp, mut level) = (20.0, 1);
    assert_eq!(data::spend_xp(&mut xp, &mut level), 2);
    assert!(close(xp, 0.0) && level == 3);

    let (mut xp, mut level) = (1_000.0, 1);
    let gained = data::spend_xp(&mut xp, &mut level);
    assert_eq!(level, 1 + gained);
    assert!(
        xp >= 0.0 && xp < data::xp_to_next(level),
        "left {xp} at level {level}"
    );
}

#[test]
fn death_beats_victory_and_victory_needs_the_clock() {
    assert_eq!(hero::outcome(0.0, 0.0), Some(Outcome::Died));
    assert_eq!(hero::outcome(-5.0, data::VICTORY_SECS), Some(Outcome::Died));
    assert_eq!(hero::outcome(1.0, data::VICTORY_SECS - 0.1), None);
    assert_eq!(hero::outcome(1.0, data::VICTORY_SECS), Some(Outcome::Won));
}

// ── Damage taken ─────────────────────────────────────────────────────────────────────────────────

#[test]
fn armour_takes_its_share_and_a_blow_always_lands_for_one() {
    let mut run = run_for(CASTER);
    run.passives.push((passive("wild"), 3));
    let got = hero::hurt_hero(&mut run, 10.0);
    assert!(close(got, 7.0));
    assert!(close(run.hp, BASE_HP - 7.0));

    let got = hero::hurt_hero(&mut run, 2.0);
    assert!(close(got, 1.0), "a glancing blow still deals one");
    assert!(close(run.hp, BASE_HP - 8.0));

    assert!(close(hero::hurt_hero(&mut run, 0.0), 1.0));
    assert!(close(hero::hurt_hero(&mut run, -40.0), 1.0));
}

#[test]
fn immunity_and_a_fallen_hero_take_nothing() {
    let mut run = run_for(CASTER);
    run.immune_t = 1.0;
    assert_eq!(hero::hurt_hero(&mut run, 50.0), 0.0);
    assert!(close(run.hp, BASE_HP));

    let mut run = run_for(CASTER);
    run.dead_t = Some(0.5);
    assert_eq!(hero::hurt_hero(&mut run, 50.0), 0.0);
    assert!(close(run.hp, BASE_HP));
}

#[test]
fn shield_soaks_only_while_it_is_up() {
    let mut run = run_for(CASTER);
    run.shield = 10.0;
    run.shield_t = 5.0;
    assert_eq!(hero::hurt_hero(&mut run, 6.0), 0.0, "fully soaked");
    assert!(close(run.shield, 4.0));
    assert!(close(run.hp, BASE_HP));

    let got = hero::hurt_hero(&mut run, 10.0);
    assert!(close(got, 6.0), "the shield's last four, then health");
    assert!(close(run.shield, 0.0));
    assert!(close(run.hp, BASE_HP - 6.0));

    let mut run = run_for(CASTER);
    run.shield = 10.0;
    run.shield_t = 0.0;
    assert!(
        close(hero::hurt_hero(&mut run, 6.0), 6.0),
        "an expired shield absorbs nothing"
    );
}

// ── Offers and picks ─────────────────────────────────────────────────────────────────────────────

fn label(c: &Choice) -> String {
    match c {
        Choice::NewAbility(def) => format!("new:{}", def.key),
        Choice::Upgrade(k) => format!("up:{k}"),
        Choice::NewPassive(p) => format!("newp:{}", p.key),
        Choice::UpPassive(k) => format!("upp:{k}"),
        Choice::Heal => "heal".to_owned(),
    }
}

fn full_ability_slots() -> Vec<Owned> {
    data::ABILITIES
        .iter()
        .take(data::MAX_ABILITIES)
        .map(|d| owned(d.key, 1))
        .collect()
}

#[test]
fn a_fresh_run_is_offered_three_distinct_choices() {
    let run = run_for(CASTER);
    for seed in 1..200 {
        let offer = hud::choices_for(&run, &mut Rng::seeded(seed));
        assert_eq!(offer.len(), 3, "seed {seed}");
        let mut labels: Vec<String> = offer.iter().map(label).collect();
        labels.sort();
        labels.dedup();
        assert_eq!(labels.len(), 3, "seed {seed} repeated a card");
    }
}

#[test]
fn an_offer_never_teaches_a_spell_the_run_already_has() {
    let mut run = run_for(CASTER);
    run.abilities.push(owned(start_of(CASTER), 1));
    for seed in 1..300 {
        for c in hud::choices_for(&run, &mut Rng::seeded(seed)) {
            if let Choice::NewAbility(def) = c {
                assert_ne!(def.key, start_of(CASTER), "seed {seed}");
            }
        }
    }
}

#[test]
fn full_ability_slots_offer_no_new_spells() {
    let mut run = run_for(CASTER);
    run.abilities = full_ability_slots();
    for seed in 1..300 {
        for c in hud::choices_for(&run, &mut Rng::seeded(seed)) {
            assert!(!matches!(c, Choice::NewAbility(_)), "seed {seed}");
        }
    }
}

#[test]
fn full_passive_slots_offer_no_new_passives() {
    let mut run = run_for(CASTER);
    run.passives = data::PASSIVES
        .iter()
        .take(data::MAX_PASSIVES)
        .map(|p| (p, 1))
        .collect();
    for seed in 1..300 {
        for c in hud::choices_for(&run, &mut Rng::seeded(seed)) {
            assert!(!matches!(c, Choice::NewPassive(_)), "seed {seed}");
        }
    }
}

#[test]
fn a_maxed_spell_is_never_offered_an_upgrade() {
    let mut run = run_for(CASTER);
    run.abilities = vec![owned(start_of(CASTER), data::MAX_ABILITY_LEVEL)];
    for seed in 1..200 {
        for c in hud::choices_for(&run, &mut Rng::seeded(seed)) {
            assert!(!matches!(c, Choice::Upgrade(_)), "seed {seed}");
        }
    }
}

#[test]
fn a_fully_ranked_run_is_offered_only_a_heal() {
    let mut run = run_for(CASTER);
    run.abilities = full_ability_slots()
        .into_iter()
        .map(|o| Owned {
            level: data::MAX_ABILITY_LEVEL,
            ..o
        })
        .collect();
    run.passives = data::PASSIVES
        .iter()
        .take(data::MAX_PASSIVES)
        .map(|p| (p, data::MAX_PASSIVE_LEVEL))
        .collect();
    let offer = hud::choices_for(&run, &mut Rng::seeded(3));
    assert_eq!(offer.iter().map(label).collect::<Vec<_>>(), ["heal"]);
}

#[test]
fn the_same_seed_gives_the_same_offer() {
    let run = run_for(CASTER);
    let a: Vec<String> = hud::choices_for(&run, &mut Rng::seeded(11))
        .iter()
        .map(label)
        .collect();
    let b: Vec<String> = hud::choices_for(&run, &mut Rng::seeded(11))
        .iter()
        .map(label)
        .collect();
    assert_eq!(a, b);
}

#[test]
fn picking_a_spell_adds_it_at_rank_one_and_resumes_play() {
    let mut run = Run {
        pending_levels: 1,
        ..run_for(CASTER)
    };
    let mut offer = Offer {
        choices: vec![Choice::NewAbility(data::ability(start_of(CASTER)))],
    };
    let mut next = NextState::<Phase>::default();
    hud::pick(0, &mut run, &mut offer, &mut Rng::seeded(1), &mut next);

    assert_eq!(run.abilities.len(), 1);
    assert_eq!(run.abilities[0].level, 1);
    assert_eq!(run.pending_levels, 0);
    assert!(offer.choices().is_empty());
    assert!(matches!(next, NextState::Pending(Phase::Playing)));
}

#[test]
fn picking_while_more_levels_are_owed_redraws_the_offer() {
    let mut run = Run {
        pending_levels: 2,
        ..run_for(CASTER)
    };
    run.abilities.push(owned(start_of(CASTER), 1));
    let mut offer = Offer {
        choices: vec![Choice::Upgrade(0)],
    };
    let mut next = NextState::<Phase>::default();
    hud::pick(0, &mut run, &mut offer, &mut Rng::seeded(1), &mut next);

    assert_eq!(run.abilities[0].level, 2);
    assert_eq!(run.pending_levels, 1);
    assert_eq!(offer.choices().len(), 3, "the next level-up is dealt");
    assert!(matches!(next, NextState::Unchanged));
}

#[test]
fn an_out_of_range_pick_changes_nothing() {
    let mut run = Run {
        pending_levels: 1,
        ..run_for(CASTER)
    };
    let mut offer = Offer {
        choices: vec![Choice::Heal],
    };
    let mut next = NextState::<Phase>::default();
    hud::pick(4, &mut run, &mut offer, &mut Rng::seeded(1), &mut next);

    assert_eq!(run.pending_levels, 1);
    assert_eq!(offer.choices().len(), 1);
    assert!(matches!(next, NextState::Unchanged));
}

#[test]
fn heal_restores_health_to_the_maximum() {
    let mut run = Run {
        hp: 1.0,
        pending_levels: 1,
        ..run_for(CASTER)
    };
    let mut offer = Offer {
        choices: vec![Choice::Heal],
    };
    let mut next = NextState::<Phase>::default();
    hud::pick(0, &mut run, &mut offer, &mut Rng::seeded(1), &mut next);
    assert!(close(run.hp, run.stats().max_hp));
}
