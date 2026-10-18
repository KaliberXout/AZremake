//! Headless simulation of a whole run on a fixed clock.
//!
//! The harness runs the mode's own schedule ([`super::add_run_loop`]), the same run start
//! ([`super::run_lifecycle::start_run`]), hero spawn ([`super::hero::spawn_hero`]), level-up offers
//! ([`super::level_up::roll_offer`]) and card picks ([`super::level_up::pick`]) that the game runs. There is
//! no window, renderer, WoW data or network. The clock steps a fixed amount per frame and the
//! dice are seeded, so a run is repeatable frame for frame.
//!
//! The harness controls three things and nothing else: the seed, the step size, and the card
//! policy (the dice pick one of the offered cards). Invulnerability is an explicit switch, used
//! only to reach the victory path, which a stationary hero would otherwise never finish.

use std::time::Duration;

use bevy::ecs::schedule::ExecutorKind;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use benilla_world::model_fade::DespawnFade;

use super::data;
use super::enemies::{Dying, Enemy};
use super::hero;
use super::level_up::{self, Offer};
use super::run_lifecycle;
use super::{Phase, Rng, Run, Selection};
use crate::combat_text::CombatTextSpawn;
use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
use crate::loading_screen::LoadingScreen;
use crate::net::FieldChanged;

/// One frame of simulated time: 1/20 s, so a 900 s run is 18,000 frames.
pub(super) const STEP_SECS: f32 = 1.0 / 20.0;

/// One recorded frame, after the frame's systems ran.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Sample {
    pub t: f32,
    pub phase: Phase,
    pub level: u32,
    pub hp: f32,
    pub kills: u32,
    pub bosses: u32,
    pub enemies: usize,
    /// The highest roster tier among the enemies alive, 0 with none.
    pub max_tier: usize,
}

/// A boss the run has put on the field, by its roster name, at the time it appeared.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct BossSpawn {
    pub t: f32,
    pub name: &'static str,
}

/// The run's whole recording, for assertions.
#[derive(Debug, PartialEq)]
pub(super) struct Outcome {
    pub samples: Vec<Sample>,
    pub bosses: Vec<BossSpawn>,
    pub picks: u32,
    pub level_ups: u32,
    pub enemy_spawns_after_death: usize,
    pub final_run: FinalRun,
}

