//! NPC wandering behaviour — idle/walk cycle near spawn point.
//!
//! When the player is within interaction range (indicated by
//! [`DialogueTrigger`] pointing at this NPC), the NPC faces the player
//! and stops walking.

use bevy::prelude::*;
use dialog::components::DialogueTrigger;
use models::npc_anim::{NpcAnimKind, NpcFacing};
use models::speed::Speed;
use rand::Rng;

/// NPC walk speed in pixels per second.
const WANDER_SPEED_PX: f32 = 16.0;
/// Minimum idle duration (seconds).
const IDLE_MIN_SECS: f32 = 2.0;
/// Maximum idle duration (seconds).
const IDLE_MAX_SECS: f32 = 5.0;
/// Minimum walk duration (seconds).
const WALK_MIN_SECS: f32 = 1.0;
/// Maximum walk duration (seconds).
const WALK_MAX_SECS: f32 = 2.5;
/// Maximum wander distance from spawn point (pixels).
const WANDER_RADIUS_PX: f32 = 32.0;

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Drives an NPC's idle/walk cycle around its spawn point.
#[derive(Component)]
pub struct NpcWander {
    pub state: WanderState,
    pub timer: Timer,
    /// World-space spawn point — the NPC stays within [`WANDER_RADIUS_PX`].
    pub origin: Vec2,
}

pub enum WanderState {
    Idle,
    /// Normalized direction vector.
    Walking(Vec2),
}

impl NpcWander {
    pub fn new(origin: Vec2) -> Self {
        Self {
            state: WanderState::Idle,
            timer: Timer::from_seconds(IDLE_MIN_SECS, TimerMode::Once),
            origin,
        }
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Drives NPC wander behaviour.
///
/// - If the player is targeting this NPC, face them and idle.
/// - Otherwise cycle between idling and walking in random directions.
pub fn wander_npcs(
    time: Res<Time>,
    player_q: Query<(Option<&DialogueTrigger>, &GlobalTransform), With<Speed>>,
    mut npc_q: Query<(
        Entity,
        &mut Transform,
        &mut NpcWander,
        &mut NpcFacing,
        &mut NpcAnimKind,
    )>,
) {
    let (trigger, player_pos) = match player_q.single() {
        Ok((t, gt)) => (t.map(|tr| tr.npc), gt.translation().truncate()),
        Err(_) => return,
    };

    for (entity, mut tf, mut wander, mut facing, mut anim_kind) in &mut npc_q {
        // ---- Face player when targeted ----
        if trigger == Some(entity) {
            let dir = player_pos - tf.translation.truncate();
            if dir.length_squared() > 1.0 {
                *facing = NpcFacing::from_vec2(dir);
            }
            if *anim_kind != NpcAnimKind::Idle {
                *anim_kind = NpcAnimKind::Idle;
            }
            wander.timer.reset();
            continue;
        }

        // ---- Wander ----
        wander.timer.tick(time.delta());

        // Move while walking (even before timer finishes).
        if let WanderState::Walking(dir) = wander.state {
            let candidate = tf.translation.truncate() + dir * WANDER_SPEED_PX * time.delta_secs();
            let offset = candidate - wander.origin;
            if offset.length() <= WANDER_RADIUS_PX {
                tf.translation.x = candidate.x;
                tf.translation.y = candidate.y;
            } else {
                // Hit radius boundary — stop early.
                wander.state = WanderState::Idle;
                *anim_kind = NpcAnimKind::Idle;
                let mut rng = rand::thread_rng();
                let dur = rng.gen_range(IDLE_MIN_SECS..IDLE_MAX_SECS);
                wander.timer = Timer::from_seconds(dur, TimerMode::Once);
                continue;
            }
        }

        if !wander.timer.is_finished() {
            continue;
        }

        // ---- Transition ----
        let mut rng = rand::thread_rng();
        match wander.state {
            WanderState::Idle => {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let dir = Vec2::new(angle.cos(), angle.sin());
                *facing = NpcFacing::from_vec2(dir);
                *anim_kind = NpcAnimKind::Walk;
                wander.state = WanderState::Walking(dir);
                let dur = rng.gen_range(WALK_MIN_SECS..WALK_MAX_SECS);
                wander.timer = Timer::from_seconds(dur, TimerMode::Once);
            }
            WanderState::Walking(_) => {
                *anim_kind = NpcAnimKind::Idle;
                wander.state = WanderState::Idle;
                let dur = rng.gen_range(IDLE_MIN_SECS..IDLE_MAX_SECS);
                wander.timer = Timer::from_seconds(dur, TimerMode::Once);
            }
        }
    }
}
