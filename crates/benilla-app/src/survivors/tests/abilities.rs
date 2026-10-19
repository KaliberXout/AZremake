use super::*;

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
        owned(start_of(CASTER), rank).cooldown(&super::super::Stats { haste, ..stats })
    };
    assert!(close(at(1, 0.0), def.cooldown.max(0.2)));
    assert!(at(2, 0.0) < at(1, 0.0));
    // Haste shortens by its own fraction, but never past 65 %.
    assert!(close(at(1, 0.5), (def.cooldown * 0.5).max(0.2)));
    assert!(close(at(1, 0.9), (def.cooldown * 0.35).max(0.2)));
}

#[test]
fn cooldown_never_falls_below_a_fifth_of_a_second() {
    let stats = super::super::Stats {
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
