//! Frame-based sprite animation for scenery, props and effects.
//!
//! Characters use [`crate::npc_anim`], which is direction-aware and indexes a
//! grid by facing row. This is the flat case: one horizontal strip of frames
//! that plays on a timer -- swaying flora, a waterfall, a guttering torch, a
//! banner in the wind.
//!
//! Transform and alpha tricks ([`level::grass::animate_grass_sway`] and
//! friends) remain useful as an *accessory* -- they cost no texture memory and
//! keep wang-tiled edges seamless. Reach for a strip whenever the artwork's
//! shape changes rather than just its position or opacity.

use bevy::prelude::*;

/// What happens when a strip reaches its last frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpriteAnimMode {
    /// Wrap to the first frame. The default for ambient scenery.
    #[default]
    Loop,
    /// Play back down to the first frame, then forward again. Halves the frame
    /// count for symmetric motion such as a sway or a slow pulse, and avoids
    /// the visible jump a `Loop` shows when the ends do not meet.
    PingPong,
    /// Stop on the last frame and add [`SpriteAnimDone`]. For one-shot effects.
    Once,
}

/// A horizontal run of `frames` cells beginning at atlas index `start`.
///
/// Sheets are built by `scripts/build_sheet.py --layout loop`; `start` is
/// non-zero only when several strips share one atlas.
#[derive(Component, Clone, Copy, Debug)]
pub struct SpriteAnim {
    /// Atlas index of the first frame.
    pub start: usize,
    /// Number of frames in the strip. Must be at least 1.
    pub frames: usize,
    /// Playback rate. Ambient scenery reads best slow -- see [`AMBIENT_FPS`].
    pub fps: f32,
    pub mode: SpriteAnimMode,
}

/// Frame rate for ambient scenery. Fast enough to read as alive, slow enough
/// that a screen full of foliage does not draw the eye away from the player.
pub const AMBIENT_FPS: f32 = 6.0;

impl SpriteAnim {
    /// A looping strip at [`AMBIENT_FPS`] starting at atlas index 0.
    pub fn ambient(frames: usize) -> Self {
        Self {
            start: 0,
            frames,
            fps: AMBIENT_FPS,
            mode: SpriteAnimMode::Loop,
        }
    }

    pub fn with_mode(mut self, mode: SpriteAnimMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_fps(mut self, fps: f32) -> Self {
        self.fps = fps;
        self
    }

    /// Atlas index for a frame offset within the strip.
    pub fn atlas_index(&self, frame: usize) -> usize {
        self.start + frame.min(self.frames.saturating_sub(1))
    }
}

/// Current frame offset within the strip, plus the direction `PingPong` is
/// travelling. Spawn with a non-zero `index` to de-phase identical scenery --
/// a stand of ferns swaying in lockstep reads as a rendering bug.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SpriteAnimFrame {
    pub index: usize,
    /// Only consulted by [`SpriteAnimMode::PingPong`].
    pub reversing: bool,
}

impl SpriteAnimFrame {
    /// Start `index` frames into the strip.
    pub fn offset(index: usize) -> Self {
        Self {
            index,
            reversing: false,
        }
    }
}

/// Ticks down to the next frame change.
#[derive(Component, Debug)]
pub struct SpriteAnimTimer(pub Timer);

impl SpriteAnimTimer {
    pub fn from_fps(fps: f32) -> Self {
        Self(Timer::from_seconds(1.0 / fps, TimerMode::Repeating))
    }
}

/// Added to a [`SpriteAnimMode::Once`] entity when its last frame is reached,
/// so gameplay code can despawn or swap it without polling the frame index.
#[derive(Component, Debug)]
pub struct SpriteAnimDone;

/// Components needed to play a strip. Spawn alongside a `Sprite` carrying the
/// `TextureAtlas` the strip indexes into.
pub fn sprite_anim_bundle(anim: SpriteAnim, start_frame: usize) -> impl Bundle {
    (
        anim,
        SpriteAnimFrame::offset(start_frame % anim.frames.max(1)),
        SpriteAnimTimer::from_fps(anim.fps),
    )
}
