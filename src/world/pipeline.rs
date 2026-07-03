use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};
use rand::RngExt;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::types::*;
use crate::world::LdtkHandle;

const MAX_ROOMS: usize = 1000;
pub const CAMERA_SPAWN_DIST: f32 = 1000.0;

pub fn is_ldtk_loaded(
    asset_server: Res<AssetServer>,
    ldtk_handle: Res<LdtkHandle>,
    mut next_state: ResMut<NextState<GenerationState>>,
) {
    // Check if the specific LDtk file has finished loading into memory
    if asset_server.is_loaded_with_dependencies(&ldtk_handle.0) {
        next_state.set(GenerationState::Indexing);
    }
}

pub fn create_room_index(
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
    mut room_idx: ResMut<RoomIndex>,
    mut next_state: ResMut<NextState<GenerationState>>,
) {
    let Ok(handle) = projects.single() else {
        return;
    };
    let Some(project) = project_assets.get(handle) else {
        return;
    };

    let mut rooms = Vec::new();

    for level in project.iter_raw_levels() {
        let mut doors = Vec::new();
        let Some(layers) = &level.layer_instances else {
            continue;
        };

        for layer in layers {
            if layer.identifier != "Entities" {
                continue;
            }
            for entity in &layer.entity_instances {
                if entity.identifier != "Door" {
                    continue;
                }
                let dir = if entity.width != 16 {
                    if entity.grid.y == 0 { Dir::N } else { Dir::S }
                } else {
                    if entity.grid.x == 0 { Dir::W } else { Dir::E }
                };
                let width = if entity.width == 16 {
                    entity.height / 16
                } else {
                    entity.width / 16
                };

                doors.push(DoorDef {
                    local_pos: entity.grid * IVec2::new(1, -1)
                        + if dir == Dir::S {
                            IVec2::new(1, 0)
                        } else if dir == Dir::W {
                            IVec2::new(0, 0)
                        } else if dir == Dir::N {
                            IVec2::new(1, 0)
                        } else {
                            IVec2::new(1,0)
                        },
                    width: width,
                    dir,
                });
            }
        }

        let weight = level
            .field_instances
            .iter()
            .find(|f| f.identifier == "weight")
            .and_then(|f| match &f.value {
                FieldValue::Float(Some(v)) => Some(*v),
                _ => None,
            })
            .unwrap_or(1.0);

        let room_type = RoomType::from_str(
            level
                .field_instances
                .iter()
                .find(|f| f.identifier == "room_type")
                .and_then(|f| match &f.value {
                    FieldValue::Enum(Some(v)) => Some(v.clone()),
                    _ => {
                        warn!("Invalid type");
                        None
                    }
                })
                .unwrap_or_else(|| "Hallway".to_owned())
                .as_str(),
        )
        .unwrap_or(RoomType::Hallway);

        rooms.push(RoomDef {
            iid: level.iid.clone(),
            size: IVec2::new(level.px_wid, level.px_hei),
            offset: Vec2::new(level.world_x as f32, level.world_y as f32),
            doors,
            weight,
            room_type,
        });
    }

    // build the direction index
    let mut by_door_dir: HashMap<Dir, Vec<usize>> = HashMap::new();
    for (i, room) in rooms.iter().enumerate() {
        for door in &room.doors {
            by_door_dir.entry(door.dir).or_default().push(i);
        }
    }

    room_idx.rooms = rooms;
    room_idx.by_door_dir = by_door_dir;
    info!("Room index built: {} rooms", room_idx.rooms.len());
    next_state.set(GenerationState::Ready);
}

