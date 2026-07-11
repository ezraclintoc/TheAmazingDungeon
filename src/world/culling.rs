use bevy::prelude::*;
use std::collections::HashSet;

use super::types::*;

/// Returns indices from `spawned` whose position is beyond `cull_dist` from `cam_pos`.
pub fn rooms_to_cull(spawned: &[(usize, Vec2)], cam_pos: Vec2, cull_dist: f32) -> Vec<usize> {
    spawned
        .iter()
        .filter(|(_, pos)| pos.distance(cam_pos) > cull_dist)
        .map(|(index, _)| *index)
        .collect()
}

/// Returns indices from `all_rooms` within `respawn_dist` of `cam_pos` that aren't
/// already in `reserved` - every index that's either already spawned as an entity or
/// already queued to spawn one, so a room can't get queued a second time while it's
/// still waiting in the spawn queue. `respawn_dist` should be smaller than the cull
/// distance used by `rooms_to_cull` (hysteresis), or a room right at the boundary would
/// despawn and respawn every tick.
pub fn rooms_to_respawn(
    all_rooms: &[(usize, Vec2)],
    reserved: &HashSet<usize>,
    cam_pos: Vec2,
    respawn_dist: f32,
) -> Vec<usize> {
    all_rooms
        .iter()
        .filter(|(index, pos)| !reserved.contains(index) && pos.distance(cam_pos) <= respawn_dist)
        .map(|(index, _)| *index)
        .collect()
}

pub fn cull_and_respawn_rooms(
    time: Res<Time>,
    mut timer: ResMut<CullTimer>,
    camera: Query<&GlobalTransform, With<Camera2d>>,
    spawned_rooms: Query<(Entity, &SpawnedRoom)>,
    state: Res<WorldState>,
    config: Res<GenerationConfig>,
    mut spawn_queue: ResMut<SpawnQueue>,
    mut despawn_queue: ResMut<DespawnQueue>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(cam) = camera.single() else {
        return;
    };
    let cam_pos = cam.translation().truncate();

    let spawned_entities: Vec<(Entity, usize)> =
        spawned_rooms.iter().map(|(entity, marker)| (entity, marker.0)).collect();

    let spawned_positions: Vec<(usize, Vec2)> = spawned_entities
        .iter()
        .map(|(_, index)| (*index, state.rooms[*index].world_pos))
        .collect();

    let cull_set: HashSet<usize> =
        rooms_to_cull(&spawned_positions, cam_pos, config.cull_dist).into_iter().collect();

    for (entity, index) in &spawned_entities {
        if cull_set.contains(index) {
            despawn_queue.0.push_back(*entity);
        }
    }

    // reserved = already spawned or already waiting in the spawn queue - a room in
    // neither state yet is fair game to queue for respawn
    let mut reserved: HashSet<usize> = spawned_entities.iter().map(|(_, i)| *i).collect();
    reserved.extend(spawn_queue.0.iter().map(|(index, _)| *index));

    let all_rooms: Vec<(usize, Vec2)> =
        state.rooms.iter().enumerate().map(|(i, r)| (i, r.world_pos)).collect();
    let respawn_dist = config.cull_dist * 0.8;

    for index in rooms_to_respawn(&all_rooms, &reserved, cam_pos, respawn_dist) {
        spawn_queue.0.push_back((index, state.rooms[index].clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn culls_rooms_beyond_cull_dist() {
        let spawned = vec![(0, Vec2::new(0.0, 0.0)), (1, Vec2::new(500.0, 0.0))];
        let culled = rooms_to_cull(&spawned, Vec2::ZERO, 100.0);
        assert_eq!(culled, vec![1]);
    }

    #[test]
    fn keeps_rooms_within_cull_dist() {
        let spawned = vec![(0, Vec2::new(50.0, 0.0))];
        let culled = rooms_to_cull(&spawned, Vec2::ZERO, 100.0);
        assert!(culled.is_empty());
    }

    #[test]
    fn respawns_known_rooms_back_within_range() {
        let all_rooms = vec![(0, Vec2::ZERO), (1, Vec2::new(500.0, 0.0))];
        let spawned = HashSet::new();
        let respawn = rooms_to_respawn(&all_rooms, &spawned, Vec2::ZERO, 100.0);
        assert_eq!(respawn, vec![0]);
    }

    #[test]
    fn does_not_respawn_reserved_rooms() {
        // reserved covers both already-spawned rooms and rooms merely queued to spawn -
        // the pure function doesn't distinguish (the caller builds the union)
        let all_rooms = vec![(0, Vec2::ZERO)];
        let mut reserved = HashSet::new();
        reserved.insert(0);
        let respawn = rooms_to_respawn(&all_rooms, &reserved, Vec2::ZERO, 100.0);
        assert!(respawn.is_empty());
    }

    #[test]
    fn hysteresis_gap_is_neither_culled_nor_respawned() {
        // exactly between respawn_dist (80.0) and cull_dist (100.0)
        let pos = Vec2::new(90.0, 0.0);
        let spawned = vec![(0, pos)];
        assert!(rooms_to_cull(&spawned, Vec2::ZERO, 100.0).is_empty());

        let all_rooms = vec![(0, pos)];
        let empty_spawned = HashSet::new();
        assert!(rooms_to_respawn(&all_rooms, &empty_spawned, Vec2::ZERO, 80.0).is_empty());
    }
}
