use bevy::{log::tracing_subscriber::layer::Layered, prelude::*};
use bevy_ecs_ldtk::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<OpenDoorBundle>("OpenDoor");
        app.add_systems(Update, spawn_doors);
    }
}

#[derive(Default, Component)]
struct OpenDoor;

// derive direction at spawn from the floor around it, so no LDtk field needed
#[derive(Default, Component)]
struct Connector {
    direction: Option<Vec2>, // filled in after spawn
}

#[derive(Default, Bundle, LdtkEntity)]
struct OpenDoorBundle {
    marker: OpenDoor,
    connector: Connector,
    // grid coords and instance data come in automatically via EntityInstance
    #[from_entity_instance]
    instance: EntityInstance,
}

fn spawn_doors(mut commands: Commands, doors: Query<Entity, Added<OpenDoor>>) {
    for door in &doors {
        commands.entity(door).insert(Visibility::default());
        commands.entity(door).with_children(|parent| {
            parent.spawn((
                Sprite::from_color(
                    Color::srgba(0.2, 1.0, 0.2, 0.5),
                    Vec2::splat(16.0),
                ),
                Transform::from_xyz(0.0, 0.0, 10.0),
            ));
        });
    }
}
