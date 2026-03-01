use bevy::prelude::*;
use models::health::Health;
use std::f32::consts::TAU;

const HUD_OFFSET_PX: f32 = 8.0;
const ROSE_CONTAINER_PX: f32 = 80.0;
const PETAL_SIZE_PX: f32 = 32.0;
const PETAL_ORBIT_RADIUS_PX: f32 = 20.0;
const PETAL_COUNT: u16 = 10;

#[derive(Component)]
pub struct Hud;

#[derive(Component)]
pub struct RosePetal(pub u16);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let petal_image = asset_server.load("rose_petal.png");
    let half_petal = PETAL_SIZE_PX / 2.0;
    let center = ROSE_CONTAINER_PX / 2.0;

    commands
        .spawn((
            Hud,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(HUD_OFFSET_PX),
                right: Val::Px(HUD_OFFSET_PX),
                width: Val::Px(ROSE_CONTAINER_PX),
                height: Val::Px(ROSE_CONTAINER_PX),
                ..Node::default()
            },
        ))
        .with_children(|parent| {
            for i in 1..=PETAL_COUNT {
                let angle = TAU * f32::from(i - 1) / f32::from(PETAL_COUNT);
                let x = PETAL_ORBIT_RADIUS_PX * angle.sin();
                let y = -PETAL_ORBIT_RADIUS_PX * angle.cos();

                parent.spawn((
                    RosePetal(i),
                    ImageNode::new(petal_image.clone()),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(center + x - half_petal),
                        top: Val::Px(center + y - half_petal),
                        width: Val::Px(PETAL_SIZE_PX),
                        height: Val::Px(PETAL_SIZE_PX),
                        ..Node::default()
                    },
                    Transform::from_rotation(Quat::from_rotation_z(-angle)),
                ));
            }
        });
}

pub fn sync_petals(
    health_query: Query<&Health, Changed<Health>>,
    petal_query: Query<(Entity, &RosePetal)>,
    mut commands: Commands,
) {
    let Ok(health) = health_query.single() else {
        return;
    };
    petal_query
        .iter()
        .filter(|(_, petal)| petal.0 > health.0)
        .for_each(|(entity, _)| commands.entity(entity).despawn());
}
