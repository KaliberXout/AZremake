use super::*;

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
