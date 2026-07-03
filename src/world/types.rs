use bevy::tasks::Task;
use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};
use rand::SeedableRng;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GenerationState {
    #[default]
    AssetLoading,
    Indexing,
    Ready,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomType {
    Spawn,
    Room,
    Hallway,
}

impl FromStr for RoomType {
    type Err = ();

    fn from_str(input: &str) -> Result<RoomType, Self::Err> {
        match input {
            "Hallway" => Ok(RoomType::Hallway),
            "Room" => Ok(RoomType::Room),
            "Spawn" => Ok(RoomType::Spawn),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomDef {
    pub iid: String,
    pub size: IVec2,
    pub offset: Vec2,
    pub doors: Vec<DoorDef>,
    pub weight: f32,
    pub room_type: RoomType,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub room: RoomDef,
    pub world_pos: Vec2,
}

impl Room {
    pub fn new(roomdef: &RoomDef, world_pos: Vec2) -> Self {
        Room {
            world_pos,
            room: roomdef.clone(),
        }
    }
}

#[derive(Resource, Default)]
pub struct GenTask(pub Option<Task<Vec<Room>>>);

#[derive(Debug, Clone, PartialEq)]
pub struct DoorDef {
    pub local_pos: IVec2,
    pub width: i32,
    pub dir: Dir,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Door {
    pub door: DoorDef,
    pub world_pos: Vec2,
}

impl Door {
    pub fn new(room: &Room, door: &DoorDef) -> Self {
        Door {
            door: door.clone(),
            world_pos: room.world_pos
                + door.local_pos.as_vec2() * 16.0
                + door.dir.door_offset(door.width as f32) * Vec2::new(0.0, 1.0),
        }
    }

    pub fn get_bounding_box(&self) -> (Vec2, Vec2) {
        let pos = self.world_pos + self.door.dir.as_vec() * 16.0;
        match self.door.dir {
            Dir::N => (pos + Vec2::new(0.0, 8.0), Vec2::new(64.0, 48.0)),
            Dir::S => (pos, Vec2::new(64.0, 32.0)),
            Dir::E => (pos + Vec2::new(0.0, 8.0), Vec2::new(32.0, 80.0)),
            Dir::W => (pos + Vec2::new(0.0, 8.0), Vec2::new(32.0, 80.0)),
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct WorldState {
    pub open_doors: Vec<Door>,
    pub rooms: Vec<Room>,
}

#[derive(Resource, Default, Clone)]
pub struct RoomIndex {
    pub rooms: Vec<RoomDef>,
    pub by_door_dir: HashMap<Dir, Vec<usize>>, // direction -> indices into rooms
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum Dir {
    N,
    S,
    E,
    W,
}

impl Dir {
    pub fn as_vec(&self) -> Vec2 {
        match self {
            Dir::N => Vec2::from_array([0.0, 1.0]),
            Dir::S => Vec2::from_array([0.0, -1.0]),
            Dir::E => Vec2::from_array([1.0, 0.0]),
            Dir::W => Vec2::from_array([-1.0, 0.0]),
        }
    }

    pub fn opposite(&self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E,
        }
    }

    pub fn rotate_cl(&self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::S => Dir::W,
            Dir::E => Dir::S,
            Dir::W => Dir::N,
        }
    }

    pub fn door_offset(&self, width: f32) -> Vec2 {
        let wh = width / 2.0;
        let door_offset = match self {
            Dir::N => Vec2::new(16.0 * wh, 0.0),
            Dir::S => Vec2::new(16.0 * wh, -16.0),
            Dir::E => Vec2::new(16.0, -16.0 * wh),
            Dir::W => Vec2::new(0.0, -16.0 * wh),
        };
        door_offset
    }

    // pub fn is_vertical(&self) -> bool {
    //     match self {
    //         Dir::N => true,
    //         Dir::S => true,
    //         Dir::E => false,
    //         Dir::W => false,
    //     }
    // }

    // pub fn is_horizontal(&self) -> bool {
    //     !self.is_vertical()
    // }
}

pub fn rects_collide(center_a: Vec2, size_a: Vec2, top_left_b: Vec2, size_b: Vec2) -> bool {
    // rect A from center
    let a_left = center_a.x - size_a.x / 2.0;
    let a_right = center_a.x + size_a.x / 2.0;
    let a_top = center_a.y + size_a.y / 2.0;
    let a_bottom = center_a.y - size_a.y / 2.0;

    // rect B from top-left, extending right and down (y decreasing)
    let b_left = top_left_b.x;
    let b_right = top_left_b.x + size_b.x;
    let b_top = top_left_b.y;
    let b_bottom = top_left_b.y - size_b.y;

    a_left < b_right && a_right > b_left && a_bottom < b_top && a_top > b_bottom
}