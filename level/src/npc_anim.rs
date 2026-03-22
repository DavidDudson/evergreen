//! NPC sprite-sheet animation systems.
//!
//! Ticks [`NpcAnimTimer`], advances [`NpcAnimFrame`], and updates the
//! `TextureAtlas` index to display the correct frame.

use bevy::prelude::*;
use models::npc_anim::{NpcAnimFrame, NpcAnimKind, NpcAnimTimer, NpcFacing, NpcSheet};

/// Advance the frame counter and update the sprite atlas index each tick.
pub fn advance_npc_frame(
    time: Res<Time>,
    mut query: Query<(
        &mut Sprite,
        &NpcFacing,
        &NpcAnimKind,
        &NpcSheet,
        &mut NpcAnimFrame,
        &mut NpcAnimTimer,
    )>,
) {
    for (mut sprite, facing, kind, sheet, mut frame, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if !timer.0.just_finished() {
            continue;
        }

        let count = sheet.frame_count(*kind);
        frame.0 = (frame.0 + 1) % count;

        let index = facing.row() * sheet.cols + sheet.col_start(*kind) + frame.0;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = index;
        }
    }
}

/// Reset frame counter and timer speed when [`NpcAnimKind`] changes.
pub fn reset_npc_anim_on_change(
    mut query: Query<
        (&NpcAnimKind, &mut NpcAnimFrame, &mut NpcAnimTimer),
        Changed<NpcAnimKind>,
    >,
) {
    for (kind, mut frame, mut timer) in &mut query {
        frame.0 = 0;
        timer.0 = Timer::from_seconds(1.0 / kind.fps(), TimerMode::Repeating);
    }
}
