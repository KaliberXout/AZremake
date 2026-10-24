//! The mode's pure logic: the seeded dice, the level-up offer, ability and stat scaling, the
//! run's pacing curves and the static tables. No WoW data, no window, no `App`.

use bevy::prelude::*;

use super::cards::{rarity, Rarity};
use super::data::{self, Stat};
use super::hud::{choices_for, pick, start_run, Choice, Offer};
use super::{Owned, Phase, Rng, Run, Selection, BASE_HP, BASE_MAGNET, BASE_SPEED};

/// A choice as a comparable string.
fn key(c: &Choice) -> String {
    match c {
        Choice::NewAbility(d) => format!("new:{}", d.key),
        Choice::Upgrade(i) => format!("up:{i}"),
        Choice::NewPassive(p) => format!("newp:{}", p.key),
        Choice::UpPassive(i) => format!("upp:{i}"),
        Choice::Heal => "heal".into(),
    }
}

fn keys(cs: &[Choice]) -> Vec<String> {
    cs.iter().map(key).collect()
}

/// A run of class `class` (an index into [`data::CLASSES`]) holding its start ability, as
/// [`start_run`] leaves it.
fn run_of(class: usize, seed: u64) -> (Run, Rng) {
    let mut selection = Selection {
        class: Some(class),
        looks: vec![(0, 1, 0)],
        look: Some(0),
        ..Selection::default()
    };
    let mut run = Run::default();
    let mut rng = Rng::seeded(0);
    assert!(start_run(&mut run, &mut selection, &mut rng, seed));
    (run, rng)
}

/// Level up `levels` times, always taking the first card; the cards seen, in order.
fn play_levels(class: usize, seed: u64, levels: usize) -> Vec<Vec<String>> {
    let (mut run, mut rng) = run_of(class, seed);
    let mut offer = Offer::default();
    let mut next = NextState::<Phase>::default();
    let mut seen = Vec::new();
    for _ in 0..levels {
        run.pending_levels = 1;
        offer.choices = choices_for(&run, &mut rng);
        seen.push(keys(&offer.choices));
        pick(0, &mut run, &mut offer, &mut rng, &mut next);
    }
    seen
}

// --- the dice ---

#[test]
fn the_same_seed_rolls_the_same_sequence() {
    let (mut a, mut b) = (Rng::seeded(42), Rng::seeded(42));
    for _ in 0..1000 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}

#[test]
fn different_seeds_roll_different_sequences() {
    let first: Vec<u64> = (0..64).map(|s| Rng::seeded(s).next_u64()).collect();
    let mut unique = first.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), first.len(), "neighbouring seeds collide");
}

#[test]
fn seed_zero_is_not_stuck() {
    let mut rng = Rng::seeded(0);
    let draws: Vec<u64> = (0..8).map(|_| rng.next_u64()).collect();
    assert!(draws.iter().all(|&d| d != 0));
    assert!(draws.windows(2).all(|w| w[0] != w[1]));
}

#[test]
fn the_dice_stay_in_their_ranges() {
    let mut rng = Rng::seeded(7);
    for _ in 0..10_000 {
        let f = rng.f32();
        assert!((0.0..1.0).contains(&f));
        let r = rng.range(34.0, 42.0);
        assert!((34.0..=42.0).contains(&r));
        assert!(rng.index(5) < 5);
        assert_eq!(rng.index(0), 0);
    }
}

#[test]
fn the_dice_are_roughly_uniform() {
    let mut rng = Rng::seeded(1);
    let mut buckets = [0u32; 10];
    for _ in 0..100_000 {
        buckets[(rng.f32() * 10.0) as usize] += 1;
    }
    for b in buckets {
        assert!((9_000..11_000).contains(&b), "{buckets:?}");
    }
}

// --- the run's seed ---

#[test]
fn starting_a_run_resets_the_dice_to_its_seed() {
    let (run, mut rng) = run_of(0, 1234);
    assert_eq!(run.seed, 1234);
    assert_eq!(rng.next_u64(), Rng::seeded(1234).next_u64());
}

