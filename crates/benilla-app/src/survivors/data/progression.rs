/// Survive this long to win.
pub(crate) const VICTORY_SECS: f32 = 900.0;
/// A boss every two and a quarter minutes, the last of six at 13:30, a fight before the bell.
pub(crate) const BOSS_EVERY_SECS: f32 = 135.0;
/// Experience per gem: every level of the curve comes at the run's pace.
pub(crate) const XP_RATE: f32 = 2.0;

/// How many of a battleground's `total` bosses are due by `elapsed`, one every [`BOSS_EVERY_SECS`].
pub(crate) fn bosses_due(elapsed: f32, total: usize) -> u32 {
    ((elapsed / BOSS_EVERY_SECS) as u32).min(total as u32)
}

/// How many closing rings are due by `elapsed`: one half way between each pair of bosses.
pub(crate) fn rings_due(elapsed: f32) -> u32 {
    ((elapsed + BOSS_EVERY_SECS / 2.0) / BOSS_EVERY_SECS) as u32
}

/// How far through the run, 0 at the start and 1 at [`VICTORY_SECS`].
pub(crate) fn progress(elapsed: f32) -> f32 {
    (elapsed / VICTORY_SECS).max(0.0)
}

/// The pool of roster indices at a point in the run, from a roster `tiers` long.
pub(crate) fn wave_pool(elapsed: f32, tiers: usize) -> [usize; 3] {
    // The roster spread over thirty steps of the run, the last three from step 28: on The
    // Barrens' thirty tiers, one newer every step.
    let step = (progress(elapsed) * 30.0) as usize;
    let span = tiers.saturating_sub(3);
    let lo = (step.saturating_sub(1) * span / 27).min(span);
    [lo, lo + 1, lo + 2]
}

/// How many enemies the spawner keeps alive at a point in the run.
pub(crate) fn target_alive(elapsed: f32) -> usize {
    ((14.0 + progress(elapsed) * 225.0) as usize).min(220)
}

/// Enemy health grows on top of the tiers, so a late kill takes several spells.
pub(crate) fn hp_scale(elapsed: f32) -> f32 {
    1.0 + progress(elapsed) * 0.75
}

/// Enemy damage grows the same way.
pub(crate) fn damage_scale(elapsed: f32) -> f32 {
    1.0 + progress(elapsed) * 0.9
}

/// The experience to go from `level` to the next: Vampire Survivors' curve, cheap early.
pub(crate) fn xp_to_next(level: u32) -> f32 {
    let l = level as f32;
    if level < 20 {
        5.0 + (l - 1.0) * 10.0
    } else if level < 40 {
        195.0 + (l - 20.0) * 13.0
    } else {
        455.0 + (l - 40.0) * 16.0
    }
}

/// Spends the experience that reaches the next level, one level at a time, and returns how many
/// levels it gained.
pub(crate) fn spend_xp(xp: &mut f32, level: &mut u32) -> u32 {
    let mut gained = 0;
    while *xp >= xp_to_next(*level) {
        *xp -= xp_to_next(*level);
        *level += 1;
        gained += 1;
    }
    gained
}
