use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::types::*;

pub fn debug_open_doors(mut gizmos: Gizmos, world_state: Res<WorldState>) {
    for door in &world_state.open_doors {
        gizmos.circle_2d(
            Isometry2d::new(door.world_pos, Rot2::default()),
            2.0,
            Color::srgba(0.2, 1.0, 0.2, 1.0),
        );
    }
}

pub fn debug_grid(
    mut gizmos: Gizmos,
    camera: Query<&Transform, With<Camera2d>>,
    windows: Query<&Window>,
    config: Res<GenerationConfig>,
) {
    let Ok(cam) = camera.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    let cam_x = cam.translation.x;
    let cam_y = cam.translation.y;

    let range = config.camera_spawn_dist * 1.75;
    let step = 16.0;

    // PanCamera zooms by scaling the camera's transform (smaller scale = zoomed in
    // more; see bevy_camera_controller::pan_camera), so the visible viewport in world
    // units is the window size scaled by that factor. Only draw the grid once we're
    // zoomed in far enough that the CAMERA_SPAWN_DIST circle no longer fits on screen.
    let visible_half_extent = window.width().min(window.height()) / 2.0 * cam.scale.z;

    if visible_half_extent < config.camera_spawn_dist {
        let x_start = ((cam_x - range) / step).floor() * step;
        let y_start = ((cam_y - range) / step).floor() * step;

        for i in 0..((range * 2.0 / step) as i32) {
            let x = x_start + i as f32 * step;
            let y = y_start + i as f32 * step;

            // vertical lines
            gizmos.line_2d(
                Vec2::new(x, cam_y - range),
                Vec2::new(x, cam_y + range),
                Color::srgba(1.0, 1.0, 1.0, 0.1),
            );

            // horizontal lines
            gizmos.line_2d(
                Vec2::new(cam_x - range, y),
                Vec2::new(cam_x + range, y),
                Color::srgba(1.0, 1.0, 1.0, 0.1),
            );
        }
    };

    // draw origin
    gizmos.line_2d(
        Vec2::new(-8.0, 0.0),
        Vec2::new(8.0, 0.0),
        Color::srgb(1.0, 0.0, 0.0),
    );
    gizmos.line_2d(
        Vec2::new(0.0, -8.0),
        Vec2::new(0.0, 8.0),
        Color::srgb(1.0, 0.0, 0.0),
    );

    gizmos.circle_2d(
        Isometry2d::new(cam.translation.truncate(), Rot2::default()),
        config.camera_spawn_dist,
        Color::srgba(1.0, 1.0, 1.0, 0.1),
    );
}

pub fn debug_room_bounds(
    mut gizmos: Gizmos,
    world_state: Res<WorldState>,
) {
    for room in &world_state.rooms {
        let x = room.world_pos.x + room.room.size.x as f32 / 2.0; //gt.translation().x;// + room.width as f32 / 2.0 + room.offset_x;
        let y = room.world_pos.y - room.room.size.y as f32 / 2.0; //gt.translation().y;// + room.height as f32 / 2.0;

        gizmos.circle_2d(Vec2::new(x, y), 2.0, Color::srgba(1.0, 0.2, 0.2, 1.0));

        gizmos.rect_2d(
            Vec2::new(x, y),
            room.room.size.as_vec2(),
            Color::srgba(0.0, 1.0, 0.0, 0.15),
        );
    }
}

pub fn regenerate_on_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world_state: ResMut<WorldState>,
    mut spawn_queue: ResMut<SpawnQueue>,
    worlds: Query<Entity, With<LevelSet>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &worlds {
            commands.entity(entity).despawn();
        }
        // full reset, not per-field .clear() - room_grid must reset with rooms
        *world_state = WorldState::default();
        // also clear queued-but-unspawned rooms, or they'd spawn into the fresh world
        spawn_queue.0.clear();
    }
}

pub fn debug_door_collision(
    mut gizmos: Gizmos,
    world_state: Res<WorldState>,
) {
    for door in &world_state.open_doors {
        let (pos, size) = door.get_bounding_box();

        gizmos.rect_2d(
            Isometry2d::new(pos, Rot2::default()),
            size,
            Color::srgba(0.2, 1.0, 0.2, 1.0),
        );

        gizmos.circle_2d(
            Isometry2d::new(pos, Rot2::default()),
            2.0,
            Color::srgba(1.0, 0.2, 0.2, 1.0),
        );
    }
}
