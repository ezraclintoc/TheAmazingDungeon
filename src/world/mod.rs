mod world;
mod util;
mod debug;

use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};

use self::world::*;
use self::util::*;
use self::debug::*;
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomIndex>()
            .init_resource::<WorldRng>()
            .init_resource::<WorldState>()
            .insert_resource(ClearColor(Color::srgb_u8(118,  59, 54)))
            .add_systems(Startup, setup_world)
            .add_systems(Update, create_room_index)
            .add_systems(Update, (debug_grid, regenerate_on_key))
            .add_systems(Update, debug_room_bounds)
            .add_systems(PostUpdate, generation_loop.after(TransformSystems::Propagate));
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