//! Animation components for NPC entities (4-cardinal direction sheets).

use bevy::prelude::*;

/// Cardinal facing direction for NPCs.
///
/// Row indices match PixelLab's 4-direction output order.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum NpcFacing {
    #[default]
    South,
    East,
    North,
    West,
}

impl NpcFacing {
    /// Row index in the sprite sheet.
    pub fn row(self) -> usize {
        match self {
            Self::South => 0,
            Self::East => 1,
            Self::North => 2,
            Self::West => 3,
        }
    }

    /// Snap a world-space direction vector to the nearest cardinal.
    pub fn from_vec2(v: Vec2) -> Self {
        if v.x.abs() > v.y.abs() {
            if v.x > 0.0 {
                Self::East
            } else {
                Self::West
            }
        } else if v.y > 0.0 {
            Self::North
        } else {
            Self::South
        }
    }
}

/// NPC animation type.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum NpcAnimKind {
    #[default]
    Idle,
    Walk,
}

const IDLE_FPS: f32 = 4.0;
const WALK_FPS: f32 = 8.0;

impl NpcAnimKind {
    pub fn fps(self) -> f32 {
        match self {
            Self::Idle => IDLE_FPS,
            Self::Walk => WALK_FPS,
        }
    }
}

/// Describes the sprite sheet layout — frame counts may vary per character.
#[derive(Component, Clone, Copy)]
pub struct NpcSheet {
    pub idle_frames: usize,
    pub walk_frames: usize,
    pub cols: usize,
}

impl NpcSheet {
    pub fn col_start(self, kind: NpcAnimKind) -> usize {
        match kind {
            NpcAnimKind::Idle => 0,
            NpcAnimKind::Walk => self.idle_frames,
        }
    }

    pub fn frame_count(self, kind: NpcAnimKind) -> usize {
        match kind {
            NpcAnimKind::Idle => self.idle_frames,
            NpcAnimKind::Walk => self.walk_frames,
        }
    }
}

impl Default for NpcSheet {
    fn default() -> Self {
        Self {
            idle_frames: 8,
            walk_frames: 4,
            cols: 12,
        }
    }
}

/// Current animation frame index (0..frame_count).
#[derive(Component, Default)]
pub struct NpcAnimFrame(pub usize);

/// Timer that drives frame advancement.
#[derive(Component)]
pub struct NpcAnimTimer(pub Timer);

impl Default for NpcAnimTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0 / IDLE_FPS, TimerMode::Repeating))
    }
}
