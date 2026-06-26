use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};
use rand::SeedableRng;
use std::collections::HashMap;
use std::str::FromStr;

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
    pub width: i32, 
    pub height: i32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub doors: Vec<DoorDef>,
    pub weight: f32,
    pub room_type: RoomType,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub world_x: f32,
    pub world_y: f32,
    pub room: RoomDef, 
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoorDef {
    pub x: i32, // local
    pub y: i32,
    pub width: i32,
    pub dir: Dir,    
}

#[derive(Debug, Clone, PartialEq)]
pub struct Door {
    pub door: DoorDef,
    pub world_x: f32, // global
    pub world_y: f32,
}

#[derive(Resource, Default, Clone)]
pub struct WorldState {
    pub initialized: bool,
    pub open_doors: Vec<Door>,
    pub rooms: Vec<Room>,
}

#[derive(Resource, Default)]
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
        let wh = width/2.0;
        let door_offset = match self {
            Dir::N => Vec2::new(16.0*wh, 0.0),
            Dir::S => Vec2::new(16.0*wh, -16.0),
            Dir::E => Vec2::new(16.0, -16.0*wh),
            Dir::W => Vec2::new(0.0, -16.0*wh),
        };
        door_offset
    }
}

#[derive(Resource)]
pub struct WorldRng(pub rand::rngs::SmallRng);

impl Default for WorldRng {
    fn default() -> Self {
        Self(rand::rngs::SmallRng::from_rng(&mut rand::rng()))
    }
}