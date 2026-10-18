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

#[test]
fn a_degenerate_range_returns_its_bound() {
    let mut rng = Rng::seeded(4);
    assert_eq!(rng.range(2.5, 2.5), 2.5);
}
