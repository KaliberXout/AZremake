use bevy::prelude::*;

use super::data::{self, AbilityDef, PassiveDef};
use super::{Owned, Phase, Rng, Run};

#[derive(Clone, Copy)]
pub(super) enum Choice {
    NewAbility(&'static AbilityDef),
    Upgrade(usize),
    NewPassive(&'static PassiveDef),
    UpPassive(usize),
    Heal,
}

/// The level-up cards on offer.
#[derive(Resource, Default)]
pub(super) struct Offer {
    pub(super) choices: Vec<Choice>,
}

impl Offer {
    pub(super) fn len(&self) -> usize {
        self.choices.len()
    }

    pub(super) fn choices(&self) -> &[Choice] {
        &self.choices
    }
}

pub(super) fn choices_for(run: &Run, rng: &mut Rng) -> Vec<Choice> {
    let mut pool: Vec<(Choice, f32)> = Vec::new();
    for (i, o) in run.abilities.iter().enumerate() {
        if o.level < data::MAX_ABILITY_LEVEL {
            pool.push((Choice::Upgrade(i), 1.2));
        }
    }
    if run.abilities.len() < data::MAX_ABILITIES {
        for key in run.class.pool {
            if !run.abilities.iter().any(|o| o.def.key == *key) {
                pool.push((Choice::NewAbility(data::ability(key)), 1.0));
            }
        }
    }
    for (i, (_, level)) in run.passives.iter().enumerate() {
        if *level < data::MAX_PASSIVE_LEVEL {
            pool.push((Choice::UpPassive(i), 0.8));
        }
    }
    if run.passives.len() < data::MAX_PASSIVES {
        for p in data::PASSIVES {
            if !run.passives.iter().any(|(q, _)| q.key == p.key) {
                pool.push((Choice::NewPassive(p), 0.5));
            }
        }
    }
    let mut out = Vec::with_capacity(3);
    while out.len() < 3 && !pool.is_empty() {
        let total: f32 = pool.iter().map(|(_, w)| w).sum();
        let mut pick = rng.f32() * total;
        let mut idx = pool.len() - 1;
        for (i, (_, w)) in pool.iter().enumerate() {
            if pick < *w {
                idx = i;
                break;
            }
            pick -= w;
        }
        out.push(pool.swap_remove(idx).0);
    }
    if out.is_empty() {
        out.push(Choice::Heal);
    }
    out
}

pub(super) fn roll_offer(run: Res<Run>, mut rng: ResMut<Rng>, mut offer: ResMut<Offer>) {
    offer.choices = choices_for(&run, &mut rng);
}

pub(super) fn pick(
    i: usize,
    run: &mut Run,
    offer: &mut Offer,
    rng: &mut Rng,
    next: &mut NextState<Phase>,
) {
    let Some(&choice) = offer.choices.get(i) else {
        return;
    };
    match choice {
        Choice::NewAbility(def) => run.abilities.push(Owned {
            def,
            level: 1,
            cd: 0.3,
        }),
        Choice::Upgrade(k) => run.abilities[k].level += 1,
        Choice::NewPassive(p) => run.passives.push((p, 1)),
        Choice::UpPassive(k) => run.passives[k].1 += 1,
        Choice::Heal => run.hp = run.stats().max_hp,
    }
    run.version += 1;
    run.pending_levels = run.pending_levels.saturating_sub(1);
    if run.pending_levels > 0 {
        offer.choices = choices_for(run, rng);
    } else {
        offer.choices.clear();
        next.set(Phase::Playing);
    }
}