/// The end state of the run, copied out of [`Run`].
#[derive(Debug, PartialEq)]
pub(super) struct FinalRun {
    pub elapsed: f32,
    pub won: bool,
    pub died_at: Option<f32>,
    pub level: u32,
    pub kills: u32,
    pub damage_done: f64,
    pub abilities: Vec<(&'static str, u8)>,
    pub passives: Vec<(&'static str, u8)>,
    pub pending_levels: u32,
}

/// Cards the card policy has chosen so far.
#[derive(Resource, Default)]
struct PickCount(u32);

/// The card policy: the dice pick one of the offered cards.
fn choose_card(
    mut run: ResMut<Run>,
    mut offer: ResMut<Offer>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<Phase>>,
    mut picks: ResMut<PickCount>,
) {
    let n = offer.len();
    if n == 0 {
        return;
    }
    let i = rng.index(n);
    level_up::pick(i, &mut run, &mut offer, &mut rng, &mut next);
    picks.0 += 1;
}

/// Stands in for the renderer's corpse teardown, which the game runs in `benilla_world::model_fade`
/// (a crate-private system, render-side and gameplay-inert: no rule reads a corpse). A corpse whose
/// fade has been armed for a second is despawned, so the crowd in the harness is the crowd the game
/// shows, and the loops over enemies cost what they cost in the game.
fn teardown_corpses(
    mut commands: Commands,
    time: Res<Time>,
    faded: Query<Entity, With<DespawnFade>>,
    mut armed: Local<Vec<(Entity, f32)>>,
) {
    let now = time.elapsed_secs();
    armed.retain(|(e, _)| faded.contains(*e));
    for e in &faded {
        if !armed.iter().any(|(s, _)| *s == e) {
            armed.push((e, now));
        }
    }
    armed.retain(|&(e, since)| {
        if now - since >= 1.0 {
            commands.entity(e).despawn();
            false
        } else {
            true
        }
    });
}

pub(super) struct Sim {
    app: App,
    map: usize,
    invulnerable: bool,
    level_ups: u32,
    last_phase: Phase,
    last_bosses: u32,
    samples: Vec<Sample>,
    bosses: Vec<BossSpawn>,
    death_at: Option<f32>,
    enemy_spawns_after_death: usize,
    last_enemy_count: usize,
}

impl Sim {
    /// A run of `class_id` on battleground `map`, its dice seeded with `seed`, started and in play.
    pub(super) fn new(class_id: u8, map: usize, seed: u64, invulnerable: bool) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                STEP_SECS,
            )))
            .init_state::<Phase>()
            .insert_resource(Rng::seeded(seed))
            .init_resource::<Run>()
            .init_resource::<Selection>()
            .init_resource::<super::abilities::PendingHits>()
            .init_resource::<super::enemies::Spawner>()
            .init_resource::<Offer>()
            .init_resource::<PickCount>()
            .init_resource::<PlaySeq>()
            .init_resource::<LoadingScreen>()
            .add_message::<CastEvent>()
            .add_message::<SpellGoTargets>()
            .add_message::<SwingMessage>()
            .add_message::<CombatTextSpawn>()
            .add_message::<FieldChanged>();
        benilla_world::world_point::init_world_point_resources(app.world_mut());

        // One thread, so conflicting systems run in their declared order every frame.
        app.edit_schedule(Update, |s| {
            s.set_executor_kind(ExecutorKind::SingleThreaded);
        });
        app.edit_schedule(StateTransition, |s| {
            s.set_executor_kind(ExecutorKind::SingleThreaded);
        });
        super::add_run_loop(&mut app);
        app.add_systems(
            Update,
            (
                choose_card
                    .run_if(in_state(Phase::LevelUp))
                    .after(hero::hero_vitals),
                teardown_corpses.after(hero::hero_vitals),
            ),
        );

        // The menu's run start: the class and look are picked, then the run begins.
        let mut selection = std::mem::take(&mut *app.world_mut().resource_mut::<Selection>());
        let mut run = std::mem::take(&mut *app.world_mut().resource_mut::<Run>());
        let class = data::CLASSES
            .iter()
            .position(|c| c.id == class_id)
            .expect("a class with that id");
        selection.class = Some(class);
        selection.looks = vec![(data::CLASSES[class].looks[0], 1, 0)];
        selection.look = Some(0);
        selection.map = map;
        assert!(
            run_lifecycle::start_run(&mut run, &mut selection),
            "a run starts from a picked class and look"
        );
        let (display, race, sex) = selection.looks[0];
        let pos = hero::arena(selection.stage_def());
        let serial = run.guid();
        *app.world_mut().resource_mut::<Run>() = run;
        *app.world_mut().resource_mut::<Selection>() = selection;

        let mut queue = CommandQueue::default();
        {
            let world = app.world();
            let mut commands = Commands::new(&mut queue, world);
            hero::spawn_hero(
                &mut commands,
                &data::CLASSES[class],
                display,
                race,
                sex,
                serial,
                pos,
            );
        }
        queue.apply(app.world_mut());
        app.world_mut()
            .resource_mut::<NextState<Phase>>()
            .set(Phase::Playing);

        app.finish();
        app.cleanup();
        let last_phase = *app.world().resource::<State<Phase>>().get();
        let mut sim = Sim {
            app,
            map,
            invulnerable,
            level_ups: 0,
            last_phase,
            last_bosses: 0,
            samples: Vec::new(),
            bosses: Vec::new(),
            death_at: None,
            enemy_spawns_after_death: 0,
            last_enemy_count: 0,
        };
        sim.step();
        sim
    }

    /// One frame: the invulnerability switch, the schedule, then the recording.
    pub(super) fn step(&mut self) {
        if self.invulnerable {
            self.app.world_mut().resource_mut::<Run>().immune_t = f32::MAX;
        }
        self.app.update();

        let phase = self.phase();
        if phase == Phase::LevelUp && self.last_phase != Phase::LevelUp {
            self.level_ups += 1;
        }
        self.last_phase = phase;

        let run = self.app.world().resource::<Run>();
        let (t, level, hp, kills, bosses, dead) = (
            run.elapsed,
            run.level,
            run.hp,
            run.kills,
            run.bosses,
            run.dead_t.is_some(),
        );
        if bosses > self.last_bosses {
            let roster = data::roster(self.map);
            for i in self.last_bosses..bosses {
                let name = roster.bosses[i as usize].name;
                self.bosses.push(BossSpawn { t, name });
            }
            self.last_bosses = bosses;
        }
        let enemies = self
            .app
            .world_mut()
            .query_filtered::<&Enemy, Without<Dying>>()
            .iter(self.app.world())
            .filter(|e| !e.dead)
            .count();
        // A death recorded this frame: the spawns before it, in the same frame, were the game's.
        if self.death_at.is_some() && enemies > self.last_enemy_count {
            self.enemy_spawns_after_death += enemies - self.last_enemy_count;
        }
        if dead && self.death_at.is_none() {
            self.death_at = Some(t);
        }
        self.last_enemy_count = enemies;
        let max_tier = self.highest_tier();
        self.samples.push(Sample {
            t,
            phase,
            level,
            hp,
            kills,
            bosses,
            enemies,
            max_tier,
        });
    }

    pub(super) fn phase(&self) -> Phase {
        *self.app.world().resource::<State<Phase>>().get()
    }

    /// Frames until the run reaches the results screen, or `max_frames` pass.
    pub(super) fn run_to_end(&mut self, max_frames: usize) {
        for _ in 0..max_frames {
            if self.phase() == Phase::GameOver {
                return;
            }
            self.step();
        }
    }

    /// The highest roster tier among the enemies alive now, 0 with none. Enemy definitions are
    /// `'static` entries of the battleground's roster, so identity is the tier's identity.
    fn highest_tier(&mut self) -> usize {
        let roster = data::roster(self.map);
        let defs: Vec<&'static data::EnemyDef> = self
            .app
            .world_mut()
            .query_filtered::<&Enemy, Without<Dying>>()
            .iter(self.app.world())
            .filter(|e| !e.dead)
            .map(|e| e.def)
            .collect();
        defs.into_iter()
            .filter_map(|def| roster.enemies.iter().position(|d| std::ptr::eq(d, def)))
            .max()
            .unwrap_or(0)
    }

    pub(super) fn finish(self) -> Outcome {
        let run = self.app.world().resource::<Run>();
        let final_run = FinalRun {
            elapsed: run.elapsed,
            won: run.won,
            died_at: run.dead_t.map(|_| self.death_at.unwrap_or(run.elapsed)),
            level: run.level,
            kills: run.kills,
            damage_done: run.damage_done,
            abilities: run.abilities.iter().map(|o| (o.def.key, o.level)).collect(),
            passives: run.passives.iter().map(|(p, l)| (p.key, *l)).collect(),
            pending_levels: run.pending_levels,
        };
        let picks = self.app.world().resource::<PickCount>().0;
        Outcome {
            samples: self.samples,
            bosses: self.bosses,
            picks,
            level_ups: self.level_ups,
            enemy_spawns_after_death: self.enemy_spawns_after_death,
            final_run,
        }
    }
}