#[test]
fn starting_a_run_needs_a_class_and_a_look() {
    let mut run = Run::default();
    let mut rng = Rng::seeded(0);
    let mut selection = Selection::default();
    assert!(!start_run(&mut run, &mut selection, &mut rng, 1));
    selection.class = Some(0);
    assert!(!start_run(&mut run, &mut selection, &mut rng, 1));
}

#[test]
fn a_new_run_holds_its_class_start_and_full_health() {
    for (i, class) in data::CLASSES.iter().enumerate() {
        let (run, _) = run_of(i, 9);
        assert_eq!(run.abilities.len(), 1, "{}", class.name);
        assert_eq!(run.abilities[0].def.key, class.start);
        assert_eq!(run.hp, run.stats().max_hp);
        assert_eq!(run.level, 1);
    }
}

#[test]
fn guids_stay_unique_across_runs() {
    let mut selection = Selection {
        class: Some(0),
        looks: vec![(0, 1, 0)],
        look: Some(0),
        ..Selection::default()
    };
    let mut run = Run::default();
    let mut rng = Rng::seeded(0);
    run.guid();
    run.guid();
    let before = run.next_guid;
    assert!(start_run(&mut run, &mut selection, &mut rng, 5));
    assert_eq!(run.next_guid, before);
}

#[test]
fn the_same_seed_and_picks_deal_the_same_cards() {
    for class in 0..data::CLASSES.len() {
        assert_eq!(play_levels(class, 77, 30), play_levels(class, 77, 30));
    }
}

#[test]
fn different_seeds_deal_different_cards() {
    let runs: Vec<_> = (0..8).map(|s| play_levels(0, s, 10)).collect();
    assert!(
        runs.iter().any(|r| *r != runs[0]),
        "eight seeds dealt identical offers"
    );
}

// --- the level-up offer ---

#[test]
fn an_offer_is_three_distinct_cards_the_run_can_take() {
    for class in 0..data::CLASSES.len() {
        for seed in 0..20 {
            let (mut run, mut rng) = run_of(class, seed);
            let mut offer = Offer::default();
            let mut next = NextState::<Phase>::default();
            for _ in 0..40 {
                let cs = choices_for(&run, &mut rng);
                let ks = keys(&cs);
                assert!(!cs.is_empty() && cs.len() <= 3);
                let mut unique = ks.clone();
                unique.sort();
                unique.dedup();
                assert_eq!(unique.len(), ks.len(), "duplicate card in {ks:?}");
                for c in &cs {
                    match *c {
                        Choice::Upgrade(i) => {
                            assert!(run.abilities[i].level < data::MAX_ABILITY_LEVEL)
                        }
                        Choice::NewAbility(d) => {
                            assert!(run.class.pool.contains(&d.key));
                            assert!(!run.abilities.iter().any(|o| o.def.key == d.key));
                            assert!(run.abilities.len() < data::MAX_ABILITIES);
                        }
                        Choice::UpPassive(i) => {
                            assert!(run.passives[i].1 < data::MAX_PASSIVE_LEVEL)
                        }
                        Choice::NewPassive(p) => {
                            assert!(!run.passives.iter().any(|(q, _)| q.key == p.key));
                            assert!(run.passives.len() < data::MAX_PASSIVES);
                        }
                        Choice::Heal => assert_eq!(cs.len(), 1),
                    }
                }
                run.pending_levels = 1;
                offer.choices = cs;
                let i = rng.index(offer.choices.len());
                pick(i, &mut run, &mut offer, &mut rng, &mut next);
                assert!(run.abilities.len() <= data::MAX_ABILITIES);
                assert!(run.passives.len() <= data::MAX_PASSIVES);
            }
        }
    }
}

#[test]
fn a_finished_build_is_offered_only_a_heal() {
    let (mut run, mut rng) = run_of(0, 3);
    run.abilities = run.class.pool[..data::MAX_ABILITIES.min(run.class.pool.len())]
        .iter()
        .map(|k| Owned {
            def: data::ability(k),
            level: data::MAX_ABILITY_LEVEL,
            cd: 0.0,
        })
        .collect();
    run.passives = data::PASSIVES[..data::MAX_PASSIVES]
        .iter()
        .map(|p| (p, data::MAX_PASSIVE_LEVEL))
        .collect();
    assert_eq!(keys(&choices_for(&run, &mut rng)), vec!["heal"]);
}

