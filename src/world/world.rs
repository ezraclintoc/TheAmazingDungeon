use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};
use rand::RngExt;
use std::collections::HashMap;
use std::str::FromStr;

use super::util::*;
use crate::world::LdtkHandle;

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
                    width: entity.width / 16,
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

    for (dir, indices) in &room_idx.by_door_dir {
        info!("index has {:?} -> {} rooms", dir, indices.len());
    }
}

fn place_room(
    room: &RoomDef,
    world_x: f32,
    world_y: f32,
    commands: &mut Commands,
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
        PlacedRoomMarker {
            iid: room.iid.clone(),
        },
    ));
}

pub fn generation_loop(
    mut done: Local<bool>,
    mut commands: Commands,
    mut world_rng: ResMut<WorldRng>,
    doors: Query<(Entity, &EntityInstance, &GlobalTransform), With<Door>>,
    placed_rooms: Query<(&LevelIid, &GlobalTransform)>,
    room_idx: Res<RoomIndex>,
    ldtk_handle: Res<LdtkHandle>,
) {
    if room_idx.rooms.is_empty() {
        return;
    }
    if !*done {
        let spawn_room = room_idx
            .rooms
            .iter()
            .find(|rd| rd.room_type == RoomType::Spawn)
            .unwrap_or(&room_idx.rooms[0]);
        place_room(spawn_room, 0.0, 0.0, &mut commands, &ldtk_handle.0);

        commands.spawn((
            Sprite::from_color(Color::srgba(0.2, 0.2, 1.0, 1.0), Vec2::splat(8.0)),
            Transform::from_xyz(0.0, 0.0, 100.0),
        ));

        *done = true;
    }

    if placed_rooms.iter().len() >= 10 {
        return;
    }

    info!(
        "Rooms generated {}, doors open {}",
        placed_rooms.iter().len(),
        doors.iter().len()
    );
    for (entity, instance, gt) in &doors {
        if gt.translation() == Vec3::ZERO {
            info!("door skipped - transform not ready");
            continue;
        }

        let dir = if instance.width > instance.height {
            if instance.grid.y == 0 { Dir::N } else { Dir::S }
        } else {
            if instance.grid.x == 0 { Dir::W } else { Dir::E }
        };

        let Some(room_indices) = room_idx.by_door_dir.get(&dir.opposite()) else {
            info!("no rooms for dir {:?}", dir.opposite());
            continue;
        };

        info!("door at {:?}, dir {:?}", gt.translation(), dir);
        let rng = &mut world_rng.0;
        let room_idx_pick = pick_weighted(room_indices, &room_idx.rooms, rng);
        let room = &room_idx.rooms[room_idx_pick];

        let door_edge = gt.translation().truncate() + dir.as_vec() * 8.0;

        let door_offset = match dir.opposite() {
            Dir::N => { Vec2::new( 16.0, 0.0 )},
            Dir::S => { Vec2::new( 16.0, -16.0 )},
            Dir::E => { Vec2::new( 16.0, 16.0 )},
            Dir::W => { Vec2::new( 0.0, 16.0 )},
        };

        let matching_door = room.doors.iter().find(|d| d.dir == dir.opposite()).unwrap();
        let matching_door_vec = Vec2::new(matching_door.x as f32, -matching_door.y as f32) * 16.0 + door_offset;

        let room_world = -matching_door_vec + door_edge;//(Vec2::new(matching_door.x as f32, matching_door.y as f32) + dir.as_vec()) * -16.0 + door_edge;

        
        commands.spawn((
            Sprite::from_color(Color::srgba(0.5, 0.2, 1.0, 1.0), Vec2::splat(8.0)),
            Transform::from_translation(
                (room_world+matching_door_vec).extend(100.0),
            ),
        ));

        commands.spawn((
            Sprite::from_color(Color::srgba(0.2, 0.2, 1.0, 1.0), Vec2::splat(8.0)),
            Transform::from_translation(
                door_edge.extend(100.0),
            ),
        ));

        place_room(
            room,
            room_world.x,
            room_world.y,
            &mut commands,
            &ldtk_handle.0,
        );
        commands.entity(entity).despawn();
        break; // one room per frame to let transforms propagate
    }
}

fn pick_weighted(room_indices: &[usize], rooms: &[RoomDef], rng: &mut impl rand::Rng) -> usize {
    let total_weight: f32 = room_indices.iter().map(|&i| rooms[i].weight).sum();

    let mut roll = rng.random_range(0.0..total_weight);

    for &i in room_indices {
        roll -= rooms[i].weight;
        if roll <= 0.0 {
            return i;
        }
    }

    // fallback to last
    *room_indices.last().unwrap()
}

//         let matching_door = room.doors.iter().find(|d| d.dir == dir.opposite()).unwrap();

// door local position in Bevy space (Y flipped from LDtk grid)
// let door_local_x = (matching_door.x * 16) as f32;
// let door_local_y = (room.height - matching_door.y * 16) as f32;

// let (room_world_x, room_world_y) = match dir {
//     Dir::N => (
//         gt.translation().x - door_local_x,
//         gt.translation().y, // room sits above, bottom edge at frontier y
//     ),
//     Dir::S => (
//         gt.translation().x - door_local_x,
//         gt.translation().y - room.height as f32, // room sits below
//     ),
//     Dir::E => (
//         gt.translation().x, // room sits to the right
//         gt.translation().y - door_local_y,
//     ),
//     Dir::W => (
//         gt.translation().x - room.width as f32, // room sits to the left
//         gt.translation().y - door_local_y,
//     ),
// };

// info!(
//     "frontier door world: {}, {}",
//     gt.translation().x,
//     gt.translation().y
// );
// info!(
//     "matching door local grid: {}, {}",
//     matching_door.x, matching_door.y
// );
// info!("door local bevy: {}, {}", door_local_x, door_local_y);
// info!("placing room at: {}, {}", room_world_x, room_world_y);
// info!("frontier door dir: {:?}", dir);
// info!("room size: {}x{}", room.width, room.height);
// info!(
//     "room doors: {:?}",
//     room.doors
//         .iter()
//         .map(|d| (d.x, d.y, d.dir))
//         .collect::<Vec<_>>()
// );
