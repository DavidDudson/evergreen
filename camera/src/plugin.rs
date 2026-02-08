use avian2d::prelude::LinearVelocity;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use castle::castle::Castle;
use models::draggable::{Draggable, Dragged};
use models::game_states::GameState;

pub struct CameraPlugin;

#[derive(Component)]
struct CameraDrag {
    dragging: bool,
    target_scale: f32,
    last_drag: Option<f32>,
    last_cursor_pos: Option<Vec2>,
    cursor_velocity: Vec2,
    last_update_time: Option<f32>,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                castle_follow
                    .run_if(in_state(GameState::MainMenu).or(in_state(GameState::GameOver))),
                camera_drag.run_if(in_state(GameState::Playing)),
                drag_system.run_if(in_state(GameState::Playing)),
            ),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        CameraDrag {
            dragging: false,
            target_scale: 1.0,
            last_drag: None,
            last_cursor_pos: None,
            cursor_velocity: Vec2::ZERO,
            last_update_time: None,
        },
    ));
}

fn castle_follow(
    target_query: Query<&Transform, With<Castle>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), (With<Camera>, Without<Castle>)>,
    time: Res<Time>,
) {
    // Get the target position
    if let Ok(target_transform) = target_query.single() {
        // Get the camera
        if let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() {
            let target_position = Vec3::new(
                target_transform.translation.x,
                target_transform.translation.y,
                camera_transform.translation.z,
            );

            camera_transform.translation = camera_transform
                .translation
                .lerp(target_position, 5.0 * time.delta_secs());

            let target_scale = 0.5;
            if let Projection::Orthographic(ref mut ortho) = *projection {
                let current_scale = ortho.scale;
                ortho.scale =
                    current_scale + (target_scale - current_scale) * 2.0 * time.delta_secs();
            }
        }
    }
}

// New system for camera dragging
fn camera_drag(
    mut camera_query: Query<(&mut Transform, &mut CameraDrag), With<Camera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut scroll_events: MessageReader<MouseWheel>,
    mut projection_query: Query<&mut Projection, With<Camera>>,
    dragged_entity: Query<&Dragged>,
    time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Handle camera dragging
    let Ok((mut camera_transform, mut camera_drag)) = camera_query.single_mut() else {
        return;
    };

    let current_time = time.elapsed_secs();
    let Ok(window) = window_query.single() else {
        return;
    };

    // Get the current cursor position
    let current_cursor_pos = window.cursor_position();

    if !dragged_entity.is_empty() {
        camera_drag.last_drag = Some(time.elapsed_secs());
        camera_drag.dragging = false;

        if let Some(current_pos) = current_cursor_pos {
            if let Some(last_pos) = camera_drag.last_cursor_pos {
                if let Some(last_time) = camera_drag.last_update_time {
                    let time_delta = current_time - last_time;
                    if time_delta > 0.0 {
                        // Calculate velocity (pixels per second)
                        let displacement = current_pos - last_pos;
                        let raw_velocity = displacement / time_delta;

                        // Apply smooth filtering to avoid jitter
                        let smoothing_factor = 0.8; // Adjust as needed (higher = more smoothing)
                        camera_drag.cursor_velocity = camera_drag.cursor_velocity
                            * smoothing_factor
                            + raw_velocity * (1.0 - smoothing_factor);

                        if camera_drag.dragging {
                            // Debug output (only when actively dragging)
                            info!(
                                "Cursor velocity: {:?} pixels/sec",
                                camera_drag.cursor_velocity
                            );
                        }
                    }
                }
            }

            // Update last position and time
            camera_drag.last_cursor_pos = Some(current_pos);
            camera_drag.last_update_time = Some(current_time);
        }

        return;
    }

    if !camera_drag.last_drag.is_none()
        && time.elapsed_secs() - camera_drag.last_drag.unwrap() < 0.5
    {
        camera_drag.dragging = false;
        camera_drag.last_cursor_pos = None;
        camera_drag.cursor_velocity = Vec2::ZERO;
        camera_drag.last_update_time = None;
        return;
    }

    // Start dragging when middle mouse button is pressed
    if mouse_button.just_pressed(MouseButton::Left) {
        camera_drag.dragging = true;
        camera_drag.last_cursor_pos = current_cursor_pos;
        camera_drag.last_update_time = Some(current_time);
    }

    // Stop dragging when middle mouse button is released
    if mouse_button.just_released(MouseButton::Left) {
        camera_drag.dragging = false;
        camera_drag.last_cursor_pos = None;
        camera_drag.last_update_time = None;
    }

    // Move camera if dragging
    if camera_drag.dragging {
        for event in mouse_motion_events.read() {
            // Get the current projection scale to adjust drag sensitivity
            let Ok(projection) = projection_query.single() else {
                continue;
            };
            let scale_factor = if let Projection::Orthographic(ref ortho) = *projection {
                ortho.scale
            } else {
                1.0
            };

            // Move camera in the opposite direction of mouse movement
            // Scale the movement by the projection scale to maintain consistent drag feel at different zoom levels
            camera_transform.translation.x -= event.delta.x * scale_factor;
            camera_transform.translation.y += event.delta.y * scale_factor;
        }
    }

    // Handle zooming with mouse wheel
    let mut zoom_delta = 0.0;
    for event in scroll_events.read() {
        zoom_delta -= event.y * 0.002; // Adjust sensitivity as needed
    }

    if zoom_delta != 0.0 {
        // Update the target scale based on the zoom delta
        camera_drag.target_scale = (camera_drag.target_scale + zoom_delta).clamp(0.1, 2.0);
    }

    // Smoothly interpolate towards the target scale
    if let Ok(mut projection) = projection_query.single_mut() {
        let zoom_speed = 5.0; // Adjust this value to control zoom smoothness (higher = faster)
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = ortho.scale
                + (camera_drag.target_scale - ortho.scale) * zoom_speed * time.delta_secs();
        }
    }
}

