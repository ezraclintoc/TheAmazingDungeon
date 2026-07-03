use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};

use super::types::*;
use super::pipeline::*;

pub fn debug_open_doors(mut gizmos: Gizmos, world_state: Res<WorldState>) {
    for door in &world_state.open_doors {
        gizmos.circle_2d(
            Isometry2d::new(door.world_pos, Rot2::default()),
            2.0,
            Color::srgba(0.2, 1.0, 0.2, 1.0),
        );
    }
}

pub fn debug_grid(mut gizmos: Gizmos, camera: Query<&Transform, With<Camera2d>>) {
    let Ok(cam) = camera.single() else {
        return;
    };

    let cam_x = cam.translation.x;
    let cam_y = cam.translation.y;

    let range = 500.0;
    let step = 16.0;

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
        CAMERA_SPAWN_DIST,
        Color::srgba(1.0, 1.0, 1.0, 0.1),
    );
}

pub fn debug_room_bounds(
    mut gizmos: Gizmos,
    world_state: Res<WorldState>,
    room_idx: Res<RoomIndex>,
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
    worlds: Query<Entity, With<LevelSet>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &worlds {
            commands.entity(entity).despawn();
        }
        world_state.open_doors.clear();
        world_state.rooms.clear();
    }
}

pub fn debug_door_collision(
    mut gizmos: Gizmos,
    world_state: Res<WorldState>,
    mut commands: Commands,
) {
    for door in &world_state.open_doors {
        let (pos, size) = door.get_bounding_box();

        gizmos.rect_2d(
            Isometry2d::new(pos, Rot2::default()),
            size,
            if check_new_door_collision(door, &world_state) {
                Color::srgba(0.2, 1.0, 0.2, 1.0)
            } else {
                Color::srgba(1.0, 0.2, 0.2, 1.0)
            },
        );

        gizmos.circle_2d(
            Isometry2d::new(pos, Rot2::default()),
            2.0,
            Color::srgba(1.0, 0.2, 0.2, 1.0),
        );
    }
}
