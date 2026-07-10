mod pipeline;
mod types;
mod debug;
mod spatial_hash;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use self::pipeline::*;
use self::types::*;
use self::debug::*;
pub struct WorldPlugin;

/// Reads DUNGEON_SEED from the environment for a reproducible session (e.g. to debug a
/// specific reported layout); otherwise seeds from OS entropy so each run of the game
/// gets genuinely varied generation.
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
        app.init_state::<GenerationState>()
            .init_resource::<RoomIndex>()
            .init_resource::<WorldState>()
            .init_resource::<GenTask>()
            .init_resource::<SpawnQueue>()
            .insert_resource(GenRng(make_gen_rng()))
            .insert_resource(ClearColor(Color::srgb_u8(118, 59, 54)))
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
                (spawn_if_idle, poll_task).chain().run_if(in_state(GenerationState::Ready)),
            )
            .add_systems(Update, (debug_open_doors, debug_room_bounds, debug_door_collision, debug_grid, regenerate_on_key));
    }
}

#[derive(Resource)]
pub struct LdtkHandle(pub Handle<LdtkProject>);

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<LdtkProject> = asset_server.load("rooms.ldtk");
    commands.insert_resource(LdtkHandle(handle.clone()));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: handle.into(),
        level_set: LevelSet::default(),
        ..default()
    });
}