#[cfg(test)]
mod scenarios {
    use super::*;

    /// A Mage on The Barrens: the class and battleground the harness's own checks are written for.
    const MAGE: u8 = 8;
    const BARRENS: usize = 0;
    const SEED: u64 = 1;

    /// Frames for `secs` of simulated time, with a margin for the frame the end lands on.
    fn frames_for(secs: f32) -> usize {
        (secs / STEP_SECS).ceil() as usize + 5
    }

    fn run(seed: u64, invulnerable: bool, secs: f32) -> Outcome {
        let mut sim = Sim::new(MAGE, BARRENS, seed, invulnerable);
        // A margin past `secs`: the results screen comes a frame or two after the last second.
        sim.run_to_end(frames_for(secs + 5.0));
        sim.finish()
    }

    #[test]
    fn a_full_900_second_run_reaches_victory_on_the_game_schedule() {
        let o = run(SEED, true, data::VICTORY_SECS);
        let f = &o.final_run;
        assert_eq!(
            o.samples.last().map(|s| s.phase),
            Some(Phase::GameOver),
            "the run ends on the results screen"
        );
        assert!(f.won, "the run is won");
        assert_eq!(f.died_at, None);
        assert!(
            f.elapsed >= data::VICTORY_SECS && f.elapsed < data::VICTORY_SECS + 2.0 * STEP_SECS,
            "victory lands on the first frame at or past {}s, not {}",
            data::VICTORY_SECS,
            f.elapsed
        );

        // Bosses: every roster boss, in order, each on its own clock, none early.
        let roster = data::roster(BARRENS);
        let names: Vec<&str> = o.bosses.iter().map(|b| b.name).collect();
        let expected: Vec<&str> = roster.bosses.iter().map(|b| b.name).collect();
        assert_eq!(names, expected, "the bosses come in roster order");
        for (k, b) in o.bosses.iter().enumerate() {
            let due = (k + 1) as f32 * data::BOSS_EVERY_SECS;
            assert!(
                b.t >= due && b.t < due + 3.0 * STEP_SECS,
                "boss {} at {:.2}s, due at {due}s",
                b.name,
                b.t
            );
        }

        // Experience and cards: one card per level gained after the first, none left owed.
        assert_eq!(f.pending_levels, 0);
        assert_eq!(o.picks, f.level - 1, "one card per level gained");
        assert!(
            o.level_ups <= o.picks,
            "a level-up screen takes at least one card"
        );
        assert!(o.samples.windows(2).all(|w| w[1].level >= w[0].level));

        // The clock stops while cards are up: consecutive level-up frames share a time.
        for w in o.samples.windows(2) {
            if w[0].phase == Phase::LevelUp && w[1].phase == Phase::LevelUp {
                assert_eq!(w[0].t, w[1].t, "the clock is paused on the level-up screen");
            }
        }

        // Wave progression: the opening crowd is the first tiers, the closing crowd the last.
        let early = o
            .samples
            .iter()
            .filter(|s| s.t <= 60.0)
            .map(|s| s.max_tier)
            .max()
            .unwrap_or(0);
        let late = o
            .samples
            .iter()
            .filter(|s| s.t >= data::VICTORY_SECS - 60.0)
            .map(|s| s.max_tier)
            .max()
            .unwrap_or(0);
        assert!(
            early <= 3,
            "the first minute stays in the opening tiers: {early}"
        );
        assert!(late >= 27, "the last minute reaches the top tiers: {late}");

        // Combat and the slot limits.
        assert!(f.kills > 1_000, "a crowd is killed: {}", f.kills);
        assert!(f.damage_done > 0.0);
        assert!(o.samples.windows(2).all(|w| w[1].kills >= w[0].kills));
        assert!(f.abilities.len() <= data::MAX_ABILITIES);
        assert!(f.passives.len() <= data::MAX_PASSIVES);
        assert!(f
            .abilities
            .iter()
            .all(|(_, l)| *l <= data::MAX_ABILITY_LEVEL));
        assert!(
            f.abilities.iter().any(|(_, l)| *l >= 2),
            "upgrades are taken on the way"
        );
    }