pub fn spawn_if_idle(
    mut task: ResMut<GenTask>,
    state: Res<WorldState>,
    camera: Query<&GlobalTransform, With<Camera2d>>,
    room_idx: Res<RoomIndex>,
) {
    if task.0.is_some() {
        return;
    }

    let doors = state.open_doors.clone();
    let rooms = state.rooms.clone();

    let pool = AsyncComputeTaskPool::get();

    let cam_pos = camera
        .single()
        .unwrap_or(&GlobalTransform::default())
        .translation()
        .truncate();

    let room_idx = room_idx.clone();

    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

    task.0 = Some(pool.spawn(async move {
        let mut placed_rooms: Vec<Room> = Vec::new();

        if rooms.is_empty() {
            let mut spawn_rooms: Vec<&RoomDef> = room_idx
                .rooms
                .iter()
                .filter(|rd| rd.room_type == RoomType::Spawn)
                .collect();
            spawn_rooms.shuffle(&mut rng);

            placed_rooms.push(Room::new(
                *spawn_rooms.first().expect("No spawn room found."),
                Vec2::ZERO,
            ));
        } else {
            let mut nearby_doors: Vec<usize> = (0..doors.len())
                .filter(|&i| {
                    let door = &doors[i];
                    door.world_pos.distance(cam_pos) <= CAMERA_SPAWN_DIST
                })
                .collect();

            nearby_doors.sort_by(|&a, &b| {
                let door_a = &doors[a];
                let door_b = &doors[b];
                let dist_a = Vec2::new(door_a.world_pos.x, door_a.world_pos.y).distance(cam_pos);
                let dist_b = Vec2::new(door_b.world_pos.x, door_b.world_pos.y).distance(cam_pos);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

            for door_idx in nearby_doors {
                if rooms.len() + placed_rooms.len() >= MAX_ROOMS {
                    break;
                }

                let door = &doors[door_idx];
                let dir = door.door.dir;

                let Some(room_indices) = room_idx.by_door_dir.get(&dir.opposite()) else {
                    continue;
                };

                let mut candidates: Vec<usize> = room_indices
                    .iter()
                    .flat_map(|&i| {
                        let weight = (room_idx.rooms[i].weight * 10.0).round() as usize;
                        std::iter::repeat(i).take(weight.max(1))
                    })
                    .collect();

                candidates.shuffle(&mut rng);

                let mut tried: HashSet<usize> = HashSet::new();
                for &room_idx_pick in &candidates {
                    if tried.contains(&room_idx_pick) {
                        continue;
                    }
                    tried.insert(room_idx_pick);

                    let room = &room_idx.rooms[room_idx_pick];
                    let Some(matching_door) = room.doors.iter().find(|d| d.dir == dir.opposite())
                    else {
                        continue;
                    };

                    let matching_door_vec = (matching_door.local_pos.as_vec2() - dir.as_vec()) * 16.0;
                    let room_world_pos = door.world_pos - matching_door_vec + if dir == Dir::S { Vec2::new(0.0, 16.0)} else if dir == Dir::E { Vec2::new(-16.0, 16.0) } else if dir == Dir::W { Vec2::new(16.0, 16.0) } else { Vec2::ZERO };

                    if !check_room_bounds(
                        &room,
                        room_world_pos.x,
                        room_world_pos.y,
                        &WorldState {
                            open_doors: doors.clone(),
                            rooms: rooms.clone(),
                        },
                        0.0,
                    ) {
                        continue;
                    }

                    let r = Room::new(&room, room_world_pos);

                    //Makes sure all doors (of the room we are about to place) can go forward
                    if !check_door_collision(
                        &r,
                        &door,
                        &WorldState {
                            open_doors: doors.clone(),
                            rooms: rooms.clone(),
                        },
                    ) {
                        continue;
                    }

                    placed_rooms.push(r);
                    break;
                }
            }
        }
        placed_rooms
    }));
}

pub fn poll_task(
    mut task: ResMut<GenTask>,
    mut state: ResMut<WorldState>,
    mut commands: Commands,
    ldtk_handle: Res<LdtkHandle>,
) {
    let Some(t) = &mut task.0 else {
        return;
    };

    if let Some(new_rooms) = block_on(poll_once(t)) {
        for room in new_rooms {
            let level_set = LevelSet::from_iids([room.room.iid.clone()]);
            commands.spawn((LdtkWorldBundle {
                ldtk_handle: ldtk_handle.0.clone().into(),
                level_set,
                transform: Transform::from_xyz(
                    room.world_pos.x - room.room.offset.x as f32,
                    room.world_pos.y + room.room.offset.y as f32,
                    50.0,
                ),
                ..default()
            },));

            state.rooms.push(room.clone());

            for doordef in &room.room.doors {
                let door = Door::new(&room, doordef);

                let mut found_matching_door_idx: usize = 0;
                let mut found_matching_door = false;

                for d in &state.open_doors {
                    found_matching_door_idx += 1;

                    if door.world_pos == d.world_pos {
                        found_matching_door_idx -= 1;
                        break;
                    }
                }

                if found_matching_door_idx < state.open_doors.len() {
                    state.open_doors.swap_remove(found_matching_door_idx);
                } else {
                    state.open_doors.push(door);
                }
            }
        }
        task.0 = None;
    }
}

fn place_room(roomdef: &RoomDef, world_pos: Vec2, world_state: &mut ResMut<WorldState>) {
    world_state.rooms.push(Room {
        world_pos,
        room: roomdef.clone(),
    });

    let room = world_state.rooms.last().unwrap().clone();
    for doordef in &roomdef.doors {
        let door = Door::new(&room, doordef);

        let mut found_matching_door_idx: usize = 0;
        let mut found_matching_door = false;

        for dd in &world_state.open_doors {
            found_matching_door_idx += 1;

            if door.world_pos == dd.world_pos {
                found_matching_door_idx -= 1;
                break;
            }
        }

        if found_matching_door_idx < world_state.open_doors.len() {
            world_state.open_doors.swap_remove(found_matching_door_idx);
        } else {
            world_state.open_doors.push(door);
        }
    }
}

fn check_room_bounds(
    room: &RoomDef,
    world_x: f32,
    world_y: f32,
    world_state: &WorldState,
    gap: f32,
) -> bool {
    // candidate corners: top-left is (world_x, world_y), bottom-right is right and down
    let left = world_x;
    let right = world_x + room.size.x as f32;
    let top = world_y;
    let bottom = world_y - room.size.y as f32;

    for placed in &world_state.rooms {
        let p_left = placed.room.size.x as f32;
        let p_right = (placed.room.size.x + placed.room.size.x) as f32;
        let p_top = placed.room.size.y as f32;
        let p_bottom = (placed.room.size.y - placed.room.size.y) as f32;

        let overlaps = left - gap < p_right
            && right + gap > p_left
            && bottom - gap < p_top
            && top + gap > p_bottom;
        if overlaps {
            return false;
        }
    }
    true
}

pub fn check_door_collision(room: &Room, connecting_door: &Door, world_state: &WorldState) -> bool {
    let Some(matching_door) = room
        .room
        .doors
        .iter()
        .find(|d| d.dir == connecting_door.door.dir.opposite())
    else {
        return false;
    };

    let mut collision = false;
    for dd in &room.room.doors {
        if dd == matching_door {
            continue;
        }

        let d = Door::new(room, dd);
        if !check_new_door_collision(&d, &world_state) {
            collision = true;
        }
    }
    if collision == true {
        return false;
    }

    if !check_current_door_collision(room, &connecting_door, &world_state) {
        return false;
    }
    true
}

pub fn check_new_door_collision(door: &Door, world_state: &WorldState) -> bool {
    for room in &world_state.rooms {
        if rects_collide(
            door.get_bounding_box().0,
            door.get_bounding_box().1,
            room.world_pos,
            room.room.size.as_vec2(),
        ) {
            let colliding_door_pos = door.world_pos; // + door.door.dir.as_vec() * 16.0;
            let mut found_matching_door = false;
            for dd in &room.room.doors {
                if dd.dir == door.door.dir.opposite() {
                    let door_pos = Vec2::new(
                        room.world_pos.x
                            + (dd.local_pos.x * 16) as f32
                            + dd.dir.door_offset(dd.width as f32).x,
                        room.world_pos.y
                            + (-dd.local_pos.y * 16) as f32
                            + dd.dir.door_offset(dd.width as f32).y,
                    );
                    if door_pos == colliding_door_pos {
                        found_matching_door = true;
                    }
                }
            }
            if !found_matching_door {
                return false;
            }
        }
    }
    true
}

pub fn check_current_door_collision(
    room: &Room,
    connecting_door: &Door,
    world_state: &WorldState,
) -> bool {
    for door in &world_state.open_doors {
        if door == connecting_door {
            continue;
        }

        if rects_collide(
            door.get_bounding_box().0,
            door.get_bounding_box().1,
            room.world_pos,
            room.room.size.as_vec2(),
        ) {
            let colliding_door_pos = door.world_pos; // + door.door.dir.as_vec() * 16.0;
            let mut found_matching_door = false;
            for dd in &room.room.doors {
                if dd.dir == door.door.dir.opposite() {
                    let door_pos = Vec2::new(
                        room.world_pos.x
                            + (dd.local_pos.x * 16) as f32
                            + dd.dir.door_offset(dd.width as f32).x,
                        room.world_pos.y
                            + (-dd.local_pos.y * 16) as f32
                            + dd.dir.door_offset(dd.width as f32).y,
                    );

                    if door_pos == colliding_door_pos {
                        found_matching_door = true;
                    }
                }
            }
            if !found_matching_door {
                return false;
            }
        }
    }
    true
}
