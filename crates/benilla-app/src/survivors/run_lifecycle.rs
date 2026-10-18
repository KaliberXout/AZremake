use bevy::prelude::*;

use super::abilities;
use super::data;
use super::enemies;
use super::{Owned, Run, RunEntity, Selection};

pub(super) fn start_run(run: &mut Run, selection: &mut Selection) -> bool {
    let (Some(class), Some(&(_, race, _))) = (
        selection.class_def(),
        selection.look.and_then(|i| selection.looks.get(i)),
    ) else {
        return false;
    };
    // The world goes to the battleground picked; until now it stood in the last one.
    selection.stage = selection.map;
    let next_guid = run.next_guid;
    *run = Run {
        class,
        race,
        next_guid,
        ..Run::default()
    };
    run.abilities.push(Owned {
        def: data::ability(class.start),
        level: 1,
        cd: 0.5,
    });
    run.hp = run.stats().max_hp;
    run.version += 1;
    true
}

/// Back at the menu: everything the last run spawned goes, and the run starts over.
pub(super) fn reset_run(
    mut commands: Commands,
    spawned: Query<Entity, With<RunEntity>>,
    mut run: ResMut<Run>,
    mut pending: ResMut<abilities::PendingHits>,
    mut spawner: ResMut<enemies::Spawner>,
    mut selection: ResMut<Selection>,
) {
    for e in &spawned {
        commands.entity(e).despawn();
    }
    // Guids stay unique across runs: caches keyed by guid may outlive a despawn by a frame.
    let next_guid = run.next_guid;
    *run = Run {
        next_guid,
        ..Run::default()
    };
    pending.0.clear();
    *spawner = enemies::Spawner::default();
    // The preview hero is respawned for the look still picked.
    selection.version += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection(class_id: u8, map: usize) -> Selection {
        let class = data::CLASSES
            .iter()
            .position(|class| class.id == class_id)
            .expect("class id exists");
        Selection {
            class: Some(class),
            looks: vec![(data::CLASSES[class].looks[0], 1, 0)],
            look: Some(0),
            map,
            ..Selection::default()
        }
    }

    #[test]
    fn starts_a_run_with_its_class_and_starting_ability() {
        let mut run = Run {
            next_guid: 91,
            ..Run::default()
        };
        let mut selection = selection(8, 4);

        assert!(start_run(&mut run, &mut selection));

        assert_eq!(run.class.id, 8);
        assert_eq!(run.race, 1);
        assert_eq!(selection.stage, 4);
        assert_eq!(run.abilities.len(), 1);
        assert_eq!(run.abilities[0].def.key, run.class.start);
        assert_eq!(run.abilities[0].level, 1);
        assert_eq!(run.abilities[0].cd, 0.5);
        assert_eq!(run.hp, run.stats().max_hp);
        assert_eq!(run.version, 1);
    }

    #[test]
    fn failed_run_start_leaves_run_state_unchanged() {
        let mut run = Run {
            elapsed: 42.0,
            hp: 17.0,
            next_guid: 91,
            ..Run::default()
        };
        run.abilities.push(Owned {
            def: data::ability(data::CLASSES[0].start),
            level: 3,
            cd: 0.25,
        });
        let before = (
            run.class.id,
            run.race,
            run.elapsed,
            run.hp,
            run.next_guid,
            run.version,
            run.abilities[0].def.key,
            run.abilities[0].level,
            run.abilities[0].cd,
        );
        let mut selection = Selection {
            class: Some(0),
            stage: 2,
            ..Selection::default()
        };

        assert!(!start_run(&mut run, &mut selection));

        assert_eq!(
            (
                run.class.id,
                run.race,
                run.elapsed,
                run.hp,
                run.next_guid,
                run.version,
                run.abilities[0].def.key,
                run.abilities[0].level,
                run.abilities[0].cd,
            ),
            before
        );
        assert_eq!(selection.stage, 2);
    }

    #[test]
    fn run_start_preserves_the_guid_counter() {
        let mut run = Run {
            next_guid: 1234,
            ..Run::default()
        };

        assert!(start_run(&mut run, &mut selection(8, 0)));

        assert_eq!(run.next_guid, 1234);
    }

    #[test]
    fn reset_run_preserves_guids_and_despawns_run_entities() {
        let mut app = App::new();
        app.insert_resource(Run {
            elapsed: 42.0,
            next_guid: 5678,
            ..Run::default()
        })
        .insert_resource(Selection {
            version: 6,
            ..Selection::default()
        })
        .init_resource::<abilities::PendingHits>()
        .init_resource::<enemies::Spawner>()
        .add_systems(Update, reset_run);
        let entity = app.world_mut().spawn(RunEntity).id();

        app.update();

        let run = app.world().resource::<Run>();
        assert_eq!(run.elapsed, 0.0);
        assert_eq!(run.next_guid, 5678);
        assert_eq!(app.world().resource::<Selection>().version, 7);
        assert!(app.world().get_entity(entity).is_err());
    }
}
