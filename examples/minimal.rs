use bevy::prelude::*;
use bevy::image::ImagePlugin;
use bevy_ecs_ldtk::prelude::*;

use bevy_ldtk_procgen::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins(WorldPlugin::default())
        .add_systems(Startup, setup)
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: false,
            },
            ..default()
        })
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
