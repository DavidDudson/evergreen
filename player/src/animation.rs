use bevy::prelude::*;
use keybinds::Keybinds;
use std::time::Duration;

use crate::input::{is_sprinting, read_movement_input};

pub const SHEET_COLS: usize = 12; // 4 idle + 4 walk + 4 run
pub const SHEET_ROWS: usize = 8; // S, SW, W, NW, N, NE, E, SE
pub const FRAME_W_PX: u32 = 32;
pub const FRAME_H_PX: u32 = 64;

const IDLE_FRAMES: usize = 4;
const WALK_FRAMES: usize = 4;
const RUN_FRAMES: usize = 4;
const IDLE_FPS: f32 = 3.0;
const WALK_FPS: f32 = 8.0;
const RUN_FPS: f32 = 12.0;

const DIRECTION_COUNT: u8 = 8;

/// Eight-way facing for the player sprite. The discriminants are the row
/// index into the sprite atlas, so `From<FacingDirection> for u8` and
/// `TryFrom<u8>` provide direct row math without `match` ladders.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FacingDirection {
    #[default]
    South = 0,
    SouthWest = 1,
    West = 2,
    NorthWest = 3,
    North = 4,
    NorthEast = 5,
    East = 6,
    SouthEast = 7,
}

impl From<FacingDirection> for u8 {
    fn from(dir: FacingDirection) -> Self {
        match dir {
            FacingDirection::South => 0,
            FacingDirection::SouthWest => 1,
            FacingDirection::West => 2,
            FacingDirection::NorthWest => 3,
            FacingDirection::North => 4,
            FacingDirection::NorthEast => 5,
            FacingDirection::East => 6,
            FacingDirection::SouthEast => 7,
        }
    }
}

impl TryFrom<u8> for FacingDirection {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::South),
            1 => Ok(Self::SouthWest),
            2 => Ok(Self::West),
            3 => Ok(Self::NorthWest),
            4 => Ok(Self::North),
            5 => Ok(Self::NorthEast),
            6 => Ok(Self::East),
            7 => Ok(Self::SouthEast),
            other => Err(other),
        }
    }
}

impl FacingDirection {
    /// Sprite-atlas row index for this direction.
    pub fn row(self) -> usize {
        usize::from(u8::from(self))
    }

    /// Map a (non-zero) velocity vector to the nearest of the eight facings.
    ///
    /// East is octant 0 in atan2 space, then we walk counter-clockwise. The
    /// table below (indexed by octant) gives the `FacingDirection` ordinal so
    /// we can build the result via `TryFrom<u8>` without a second match arm.
    pub fn from_velocity(v: Vec2) -> Self {
        // octant index 0..=7, where 0 is +X (East) and increments rotate CCW.
        // f32 -> i32 has no `From` impl; the `.round()` value is bounded to
        // a small integer range by `rem_euclid(8)` immediately afterward.
        #[allow(clippy::as_conversions)] // f32->i32 rounded; bounded by rem_euclid(8) below
        let octant_signed = (v.y.atan2(v.x) / std::f32::consts::FRAC_PI_4).round() as i32;
        let octant = u8::try_from(octant_signed.rem_euclid(i32::from(DIRECTION_COUNT)))
            .expect("rem_euclid(8) is always 0..=7");

        // Octant -> FacingDirection ordinal. East = 6 in the row order, then
        // CCW: NE=5, N=4, NW=3, W=2, SW=1, S=0, SE=7.
        const OCTANT_TO_ORDINAL: [u8; 8] = [6, 5, 4, 3, 2, 1, 0, 7];
        let ordinal = OCTANT_TO_ORDINAL[usize::from(octant)];
        Self::try_from(ordinal).expect("OCTANT_TO_ORDINAL values are 0..=7")
    }
}

/// Which animation strip is currently playing on the sprite. Drives FPS,
/// frame count, and column offset into the atlas.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    #[default]
    Idle,
    Walk,
    Run,
}

impl AnimationKind {
    fn fps(self) -> f32 {
        match self {
            Self::Idle => IDLE_FPS,
            Self::Walk => WALK_FPS,
            Self::Run => RUN_FPS,
        }
    }

    fn frame_count(self) -> usize {
        match self {
            Self::Idle => IDLE_FRAMES,
            Self::Walk => WALK_FRAMES,
            Self::Run => RUN_FRAMES,
        }
    }

    fn col_start(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Walk => 4,
            Self::Run => 8,
        }
    }
}

/// Logical movement state of the player, separated from the render-side
/// `AnimationKind` so movement systems don't need to know about sprite
/// strips. `update_animation_state` writes both in lock-step.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    #[default]
    Idle,
    Walk,
    Run,
}

#[derive(Component, Default)]
pub struct AnimationFrame(pub usize);

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0 / WALK_FPS, TimerMode::Repeating))
    }
}

/// Updates facing direction, animation kind, and movement state from input.
pub fn update_animation_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    mut query: Query<(
        &mut FacingDirection,
        &mut AnimationKind,
        &mut MovementState,
        &mut AnimationFrame,
        &mut AnimationTimer,
    )>,
) {
    let Ok((mut facing, mut kind, mut movement, mut frame, mut timer)) = query.single_mut() else {
        return;
    };

    let velocity = read_movement_input(&keyboard, &bindings);
    let sprinting = is_sprinting(&keyboard, &bindings);

    let (new_kind, new_movement) = if velocity == Vec2::ZERO {
        (AnimationKind::Idle, MovementState::Idle)
    } else if sprinting {
        (AnimationKind::Run, MovementState::Run)
    } else {
        (AnimationKind::Walk, MovementState::Walk)
    };

    if new_kind != *kind {
        *kind = new_kind;
        timer
            .0
            .set_duration(Duration::from_secs_f32(1.0 / new_kind.fps()));
        timer.0.reset();
        frame.0 = 0;
    }

    if new_movement != *movement {
        *movement = new_movement;
    }

    if velocity != Vec2::ZERO {
        *facing = FacingDirection::from_velocity(velocity.normalize());
    }
}

/// Ticks the animation timer and updates the sprite atlas index.
pub fn advance_frame(
    time: Res<Time>,
    mut query: Query<(
        &FacingDirection,
        &AnimationKind,
        &mut AnimationFrame,
        &mut AnimationTimer,
        &mut Sprite,
    )>,
) {
    let Ok((facing, kind, mut frame, mut timer, mut sprite)) = query.single_mut() else {
        return;
    };

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        frame.0 = (frame.0 + 1) % kind.frame_count();
    }

    let index = facing.row() * SHEET_COLS + kind.col_start() + frame.0;
    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        atlas.index = index;
    }
}
