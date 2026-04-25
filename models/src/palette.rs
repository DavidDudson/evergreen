#![allow(clippy::disallowed_methods)]

use bevy::prelude::{Color, Resource};

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

// Weather particle colors.
// Firefly is emissive (>>1.0) so it survives bevy_light_2d's multiplicative
// ambient pass at night (brightness 0.30) and still exceeds bloom threshold (1.0).
pub const FIREFLY: Color = Color::srgb(6.0, 9.0, 1.5);
pub const DUST_MOTE: Color = Color::srgb(0.9, 0.85, 0.75);
pub const FOG: Color = Color::srgba(0.6, 0.65, 0.7, 0.35);

// Base tint applied to the baked soft-edge drop shadow sprite. Alpha is
// modulated per-frame by time-of-day fade.
pub const DROP_SHADOW_TINT: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);

/// Translucent dark blue applied to fish-shadow silhouettes drifting under
/// ocean tiles.
pub const FISH_SHADOW_TINT: Color = Color::srgba(0.05, 0.08, 0.15, 0.45);

/// Soft-white tint for splash ripple sprites kicked up while the player
/// wades through shallow water. Initial alpha; fades over the splash
/// lifetime via `Sprite::with_alpha`.
pub const SPLASH_TINT: Color = Color::srgba(1.0, 1.0, 1.0, 0.55);

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

/// Runtime-overridable copy of the palette. Plugins (themes, color-blind
/// presets, mods) can replace this resource to retint the UI without
/// touching individual screens. The `pub const` defaults above remain
/// the source of truth -- this resource is initialised from them and
/// systems can opt-in to read from `Res<PaletteTheme>` for live theming.
#[derive(Resource, Clone, Debug)]
pub struct PaletteTheme {
    pub dark_bg: Color,
    pub title: Color,
    pub accent: Color,
    pub button_bg: Color,
    pub button_text: Color,
    pub overlay: Color,
    pub player: Color,
    pub minimap_bg: Color,
    pub minimap_room: Color,
    pub minimap_current: Color,
    pub minimap_npc: Color,
    pub dialog_bg: Color,
    pub dialog_border: Color,
    pub dialog_text: Color,
    pub dialog_speaker: Color,
    pub dialog_choice_bg: Color,
    pub dialog_choice_hover: Color,
    pub bark_text: Color,
    pub align_greenwoods: Color,
    pub align_darkwoods: Color,
    pub align_cities: Color,
    pub align_track: Color,
    pub align_label: Color,
    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,
    pub interact_prompt: Color,
    pub biome_city_tint: Color,
    pub biome_greenwood_tint: Color,
    pub biome_darkwood_tint: Color,
    pub level_exit: Color,
    pub perf_bg: Color,
    pub perf_good: Color,
    pub perf_warn: Color,
    pub perf_bad: Color,
    pub light_exit: Color,
    pub light_torch: Color,
    pub ambient_day: Color,
    pub ambient_dawn: Color,
    pub ambient_dusk: Color,
    pub ambient_night: Color,
    pub firefly: Color,
    pub dust_mote: Color,
    pub fog: Color,
    pub drop_shadow_tint: Color,
    pub fish_shadow_tint: Color,
}

impl Default for PaletteTheme {
    fn default() -> Self {
        Self {
            dark_bg: DARK_BG,
            title: TITLE,
            accent: ACCENT,
            button_bg: BUTTON_BG,
            button_text: BUTTON_TEXT,
            overlay: OVERLAY,
            player: PLAYER,
            minimap_bg: MINIMAP_BG,
            minimap_room: MINIMAP_ROOM,
            minimap_current: MINIMAP_CURRENT,
            minimap_npc: MINIMAP_NPC,
            dialog_bg: DIALOG_BG,
            dialog_border: DIALOG_BORDER,
            dialog_text: DIALOG_TEXT,
            dialog_speaker: DIALOG_SPEAKER,
            dialog_choice_bg: DIALOG_CHOICE_BG,
            dialog_choice_hover: DIALOG_CHOICE_HOVER,
            bark_text: BARK_TEXT,
            align_greenwoods: ALIGN_GREENWOODS,
            align_darkwoods: ALIGN_DARKWOODS,
            align_cities: ALIGN_CITIES,
            align_track: ALIGN_TRACK,
            align_label: ALIGN_LABEL,
            scrollbar_track: SCROLLBAR_TRACK,
            scrollbar_thumb: SCROLLBAR_THUMB,
            interact_prompt: INTERACT_PROMPT,
            biome_city_tint: BIOME_CITY_TINT,
            biome_greenwood_tint: BIOME_GREENWOOD_TINT,
            biome_darkwood_tint: BIOME_DARKWOOD_TINT,
            level_exit: LEVEL_EXIT,
            perf_bg: PERF_BG,
            perf_good: PERF_GOOD,
            perf_warn: PERF_WARN,
            perf_bad: PERF_BAD,
            light_exit: LIGHT_EXIT,
            light_torch: LIGHT_TORCH,
            ambient_day: AMBIENT_DAY,
            ambient_dawn: AMBIENT_DAWN,
            ambient_dusk: AMBIENT_DUSK,
            ambient_night: AMBIENT_NIGHT,
            firefly: FIREFLY,
            dust_mote: DUST_MOTE,
            fog: FOG,
            drop_shadow_tint: DROP_SHADOW_TINT,
            fish_shadow_tint: FISH_SHADOW_TINT,
        }
    }
}
