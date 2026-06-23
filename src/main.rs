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
        .add_plugins(WorldPlugin)
        .add_systems(Startup, setup)
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            ..default()
        })
        .run();
}

const IIDS: [&str; 2] = [
    "f10df9c0-48b0-11f1-938d-2b1fd25e17a5",
    "ceaf3be0-48b0-11f1-938d-bdb5c12c0797",
];

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, PanCamera::default()));

    let level_set = LevelSet::from_iids(IIDS);

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("rooms.ldtk").into(),
        level_set,
        ..Default::default()
    });
}
