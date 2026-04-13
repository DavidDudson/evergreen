use bevy::prelude::*;
use models::player::Player;
use models::reveal::{FullSprite, RevealState, Revealable, StumpSprite};

/// Distance north of entity base that triggers reveal (pixels).
const REVEAL_TRIGGER_PX: f32 = 16.0;

/// Duration of the crossfade transition (seconds).
const REVEAL_DURATION_SECS: f32 = 0.3;

/// Detect when the player is behind revealable entities and trigger transitions.
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
        let behind = pp.y > base.y
            && pp.y < base.y + REVEAL_TRIGGER_PX
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

/// Animate the crossfade between full and stump sprites (for entities with child sprites).
pub fn animate_reveals(
    time: Res<Time>,
    mut revealables: Query<(&mut RevealState, &Children), With<Revealable>>,
    mut sprites: Query<&mut Sprite>,
    full_q: Query<(), With<FullSprite>>,
    stump_q: Query<(), With<StumpSprite>>,
) {
    let dt = time.delta_secs();

    for (mut state, children) in &mut revealables {
        let (progress, revealing) = match &*state {
            RevealState::Revealing(p) => (*p, true),
            RevealState::Hiding(p) => (*p, false),
            _ => continue,
        };

        let new_progress = (progress + dt / REVEAL_DURATION_SECS).min(1.0);

        let (full_alpha, stump_alpha) = if revealing {
            (1.0 - new_progress, new_progress)
        } else {
            (new_progress, 1.0 - new_progress)
        };

        for child in children.iter() {
            if let Ok(mut sprite) = sprites.get_mut(child) {
                if full_q.contains(child) {
                    sprite.color = sprite.color.with_alpha(full_alpha);
                } else if stump_q.contains(child) {
                    sprite.color = sprite.color.with_alpha(stump_alpha);
                }
            }
        }

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

/// Animate alpha fade for revealable entities without child sprites (decorations).
/// These fade to `revealed_full_alpha` (e.g. 0.3) instead of swapping to a stump.
pub fn animate_reveals_simple(
    time: Res<Time>,
    mut query: Query<(&Revealable, &mut RevealState, &mut Sprite), Without<Children>>,
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
