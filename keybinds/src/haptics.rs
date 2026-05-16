//! Tiny haptics helper: one-shot rumble on every connected gamepad.
//!
//! Use [`Haptics`] as a [`SystemParam`] and call [`Haptics::pulse`] with a
//! preset [`HapticPulse`]. The helper enumerates the gamepad query and emits
//! one [`GamepadRumbleRequest::Add`] per controller, so single-pad and
//! couch-coop both work without per-call gamepad lookup.
//!
//! WASM: the browser Gamepad API exposes rumble via `vibrationActuator` on
//! a subset of browsers (Chrome on supported pads). When unsupported, the
//! requests silently no-op.

use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;

/// Preset rumble strengths picked to feel right for short UX feedback. Use
/// the named constants on [`HapticPulse`] rather than building them inline.
#[derive(Debug, Clone, Copy)]
pub struct HapticPulse {
    pub duration_ms: u64,
    pub strong: f32,
    pub weak: f32,
}

impl HapticPulse {
    /// Subtle tick: investigation popup, hovering choices.
    pub const TAP: Self = Self {
        duration_ms: 60,
        strong: 0.0,
        weak: 0.35,
    };
    /// Single soft pulse: quest offered, item picked up.
    pub const LIGHT: Self = Self {
        duration_ms: 120,
        strong: 0.0,
        weak: 0.55,
    };
    /// Medium punch: milestone advanced, choice made.
    pub const MEDIUM: Self = Self {
        duration_ms: 200,
        strong: 0.5,
        weak: 0.5,
    };
    /// Long emphatic rumble: quest completed, level finished.
    pub const STRONG: Self = Self {
        duration_ms: 450,
        strong: 0.9,
        weak: 0.7,
    };
}

/// Grouped query + writer so systems can request rumble with a single
/// [`SystemParam`] field.
#[derive(SystemParam)]
pub struct Haptics<'w, 's> {
    pub gamepads: Query<'w, 's, Entity, With<Gamepad>>,
    pub writer: MessageWriter<'w, GamepadRumbleRequest>,
}

impl Haptics<'_, '_> {
    /// Fire `pulse` on every connected gamepad. No-op when no controllers
    /// are connected.
    pub fn pulse(&mut self, pulse: HapticPulse) {
        let duration = Duration::from_millis(pulse.duration_ms);
        let intensity = GamepadRumbleIntensity {
            strong_motor: pulse.strong.clamp(0.0, 1.0),
            weak_motor: pulse.weak.clamp(0.0, 1.0),
        };
        for gamepad in self.gamepads.iter() {
            self.writer.write(GamepadRumbleRequest::Add {
                duration,
                intensity,
                gamepad,
            });
        }
    }
}