#[test]
fn picking_banks_a_level_and_returns_to_play() {
    let (mut run, mut rng) = run_of(0, 11);
    let mut offer = Offer::default();
    let mut next = NextState::<Phase>::default();
    run.pending_levels = 2;
    offer.choices = choices_for(&run, &mut rng);
    pick(0, &mut run, &mut offer, &mut rng, &mut next);
    assert_eq!(run.pending_levels, 1);
    assert!(!offer.choices.is_empty(), "a second level deals again");
    assert!(matches!(next, NextState::Unchanged));
    pick(0, &mut run, &mut offer, &mut rng, &mut next);
    assert_eq!(run.pending_levels, 0);
    assert!(offer.choices.is_empty());
    assert!(matches!(next, NextState::Pending(Phase::Playing)));
}

#[test]
fn picking_a_card_applies_it() {
    let (mut run, mut rng) = run_of(0, 0);
    let mut next = NextState::<Phase>::default();
    let passive = &data::PASSIVES[0];
    let mut offer = Offer {
        choices: vec![Choice::Upgrade(0), Choice::NewPassive(passive)],
    };
    run.pending_levels = 2;
    pick(0, &mut run, &mut offer, &mut rng, &mut next);
    assert_eq!(run.abilities[0].level, 2);
    offer.choices = vec![Choice::NewPassive(passive)];
    pick(0, &mut run, &mut offer, &mut rng, &mut next);
    assert_eq!(run.passives.len(), 1);
    assert_eq!(run.passives[0].1, 1);
}

#[test]
fn an_out_of_range_pick_does_nothing() {
    let (mut run, mut rng) = run_of(0, 0);
    let mut next = NextState::<Phase>::default();
    let mut offer = Offer {
        choices: vec![Choice::Heal],
    };
    run.pending_levels = 1;
    pick(5, &mut run, &mut offer, &mut rng, &mut next);
    assert_eq!(run.pending_levels, 1);
    assert!(matches!(next, NextState::Unchanged));
}

#[test]
fn a_last_rank_is_legendary() {
    let (mut run, _) = run_of(0, 0);
    run.abilities[0].level = data::MAX_ABILITY_LEVEL - 1;
    assert_eq!(rarity(Choice::Upgrade(0), &run), Rarity::Legendary);
    assert_eq!(rarity(Choice::Heal, &run), Rarity::Common);
}

// --- ability and stat scaling ---

#[test]
fn ability_ranks_scale_as_documented() {
    let base = run_of(0, 0).0.stats();
    for def in data::ABILITIES {
        let at = |level| Owned {
            def,
            level,
            cd: 0.0,
        };
        let one = at(1);
        assert_eq!(one.damage_mult(), 1.0);
        assert_eq!(one.count(), def.count, "{}", def.key);
        assert_eq!(one.duration(), def.duration);
        let mut prev = one;
        for level in 2..=data::MAX_ABILITY_LEVEL {
            let o = at(level);
            assert!(o.damage_mult() > prev.damage_mult());
            assert!(o.cooldown(&base) <= prev.cooldown(&base));
            assert!(
                o.cooldown(&base) >= 0.2,
                "{} cools below the floor",
                def.key
            );
            assert!(o.radius(&base) >= prev.radius(&base));
            prev = o;
        }
        // Three extra targets, at ranks 3, 5 and 7.
        assert_eq!(at(data::MAX_ABILITY_LEVEL).count(), def.count + 3);
        assert_eq!(at(2).count(), def.count);
        assert_eq!(at(3).count(), def.count + 1);
    }
}

#[test]
fn haste_is_capped() {
    let mut stats = run_of(0, 0).0.stats();
    stats.haste = 5.0;
    let o = Owned {
        def: data::ability("frostbolt"),
        level: 1,
        cd: 0.0,
    };
    let floor = (o.def.cooldown * 0.35).max(0.2);
    assert!((o.cooldown(&stats) - floor).abs() < 1e-6);
}

