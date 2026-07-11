mod pipeline;
mod types;
mod debug;
mod spatial_hash;
mod culling;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use self::pipeline::*;
use self::types::*;
use self::debug::*;
use self::culling::*;

pub use self::types::{
    DebugToggles, Dir, Door, DoorDef, GenerationState, Room, RoomDef, RoomType, WorldState,
};

pub struct WorldPlugin {
    pub ldtk_path: String,
    /// Initial state for the debug gizmo/grid overlays (see `DebugToggles` to change at runtime).
    pub debug: bool,
    /// How far from the camera to search for open doors to fill.
    pub camera_spawn_dist: f32,
    /// Session-wide cap on total placed rooms.
    pub max_rooms: usize,
    /// How far a spawned room can get from the camera before its entity is despawned
    /// (it respawns automatically if the camera comes back within range).
    pub cull_dist: f32,
}
impl Default for WorldPlugin {
    fn default() -> Self {
        let config = GenerationConfig::default();
        Self {
            ldtk_path: "rooms.ldtk".into(),
            debug: false,
            camera_spawn_dist: config.camera_spawn_dist,
            max_rooms: config.max_rooms,
            cull_dist: config.cull_dist,
        }
    }
}

#[derive(Resource)]
pub struct LdtkPath(String);

fn make_gen_rng() -> SmallRng {
    match std::env::var("DUNGEON_SEED").ok().and_then(|s| s.parse::<u64>().ok()) {
        Some(seed) => {
            info!("DUNGEON_SEED={} set: generation session is deterministic", seed);
            SmallRng::seed_from_u64(seed)
        }
        None => SmallRng::from_rng(&mut rand::rng()),
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let config = GenerationConfig {
            camera_spawn_dist: self.camera_spawn_dist,
            max_rooms: self.max_rooms,
            cull_dist: self.cull_dist,
        };

        app.init_state::<GenerationState>()
            .init_resource::<RoomIndex>()
            .init_resource::<WorldState>()
            .init_resource::<GenTask>()
            .init_resource::<SpawnQueue>()
            .init_resource::<DespawnQueue>()
            .init_resource::<CullTimer>()
            .insert_resource(GenRng(make_gen_rng()))
            .insert_resource(ClearColor(Color::srgb_u8(118, 59, 54)))
            .insert_resource(LdtkPath(self.ldtk_path.clone()))
            .insert_resource(config)
            .insert_resource(DebugToggles { gizmos: self.debug, grid: self.debug })
            .add_systems(Startup, setup_world)
            .add_systems(
                Update,
                is_ldtk_loaded.run_if(in_state(GenerationState::AssetLoading)),
            )
            .add_systems(
                Update,
                create_room_index.run_if(in_state(GenerationState::Indexing)),
            )
            .add_systems(
                Update,
                (spawn_if_idle, poll_task, cull_and_respawn_rooms)
                    .chain()
                    .run_if(in_state(GenerationState::Ready)),
            )
            .add_systems(Update, regenerate_on_key)
            .add_systems(
                Update,
                (debug_open_doors, debug_room_bounds, debug_door_collision)
                    .run_if(|t: Res<DebugToggles>| t.gizmos),
            )
            .add_systems(Update, debug_grid.run_if(|t: Res<DebugToggles>| t.grid));
    }
}

#[derive(Resource)]
pub struct LdtkHandle(pub Handle<LdtkProject>);

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>, ldtk_path: Res<LdtkPath>) {
    let handle: Handle<LdtkProject> = asset_server.load(ldtk_path.0.clone());
    commands.insert_resource(LdtkHandle(handle.clone()));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: handle.into(),
        level_set: LevelSet::default(),
        ..default()
    });
}
