use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::image::ImagePlugin;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
use bevy::camera_controller::pan_camera::{PanCamera, PanCameraPlugin};

use bevy_ldtk_procgen::prelude::{DebugToggles, WorldPlugin, WorldState};

#[derive(Component)]
struct HudText;

fn main() {
    let debug = std::env::args().any(|arg| arg == "--debug" || arg == "-d");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(PanCameraPlugin)
        .add_plugins(LdtkPlugin)
        .add_plugins(WorldPlugin { debug, ..default() })
        .add_systems(Startup, (setup, setup_hud))
        .add_systems(Update, (update_hud, toggle_debug))
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

fn toggle_debug(keys: Res<ButtonInput<KeyCode>>, mut toggles: ResMut<DebugToggles>) {
    if keys.just_pressed(KeyCode::KeyF) {
        toggles.gizmos = !toggles.gizmos;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        toggles.grid = !toggles.grid;
    }
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: -\nRooms (data): 0\nRooms (loaded): 0"),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        HudText,
    ));
}

fn update_hud(
    diagnostics: Res<DiagnosticsStore>,
    world_state: Res<WorldState>,
    loaded_rooms: Query<Entity, With<LevelSet>>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    text.0 = format!(
        "FPS: {:.0}\nRooms (data): {}\nRooms (loaded): {}",
        fps,
        world_state.rooms.len(),
        loaded_rooms.iter().count(),
    );
}