#[test]
fn a_bare_hero_has_the_base_stats() {
    for (i, class) in data::CLASSES.iter().enumerate() {
        let s = run_of(i, 0).0.stats();
        assert_eq!(s.might, 1.0);
        assert_eq!(s.magnet, BASE_MAGNET);
        assert!(s.max_hp >= BASE_HP, "{}", class.name);
        assert!(s.speed >= BASE_SPEED, "{}", class.name);
        assert_eq!(s.crit, 0.05);
    }
}

#[test]
fn every_passive_moves_its_stat() {
    for p in data::PASSIVES {
        let (mut run, _) = run_of(0, 0);
        let before = run.stats();
        run.passives.push((p, 2));
        let after = run.stats();
        let (b, a) = match p.stat {
            Stat::Might => (before.might, after.might),
            Stat::MaxHp => (before.max_hp, after.max_hp),
            Stat::Speed => (before.speed, after.speed),
            Stat::Regen => (before.regen, after.regen),
            Stat::Haste => (before.haste, after.haste),
            Stat::Area => (before.area, after.area),
            Stat::Magnet => (before.magnet, after.magnet),
            Stat::Growth => (before.growth, after.growth),
            Stat::Armor => (before.armor, after.armor),
            Stat::Crit => (before.crit, after.crit),
        };
        assert!(a > b, "{} did not raise its stat", p.key);
    }
}

// --- pacing curves ---

#[test]
fn the_pacing_curves_rise_and_stay_bounded() {
    let mut t = 0.0;
    let (mut alive, mut hp, mut dmg) = (0, 0.0, 0.0);
    while t <= data::VICTORY_SECS * 1.5 {
        let a = data::target_alive(t);
        assert!(a >= alive && a <= 220);
        assert!(data::hp_scale(t) >= hp && data::damage_scale(t) >= dmg);
        (alive, hp, dmg) = (a, data::hp_scale(t), data::damage_scale(t));
        t += 1.0;
    }
    assert_eq!(data::progress(0.0), 0.0);
    assert_eq!(data::progress(data::VICTORY_SECS), 1.0);
}

#[test]
fn the_experience_curve_is_positive_and_rising() {
    let mut prev = 0.0;
    for level in 1..100 {
        let need = data::xp_to_next(level);
        assert!(need > 0.0 && need > prev, "level {level}");
        prev = need;
    }
}

#[test]
fn every_wave_pool_indexes_its_roster() {
    for map in 0..data::MAPS.len() {
        let tiers = data::roster(map).enemies.len();
        let mut t = 0.0;
        while t <= data::VICTORY_SECS * 1.5 {
            for i in data::wave_pool(t, tiers) {
                assert!(
                    i < tiers,
                    "{} at {t}s: tier {i} of {tiers}",
                    data::MAPS[map].name
                );
            }
            t += 5.0;
        }
    }
}

// --- the static tables ---

#[test]
fn every_class_ability_exists_and_is_unique() {
    for class in data::CLASSES {
        assert!(class.pool.contains(&class.start), "{}", class.name);
        for key in class.pool {
            assert!(
                data::ABILITIES.iter().any(|a| a.key == *key),
                "{}: no ability {key}",
                class.name
            );
        }
    }
    let mut ks: Vec<_> = data::ABILITIES.iter().map(|a| a.key).collect();
    ks.sort_unstable();
    let n = ks.len();
    ks.dedup();
    assert_eq!(ks.len(), n, "duplicate ability key");
}

#[test]
fn every_map_fields_a_horde_and_its_bosses() {
    let due = (data::VICTORY_SECS / data::BOSS_EVERY_SECS) as usize;
    for (i, map) in data::MAPS.iter().enumerate() {
        let r = data::roster(i);
        assert!(r.enemies.len() >= 3, "{}", map.name);
        assert!(
            r.bosses.len() >= due.min(6),
            "{}: {} bosses",
            map.name,
            r.bosses.len()
        );
        assert!(r.enemies.iter().chain(&r.bosses).all(|e| e.hp > 0.0));
    }
}