    #[test]
    fn the_same_seed_replays_the_same_victory_frame_for_frame() {
        let a = run(SEED, true, data::VICTORY_SECS);
        let b = run(SEED, true, data::VICTORY_SECS);
        assert_eq!(a, b);
    }

    #[test]
    fn a_hero_who_does_not_survive_ends_the_run_in_defeat() {
        let o = run(SEED, false, data::VICTORY_SECS);
        let f = &o.final_run;
        assert!(!f.won, "a defeat is not a victory");
        let died = f.died_at.expect("the hero falls before the clock runs out");
        assert!(died < data::VICTORY_SECS, "defeat at {died}s");
        assert_eq!(f.elapsed, died, "the clock stops at the death");
        assert!(o.samples.iter().any(|s| s.hp == 0.0), "health reaches zero");
        assert_eq!(
            o.samples.last().map(|s| s.phase),
            Some(Phase::GameOver),
            "the results screen follows the death"
        );
        // The death plays out for three seconds of frames before the results screen.
        let playing_after_death = o
            .samples
            .iter()
            .filter(|s| s.t == died && s.phase == Phase::Playing)
            .count();
        assert!(
            (60..=63).contains(&playing_after_death),
            "the death plays for about three seconds: {playing_after_death} frames"
        );
        assert_eq!(
            o.enemy_spawns_after_death, 0,
            "no crowd arrives after the death"
        );
        assert!(
            o.bosses.iter().all(|b| b.t < died),
            "every boss came before the death"
        );
        assert_eq!(o.picks, f.level - 1);
    }

    #[test]
    fn the_seed_decides_the_run() {
        let a = run(1, false, 120.0);
        let b = run(2, false, 120.0);
        assert_ne!(a.samples, b.samples, "two seeds play two runs");
    }
}
