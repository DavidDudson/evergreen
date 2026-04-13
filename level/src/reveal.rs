use bevy::prelude::*;
use models::player::Player;
use models::reveal::{RevealState, Revealable};

/// Duration of the crossfade transition (seconds).
const REVEAL_DURATION_SECS: f32 = 0.3;

/// Detect when the player is behind revealable entities and trigger transitions.
/// Triggers whenever the player's y > entity's y (player is "above" / behind it).
pub fn detect_reveals(
    player_q: Query<&Transform, With<Player>>,
    mut revealables: Query<(&Transform, &Revealable, &mut RevealState), Without<Player>>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let pp = player_tf.translation.truncate();

    for (tf, revealable, mut state) in &mut revealables {
        let base = tf.translation.truncate();
        // Player must be behind (higher y) AND within the sprite's visual width.
        let behind = pp.y > base.y
            && pp.y < base.y + revealable.canopy_height_px
            && (pp.x - base.x).abs() < revealable.half_width_px;

        let next = match (&*state, behind) {
            (RevealState::Full, true) => Some(RevealState::Revealing(0.0)),
            (RevealState::Hiding(progress), true) => Some(RevealState::Revealing(1.0 - progress)),
            (RevealState::Revealed, false) => Some(RevealState::Hiding(0.0)),
            (RevealState::Revealing(progress), false) => Some(RevealState::Hiding(1.0 - progress)),
            _ => None,
        };
        if let Some(next_state) = next {
            *state = next_state;
        }
    }
}

/// Animate alpha fade for revealable entities (trees and decorations).
/// Fades to `revealed_full_alpha` (0.2 for trees, 0.3 for decorations).
pub fn animate_reveals(
    time: Res<Time>,
    mut query: Query<(&Revealable, &mut RevealState, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (revealable, mut state, mut sprite) in &mut query {
        let (progress, revealing) = match &*state {
            RevealState::Revealing(p) => (*p, true),
            RevealState::Hiding(p) => (*p, false),
            _ => continue,
        };

        let new_progress = (progress + dt / REVEAL_DURATION_SECS).min(1.0);
        let target = revealable.revealed_full_alpha;

        let alpha = if revealing {
            1.0 - (1.0 - target) * new_progress
        } else {
            target + (1.0 - target) * new_progress
        };

        sprite.color = sprite.color.with_alpha(alpha);

        if new_progress >= 1.0 {
            *state = if revealing {
                RevealState::Revealed
            } else {
                RevealState::Full
            };
        } else {
            *state = if revealing {
                RevealState::Revealing(new_progress)
            } else {
                RevealState::Hiding(new_progress)
            };
        }
    }
}
