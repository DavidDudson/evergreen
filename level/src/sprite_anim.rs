//! Strip-animation system for scenery, props and effects.
//!
//! Ticks [`SpriteAnimTimer`], walks [`SpriteAnimFrame`] according to the
//! strip's [`SpriteAnimMode`], and writes the result to the sprite's atlas
//! index. The character equivalent is [`crate::npc_anim`].

use bevy::prelude::*;
use models::sprite_anim::{
    SpriteAnim, SpriteAnimDone, SpriteAnimFrame, SpriteAnimMode, SpriteAnimTimer,
};

/// Advance every playing strip and update its atlas index.
pub fn advance_sprite_anim(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Sprite,
            &SpriteAnim,
            &mut SpriteAnimFrame,
            &mut SpriteAnimTimer,
        ),
        Without<SpriteAnimDone>,
    >,
) {
    for (entity, mut sprite, anim, mut frame, mut timer) in &mut query {
        // A one-frame strip is a still sprite; ticking it just burns time.
        if anim.frames <= 1 {
            continue;
        }

        timer.0.tick(time.delta());
        if !timer.0.just_finished() {
            continue;
        }

        let last = anim.frames - 1;
        match anim.mode {
            SpriteAnimMode::Loop => frame.index = (frame.index + 1) % anim.frames,
            SpriteAnimMode::PingPong => {
                if frame.reversing {
                    if frame.index == 0 {
                        frame.reversing = false;
                        frame.index = 1;
                    } else {
                        frame.index -= 1;
                    }
                } else if frame.index >= last {
                    frame.reversing = true;
                    frame.index = last - 1;
                } else {
                    frame.index += 1;
                }
            }
            SpriteAnimMode::Once => {
                if frame.index >= last {
                    commands.entity(entity).insert(SpriteAnimDone);
                    continue;
                }
                frame.index += 1;
            }
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = anim.atlas_index(frame.index);
        }
    }
}

/// Re-time the strip when its [`SpriteAnim`] is swapped at runtime.
pub fn resync_sprite_anim_on_change(
    mut query: Query<(&SpriteAnim, &mut SpriteAnimTimer), Changed<SpriteAnim>>,
) {
    for (anim, mut timer) in &mut query {
        timer.0 = Timer::from_seconds(1.0 / anim.fps, TimerMode::Repeating);
    }
}
