use bevy::prelude::*;
use std::time::Duration;

pub const SHEET_COLS: usize = 12; // 4 idle + 4 walk + 4 run
pub const SHEET_ROWS: usize = 8; // S, SW, W, NW, N, NE, E, SE
pub const FRAME_W_PX: u32 = 32;
pub const FRAME_H_PX: u32 = 64;

const IDLE_FRAMES: usize = 4;
const WALK_FRAMES: usize = 4;
const RUN_FRAMES: usize = 4;
const IDLE_FPS: f32 = 4.0;
const WALK_FPS: f32 = 8.0;
const RUN_FPS: f32 = 12.0;

#[derive(Component, Default, Clone, Copy, PartialEq)]
pub enum FacingDirection {
    #[default]
    South,
    SouthWest,
    West,
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
}

impl FacingDirection {
    pub fn row(self) -> usize {
        match self {
            Self::South => 0,
            Self::SouthWest => 1,
            Self::West => 2,
            Self::NorthWest => 3,
            Self::North => 4,
            Self::NorthEast => 5,
            Self::East => 6,
            Self::SouthEast => 7,
        }
    }

    pub fn from_velocity(v: Vec2) -> Self {
        let angle = v.y.atan2(v.x);
        #[allow(clippy::as_conversions)] // f32→i32: no From impl; value is a small rounded int
        let octant = (angle / std::f32::consts::FRAC_PI_4).round() as i32;
        let octant =
            usize::try_from(octant.rem_euclid(8)).expect("rem_euclid(8) is always 0..=7");
        match octant {
            0 => Self::East,
            1 => Self::NorthEast,
            2 => Self::North,
            3 => Self::NorthWest,
            4 => Self::West,
            5 => Self::SouthWest,
            6 => Self::South,
            7 => Self::SouthEast,
            _ => unreachable!(),
        }
    }
}

#[derive(Component, Default, Clone, Copy, PartialEq)]
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

#[derive(Component, Default)]
pub struct AnimationFrame(pub usize);

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0 / WALK_FPS, TimerMode::Repeating))
    }
}

/// Updates facing direction and animation kind from keyboard input.
pub fn update_animation_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut FacingDirection,
        &mut AnimationKind,
        &mut AnimationFrame,
        &mut AnimationTimer,
    )>,
) {
    let Ok((mut facing, mut kind, mut frame, mut timer)) = query.single_mut() else {
        return;
    };

    let velocity: Vec2 = [
        (KeyCode::KeyW, Vec2::Y),
        (KeyCode::ArrowUp, Vec2::Y),
        (KeyCode::KeyS, Vec2::NEG_Y),
        (KeyCode::ArrowDown, Vec2::NEG_Y),
        (KeyCode::KeyA, Vec2::NEG_X),
        (KeyCode::ArrowLeft, Vec2::NEG_X),
        (KeyCode::KeyD, Vec2::X),
        (KeyCode::ArrowRight, Vec2::X),
    ]
    .iter()
    .filter(|(key, _)| keyboard.pressed(*key))
    .map(|(_, dir)| *dir)
    .sum();

    let is_sprinting =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    let new_kind = if velocity == Vec2::ZERO {
        AnimationKind::Idle
    } else if is_sprinting {
        AnimationKind::Run
    } else {
        AnimationKind::Walk
    };

    if new_kind != *kind {
        *kind = new_kind;
        timer
            .0
            .set_duration(Duration::from_secs_f32(1.0 / new_kind.fps()));
        timer.0.reset();
        frame.0 = 0;
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
