mod world;

pub use world::WorldPlugin;

pub mod prelude {
    pub use crate::world::{
        DebugToggles, Dir, Door, DoorDef, GenerationState, Room, RoomDef, RoomType, WorldPlugin,
        WorldState,
    };
}
