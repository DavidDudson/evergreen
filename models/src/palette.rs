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

// General-purpose alpha constants
pub const TRANSPARENT: Color = Color::srgba(1.0, 1.0, 1.0, 0.0);
pub const OPAQUE_WHITE: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

// Performance overlay colours
pub const PERF_BG: Color = Color::srgba(0.04, 0.04, 0.04, 0.88);
pub const PERF_GOOD: Color = Color::srgb(0.20, 0.85, 0.35);
pub const PERF_WARN: Color = Color::srgb(0.95, 0.75, 0.10);
pub const PERF_BAD: Color = Color::srgb(0.90, 0.20, 0.15);
