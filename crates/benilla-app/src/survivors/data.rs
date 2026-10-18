//! The mode's tables: classes, abilities, passives, enemies and the wave clock. Spell ids name
//! `Spell.dbc` rows only for their icon, name and visual chain; every number here is the mode's.

mod abilities;
mod classes;
mod enemies;
mod maps;
mod passives;
mod progression;

pub(crate) use abilities::{
    ability, AbilityDef, Effect, Kind, Targeting, MAX_ABILITIES, MAX_ABILITY_LEVEL,
};
// Keep the existing `data::ABILITIES` path for test and internal callers.
#[allow(unused_imports)]
pub(crate) use abilities::ABILITIES;
pub(crate) use classes::{ClassDef, Weapon, CLASSES};
pub(crate) use enemies::{EnemyDef, BOSSES, ENEMIES};
pub(crate) use maps::{Foe, MapDef, MAPS};
pub(crate) use passives::{PassiveDef, Stat, MAX_PASSIVES, MAX_PASSIVE_LEVEL, PASSIVES};

pub(crate) use progression::{
    bosses_due, damage_scale, hp_scale, progress, rings_due, spend_xp, target_alive, wave_pool,
    xp_to_next, VICTORY_SECS, XP_RATE,
};
// Keep the existing `data::BOSS_EVERY_SECS` path for test and internal callers.
#[allow(unused_imports)]
pub(crate) use progression::BOSS_EVERY_SECS;
/// A battleground's horde and bosses with their numbers.
pub(crate) struct Roster {
    pub enemies: Vec<EnemyDef>,
    pub bosses: Vec<EnemyDef>,
}

static ROSTERS: std::sync::LazyLock<Vec<Roster>> = std::sync::LazyLock::new(|| {
    MAPS.iter()
        .map(|m| {
            if m.foes.is_empty() {
                Roster {
                    enemies: ENEMIES.to_vec(),
                    bosses: BOSSES.to_vec(),
                }
            } else {
                Roster {
                    enemies: ladder(m.foes, ENEMIES),
                    bosses: ladder(m.bosses, BOSSES),
                }
            }
        })
        .collect()
});

pub(crate) fn roster(map: usize) -> &'static Roster {
    &ROSTERS[map.min(ROSTERS.len() - 1)]
}

/// Number a roster off a tuned ladder: each creature takes the health, damage and experience at
/// its own place along it, from the ladder's running peak so no later tier comes out softer.
fn ladder(foes: &[Foe], tuned: &[EnemyDef]) -> Vec<EnemyDef> {
    let mut peak = (0.0f32, 0.0f32, 0.0f32);
    let rungs: Vec<(f32, f32, f32)> = tuned
        .iter()
        .map(|d| {
            peak = (
                peak.0.max(d.hp),
                peak.1.max(d.damage),
                peak.2.max(d.xp as f32),
            );
            peak
        })
        .collect();
    let top = (rungs.len() - 1) as f32;
    foes.iter()
        .enumerate()
        .map(|(i, f)| {
            let t = if foes.len() > 1 {
                i as f32 / (foes.len() - 1) as f32 * top
            } else {
                0.0
            };
            let (lo, hi) = (rungs[t.floor() as usize], rungs[t.ceil() as usize]);
            let k = t.fract();
            let mix = |a: f32, b: f32| (a + (b - a) * k).round();
            EnemyDef {
                name: f.name,
                display: f.display,
                hp: mix(lo.0, hi.0),
                speed: f.speed,
                damage: mix(lo.1, hi.1),
                scale: f.scale,
                xp: mix(lo.2, hi.2) as u32,
                level: f.level,
                big: f.big,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targeting_toughest_abilities() {
        // Regression test for P2-001: verify these abilities use Toughest targeting
        let pyroblast = ability("pyroblast");
        assert_eq!(pyroblast.targeting, Targeting::Toughest);

        let aimed_shot = ability("aimed_shot");
        assert_eq!(aimed_shot.targeting, Targeting::Toughest);

        let exorcism = ability("exorcism");
        assert_eq!(exorcism.targeting, Targeting::Toughest);

        let starfire = ability("starfire");
        assert_eq!(starfire.targeting, Targeting::Toughest);
    }

    #[test]
    fn targeting_defaults_to_nearest() {
        // Regression test for P2-001: verify that ordinary abilities default to Nearest targeting
        let mortal_strike = ability("mortal_strike");
        assert_eq!(mortal_strike.targeting, Targeting::Nearest);

        let sinister_strike = ability("sinister_strike");
        assert_eq!(sinister_strike.targeting, Targeting::Nearest);
    }
}
