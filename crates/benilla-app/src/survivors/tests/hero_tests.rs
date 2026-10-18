use super::*;

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
