use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};
use rand::RngExt;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::util::*;
use crate::world::LdtkHandle;

const MAX_ROOMS: usize = 1000;
const MAX_ROOMS_PER_FRAME: usize = 10;

pub fn create_room_index(
    projects: Query<&LdtkProjectHandle>,
    project_assets: Res<Assets<LdtkProject>>,
    mut room_idx: ResMut<RoomIndex>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
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
                doors.push(DoorDef {
                    x: entity.grid.x,
                    y: entity.grid.y,
                    width: if entity.width == 16 {
                        entity.height / 16
                    } else {
                        entity.width / 16
                    },
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
            width: level.px_wid,
            height: level.px_hei,
            offset_x: level.world_x as f32,
            offset_y: level.world_y as f32,
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
    *done = true;
    info!("Room index built: {} rooms", room_idx.rooms.len());
}

fn place_room(
    room: &RoomDef,
    world_x: f32,
    world_y: f32,
    connecting_door: &DoorDef,
    commands: &mut Commands,
    world_state: &mut ResMut<WorldState>,
    ldtk_handle: &Handle<LdtkProject>,
) {
    let level_set = LevelSet::from_iids([room.iid.clone()]);
    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: ldtk_handle.clone().into(),
            level_set,
            transform: Transform::from_xyz(
                world_x - room.offset_x as f32,
                world_y + room.offset_y as f32,
                50.0,
            ),
            ..default()
        },
    ));
    world_state.rooms.push(Room {
        world_x: world_x as f32,
        world_y: world_y as f32,
        room: room.clone(),
    });

    for door in &room.doors {
        if door.x == connecting_door.x && door.y == connecting_door.y {
            continue;
        }
        world_state.open_doors.push(Door {
            door: door.clone(),
            world_x: world_x + (door.x * 16) as f32 + door.dir.door_offset(door.width as f32).x,
            world_y: world_y + (-door.y * 16) as f32 + door.dir.door_offset(door.width as f32).y,
        });
    }
}

pub fn generation_loop(
    mut commands: Commands,
    mut world_rng: ResMut<WorldRng>,
    mut world_state: ResMut<WorldState>,
    placed_rooms: Query<(&LevelIid, &GlobalTransform)>,
    room_idx: Res<RoomIndex>,
    ldtk_handle: Res<LdtkHandle>,
    camera: Query<&GlobalTransform, With<Camera2d>>,
    mut gizmos: Gizmos,
) {
    //Check if room_idx has been initiailized
    if room_idx.rooms.is_empty() {
        return;
    }

    //Generate spawn room
    if !&world_state.initialized {
        let spawn_room = room_idx
            .rooms
            .iter()
            .find(|rd| rd.room_type == RoomType::Spawn)
            .unwrap_or(&room_idx.rooms[0]);

        place_room(
            spawn_room,
            0.0,
            0.0,
            &DoorDef {
                x: 0,
                y: 0,
                width: 0,
                dir: Dir::N,
            },
            &mut commands,
            &mut world_state,
            &ldtk_handle.0,
        );

        world_state.initialized = true;
    }

    if placed_rooms.iter().len() >= MAX_ROOMS {
        warn!("Too many rooms! Over {} rooms!", MAX_ROOMS);
        return;
    }

    let Ok(cam_gt) = camera.single() else {
        error!("Camera not found!");
        return;
    };
    let cam_pos = cam_gt.translation().truncate();

    let mut nearby_doors: Vec<usize> = (0..world_state.open_doors.len())
        .filter(|&i| {
            let door = &world_state.open_doors[i];
            Vec2::new(door.world_x, door.world_y).distance(cam_pos) <= 200.0
        })
        .collect();

    nearby_doors.sort_by(|&a, &b| {
        let door_a = &world_state.open_doors[a];
        let door_b = &world_state.open_doors[b];
        let dist_a = Vec2::new(door_a.world_x, door_a.world_y).distance(cam_pos);
        let dist_b = Vec2::new(door_b.world_x, door_b.world_y).distance(cam_pos);
        dist_a.partial_cmp(&dist_b).unwrap()
    });

    let rng = &mut world_rng.0;
    let mut filled_doors = Vec::new();


    for door_idx in nearby_doors {
        if filled_doors.len() >= MAX_ROOMS_PER_FRAME {
            break;
        }

        let door = world_state.open_doors[door_idx].clone();
        let dir = door.door.dir;

        let Some(room_indices) = room_idx.by_door_dir.get(&dir.opposite()) else {
            world_state.open_doors.remove(door_idx);
            continue;
        };

        let mut candidates: Vec<usize> = room_indices
            .iter()
            .flat_map(|&i| {
                let weight = (room_idx.rooms[i].weight * 10.0).round() as usize;
                std::iter::repeat(i).take(weight.max(1))
            })
            .collect();

        candidates.shuffle(rng);

        let mut tried: HashSet<usize> = HashSet::new();
        for &room_idx_pick in &candidates {
            if tried.contains(&room_idx_pick) {
                continue;
            }
            tried.insert(room_idx_pick);

            let room = &room_idx.rooms[room_idx_pick];
            let mut matching_door_idx = 0;
            let Some(matching_door) = room.doors.iter().find(|d| {
                matching_door_idx += 1;
                d.dir == dir.opposite()
            }) else {
                continue;
            };

            let matching_door_vec = Vec2::new(matching_door.x as f32, -matching_door.y as f32)
                * 16.0
                + dir.door_offset(door.door.width as f32);
            let room_world_pos =
                -matching_door_vec + Vec2::new(door.world_x, door.world_y) + dir.as_vec() * 16.0;

            if !check_room_bounds(room, room_world_pos.x, room_world_pos.y, &world_state, 0.0) {
                continue;
            }

            place_room(
                room,
                room_world_pos.x,
                room_world_pos.y,
                matching_door,
                &mut commands,
                &mut world_state,
                &ldtk_handle.0,
            );

            filled_doors.push(door_idx);

            break;
        }
    }

    filled_doors.sort();
    filled_doors.reverse();
    filled_doors.iter().for_each(|idx| {world_state.open_doors.swap_remove(*idx); } );
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
    let right = world_x + room.width as f32;
    let top = world_y;
    let bottom = world_y - room.height as f32;

    for placed in &world_state.rooms {
        let p_left = placed.world_x;
        let p_right = placed.world_x + placed.room.width as f32;
        let p_top = placed.world_y;
        let p_bottom = placed.world_y - placed.room.height as f32;

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

fn check_door_collision(room: &RoomDef, world_x: f32, world_y: f32, world_state: &WorldState) {
    todo!();
}