// New system for dragging peasants
fn drag_system(
    // Assuming you have a Peasant component defined elsewhere
    draggable: Query<(Entity, &mut Transform), (With<Draggable>, Without<Dragged>)>,
    mut dragged: Query<(Entity, &mut Transform, &mut LinearVelocity), With<Dragged>>,
    camera_query: Query<(&Camera, &GlobalTransform, &CameraDrag)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((camera, camera_transform, camera_drag)) = camera_query.single() else {
        return;
    };

    // Get the current mouse position in world coordinates
    let cursor_position = if let Some(cursor_position) = window.cursor_position() {
        // Convert screen position to world position
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            world_position
        } else {
            if let Ok((entity, _, _)) = dragged.single() {
                commands.entity(entity).remove::<Dragged>();
            }
            return; // Cursor not in world
        }
    } else {
        if let Ok((entity, _, _)) = dragged.single() {
            commands.entity(entity).remove::<Dragged>();
        }
        return; // Cursor not in window
    };

    // Check if we're already dragging a peasant
    let mut dragging = false;
    if let Ok((entity, mut transform, mut velocity)) = dragged.single_mut() {
        dragging = true;

        // If mouse button is released, stop dragging
        if mouse_button.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Dragged>();
            let velocity_scale = 15.;

            // Set velocity directly instead of using impulse
            velocity.x = camera_drag.cursor_velocity.x * velocity_scale;
            velocity.y = -camera_drag.cursor_velocity.y * velocity_scale;
            info!("{}", camera_drag.cursor_velocity);
        } else {
            // Update the position of the dragged peasant to follow the cursor
            transform.translation.x = cursor_position.x;
            transform.translation.y = cursor_position.y;
        }
    }

    if !dragging && mouse_button.just_pressed(MouseButton::Left) {
        for (entity, transform) in draggable.iter() {
            let peasant_pos = Vec2::new(transform.translation.x, transform.translation.y);
            let distance = cursor_position.distance(peasant_pos);
            if distance < 40.0 {
                commands.entity(entity).insert(Dragged);
                break; // Only pick up one peasant at a time
            }
        }
    }
}
