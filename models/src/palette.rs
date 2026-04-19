#![allow(clippy::disallowed_methods)]

use bevy::prelude::Color;

pub const DARK_BG: Color = Color::srgb(0.06, 0.14, 0.08);
pub const TITLE: Color = Color::srgb(0.58, 0.82, 0.52);
pub const ACCENT: Color = Color::srgb(0.24, 0.48, 0.29);
pub const BUTTON_BG: Color = Color::srgb(0.12, 0.30, 0.16);
pub const BUTTON_TEXT: Color = Color::srgb(0.72, 0.92, 0.68);
pub const OVERLAY: Color = Color::srgba(0.04, 0.12, 0.06, 0.85);
pub const PLAYER: Color = Color::srgb(0.6, 0.2, 0.8);

// Minimap colours
pub const MINIMAP_BG: Color = Color::srgba(0.04, 0.08, 0.04, 0.80);
pub const MINIMAP_ROOM: Color = Color::srgb(0.38, 0.58, 0.40);
pub const MINIMAP_CURRENT: Color = Color::srgb(0.90, 0.92, 0.72);
pub const MINIMAP_NPC: Color = Color::srgb(0.95, 0.65, 0.20);

// Dialog / lore colours
pub const DIALOG_BG: Color = Color::srgba(0.04, 0.10, 0.06, 0.95);
pub const DIALOG_BORDER: Color = Color::srgb(0.24, 0.48, 0.29);
pub const DIALOG_TEXT: Color = Color::srgb(0.82, 0.94, 0.78);
pub const DIALOG_SPEAKER: Color = Color::srgb(0.58, 0.82, 0.52);
pub const DIALOG_CHOICE_BG: Color = Color::srgb(0.10, 0.24, 0.14);
pub const DIALOG_CHOICE_HOVER: Color = Color::srgb(0.18, 0.40, 0.22);
pub const BARK_TEXT: Color = Color::srgb(0.90, 0.94, 0.72);

// Alignment bar colours
pub const ALIGN_GREENWOODS: Color = Color::srgb(0.22, 0.70, 0.30);
pub const ALIGN_DARKWOODS: Color = Color::srgb(0.45, 0.15, 0.55);
pub const ALIGN_CITIES: Color = Color::srgb(0.80, 0.65, 0.20);
pub const ALIGN_TRACK: Color = Color::srgba(0.10, 0.10, 0.10, 0.60);
pub const ALIGN_LABEL: Color = Color::srgb(0.75, 0.85, 0.72);

// Scrollbar colours
pub const SCROLLBAR_TRACK: Color = Color::srgba(0.06, 0.10, 0.06, 0.40);
pub const SCROLLBAR_THUMB: Color = Color::srgb(0.24, 0.48, 0.29);

// Interact prompt
pub const INTERACT_PROMPT: Color = Color::srgb(1.0, 0.90, 0.30);

// Biome tile tint colors (used for Minecraft-style biome blending)
pub const BIOME_CITY_TINT: Color = Color::srgb(0.95, 0.92, 0.80);
pub const BIOME_GREENWOOD_TINT: Color = Color::srgb(0.85, 0.95, 0.82);
pub const BIOME_DARKWOOD_TINT: Color = Color::srgb(0.65, 0.70, 0.62);

// General-purpose alpha constants
pub const TRANSPARENT: Color = Color::srgba(1.0, 1.0, 1.0, 0.0);
pub const OPAQUE_WHITE: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

// Level exit marker color. Emissive (>>1.0) so the goal sprite stays bright
// after the multiplicative `bevy_light_2d` lighting pass even at night
// (ambient brightness 0.30) and still triggers HDR bloom (threshold 1.0).
pub const LEVEL_EXIT: Color = Color::srgb(6.0, 5.0, 1.5);

// Performance overlay colours
pub const PERF_BG: Color = Color::srgba(0.04, 0.04, 0.04, 0.88);
pub const PERF_GOOD: Color = Color::srgb(0.20, 0.85, 0.35);
pub const PERF_WARN: Color = Color::srgb(0.95, 0.75, 0.10);
pub const PERF_BAD: Color = Color::srgb(0.90, 0.20, 0.15);

// 2D point light colors.
pub const LIGHT_EXIT: Color = Color::srgb(1.0, 0.85, 0.40);
pub const LIGHT_TORCH: Color = Color::srgb(1.0, 0.70, 0.30);

// 2D lighting ambient color anchors (per time-of-day period).
pub const AMBIENT_DAY: Color = Color::srgb(1.0, 1.0, 0.95);
pub const AMBIENT_DAWN: Color = Color::srgb(1.0, 0.85, 0.65);
pub const AMBIENT_DUSK: Color = Color::srgb(0.85, 0.55, 0.55);
pub const AMBIENT_NIGHT: Color = Color::srgb(0.4, 0.5, 0.8);

/// Linearly interpolate between two colors in linear (non-gamma) color space.
pub fn lerp_linear_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_linear();
    let b = b.to_linear();
    Color::linear_rgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}
