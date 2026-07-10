use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy::image::ImagePlugin;
use bevy::window::{MonitorSelection, WindowMode};
use bevy::camera_controller::pan_camera::{PanCamera, PanCameraPlugin};

mod world;
use crate::world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PanCameraPlugin)
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
    commands.spawn((Camera2d, PanCamera::default()));
}
