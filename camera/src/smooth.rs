use bevy::prelude::*;
use level::area::{Direction, MAP_HEIGHT, MAP_WIDTH};
use level::plugin::TILE_SIZE_PX;
use level::world::AreaChanged;
use player::Player;

// ---------------------------------------------------------------------------
// Area dimensions
// ---------------------------------------------------------------------------

#[allow(clippy::as_conversions)] // const context: no From/Into available
const MAP_W_PX: f32 = MAP_WIDTH as f32 * TILE_SIZE_PX as f32;
#[allow(clippy::as_conversions)]
const MAP_H_PX: f32 = MAP_HEIGHT as f32 * TILE_SIZE_PX as f32;

const HALF_W: f32 = MAP_W_PX / 2.0;
const HALF_H: f32 = MAP_H_PX / 2.0;

// ---------------------------------------------------------------------------
// Transition tuning
// ---------------------------------------------------------------------------

/// Fraction of the area dimension used as the transition zone on each side of
/// the boundary.  Camera begins leading at this distance from the edge and
/// finishes settling the same distance into the next area.
const TRANSITION_FRAC_NS: f32 = 0.15;
const TRANSITION_FRAC_EW: f32 = 0.075;

const ZONE_X: f32 = MAP_W_PX * TRANSITION_FRAC_EW;
const ZONE_Y: f32 = MAP_H_PX * TRANSITION_FRAC_NS;

/// Time-based lerp speed for the dialogue-return camera slide.
const DIALOGUE_RETURN_SPEED: f32 = 5.0;

/// Squared distance below which offsets snap to zero.
const SNAP_THRESHOLD_SQ: f32 = 0.25;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Drives smooth camera transitions for area changes and dialogue exit.
///
/// Area transitions use **position-based** easing (camera moves in lockstep
/// with the player, peak speed at the boundary).  Dialogue return uses a
/// **time-based** lerp so it settles even if the player stands still.
#[derive(Resource, Default)]
pub struct CameraOffset {
    /// Initial residual set on area transition (for position-based decay).
    initial_residual: Vec2,
    /// Direction of the most recent area transition.
    transition_dir: Option<Direction>,
    /// Time-based residual for the dialogue camera return.
    pub dialogue_return: Vec2,
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Computes the camera position each frame.
///
/// Runs in `PostUpdate` so it always sees `AreaChanged` messages written
/// during `Update`.
pub fn apply_camera_offset(
    mut state: ResMut<CameraOffset>,
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    mut messages: MessageReader<AreaChanged>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let Ok(mut cam_tf) = camera_q.single_mut() else {
        return;
    };
    let pos = player_tf.translation.truncate();

    // ── Area transition ──
    // Set camera to the player's new position so they stay centered, then
    // let the post-crossing ease-out slide the camera back to the origin.
    // Only the crossing axis gets a residual; the perpendicular axis resets.
    for event in messages.read() {
        state.initial_residual = match event.direction {
            Direction::East | Direction::West => Vec2::new(pos.x, 0.0),
            Direction::North | Direction::South => Vec2::new(0.0, pos.y),
        };
        state.transition_dir = Some(event.direction);
        state.dialogue_return = Vec2::ZERO;
    }

    let mut target = Vec2::ZERO;

    // ── Post-crossing residual (position-based, quadratic ease-out) ──
    if let Some(dir) = state.transition_dir {
        let t = post_progress(pos, dir).clamp(0.0, 1.0);
        let factor = (1.0 - t) * (1.0 - t);
        target += state.initial_residual * factor;

        if t >= 1.0 {
            state.transition_dir = None;
            state.initial_residual = Vec2::ZERO;
        }
    }

    // ── Pre-crossing lead (quadratic ease-in, only when no transition active) ──
    if state.transition_dir.is_none() {
        target.x += edge_lead(pos.x, HALF_W, ZONE_X);
        target.y += edge_lead(pos.y, HALF_H, ZONE_Y);
    }

    // ── Dialogue return (time-based) ──
    if state.dialogue_return != Vec2::ZERO {
        let alpha = (DIALOGUE_RETURN_SPEED * time.delta_secs()).min(1.0);
        state.dialogue_return = state.dialogue_return.lerp(Vec2::ZERO, alpha);
        if state.dialogue_return.length_squared() < SNAP_THRESHOLD_SQ {
            state.dialogue_return = Vec2::ZERO;
        }
        target += state.dialogue_return;
    }

    cam_tf.translation.x = target.x;
    cam_tf.translation.y = target.y;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pre-crossing edge lead: quadratic ease-in as the player nears an edge.
///
/// Returns a signed camera offset toward the nearest edge.  The derivative is
/// zero at the threshold (smooth entry into the zone) and maximal at the
/// boundary (peak speed matched by the post-crossing ease-out).
fn edge_lead(pos: f32, half: f32, zone: f32) -> f32 {
    let threshold = half - zone;
    if pos > threshold {
        let t = ((pos - threshold) / zone).clamp(0.0, 1.0);
        t * t * half
    } else if pos < -threshold {
        let t = ((-pos - threshold) / zone).clamp(0.0, 1.0);
        -(t * t * half)
    } else {
        0.0
    }
}

/// Progress through the post-crossing zone (0 = at area edge, 1 = complete).
///
/// The player's overshoot past the boundary is mirrored into the new area,
/// so they enter at approximately the edge and progress measures distance
/// from that edge.
fn post_progress(pos: Vec2, dir: Direction) -> f32 {
    match dir {
        Direction::East => (pos.x + HALF_W) / ZONE_X,
        Direction::West => (HALF_W - pos.x) / ZONE_X,
        Direction::North => (pos.y + HALF_H) / ZONE_Y,
        Direction::South => (HALF_H - pos.y) / ZONE_Y,
    }
}
