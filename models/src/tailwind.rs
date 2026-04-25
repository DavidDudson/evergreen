//! Tailwind CSS v4 design tokens (colors, spacing, typography, radius).
//!
//! This module is the single source of truth for the visual design vocabulary.
//! UI screens should reference these constants rather than defining their own
//! pixel values or color literals so that retheming + scale tweaks land in
//! one place.
//!
//! Naming follows Tailwind v4 conventions:
//!   - colors: `<family>_<shade>` (e.g. `SLATE_500`, `EMERALD_700`).
//!   - spacing: `SPACE_<n>` where `n` is the Tailwind step (1 step = 4px,
//!     so `SPACE_4 = 16px`).
//!   - radius: `RADIUS_*` for the named scale (sm/base/md/lg/xl/2xl/3xl/full).
//!   - typography: `TEXT_*_PX` for the named scale (xs/sm/base/lg/xl/...).
//!
//! Tailwind v4 default colour values are reproduced verbatim (sRGB) and may
//! drift from upstream if Tailwind ships a future palette tweak. Update this
//! file by re-running the official palette through a `#RRGGBB -> srgb f32`
//! conversion.

#![allow(clippy::disallowed_methods)] // palette tokens construct sRGB literals.

use bevy::prelude::Color;

// ---------------------------------------------------------------------------
// Spacing scale -- Tailwind v4 default (`--spacing: 0.25rem`).
// Pixel values assume a 16px base. Use these for `Val::Px(...)` and node
// padding/margin/gap.
// ---------------------------------------------------------------------------

pub const SPACE_0_PX: f32 = 0.0;
pub const SPACE_0_5_PX: f32 = 2.0;
pub const SPACE_1_PX: f32 = 4.0;
pub const SPACE_1_5_PX: f32 = 6.0;
pub const SPACE_2_PX: f32 = 8.0;
pub const SPACE_2_5_PX: f32 = 10.0;
pub const SPACE_3_PX: f32 = 12.0;
pub const SPACE_3_5_PX: f32 = 14.0;
pub const SPACE_4_PX: f32 = 16.0;
pub const SPACE_5_PX: f32 = 20.0;
pub const SPACE_6_PX: f32 = 24.0;
pub const SPACE_7_PX: f32 = 28.0;
pub const SPACE_8_PX: f32 = 32.0;
pub const SPACE_9_PX: f32 = 36.0;
pub const SPACE_10_PX: f32 = 40.0;
pub const SPACE_11_PX: f32 = 44.0;
pub const SPACE_12_PX: f32 = 48.0;
pub const SPACE_14_PX: f32 = 56.0;
pub const SPACE_16_PX: f32 = 64.0;
pub const SPACE_20_PX: f32 = 80.0;
pub const SPACE_24_PX: f32 = 96.0;
pub const SPACE_28_PX: f32 = 112.0;
pub const SPACE_32_PX: f32 = 128.0;
pub const SPACE_40_PX: f32 = 160.0;
pub const SPACE_48_PX: f32 = 192.0;
pub const SPACE_56_PX: f32 = 224.0;
pub const SPACE_64_PX: f32 = 256.0;
pub const SPACE_72_PX: f32 = 288.0;
pub const SPACE_80_PX: f32 = 320.0;
pub const SPACE_96_PX: f32 = 384.0;

// ---------------------------------------------------------------------------
// Border radius (Tailwind v4 default scale).
// ---------------------------------------------------------------------------

pub const RADIUS_NONE_PX: f32 = 0.0;
pub const RADIUS_XS_PX: f32 = 2.0;
pub const RADIUS_SM_PX: f32 = 4.0;
pub const RADIUS_BASE_PX: f32 = 6.0;
pub const RADIUS_MD_PX: f32 = 8.0;
pub const RADIUS_LG_PX: f32 = 10.0;
pub const RADIUS_XL_PX: f32 = 12.0;
pub const RADIUS_2XL_PX: f32 = 16.0;
pub const RADIUS_3XL_PX: f32 = 24.0;
pub const RADIUS_4XL_PX: f32 = 32.0;
pub const RADIUS_FULL_PX: f32 = 9999.0;

// ---------------------------------------------------------------------------
// Typography scale (Tailwind v4 default `text-*` sizes, sRGB pixels).
// ---------------------------------------------------------------------------

pub const TEXT_XS_PX: f32 = 12.0;
pub const TEXT_SM_PX: f32 = 14.0;
pub const TEXT_BASE_PX: f32 = 16.0;
pub const TEXT_LG_PX: f32 = 18.0;
pub const TEXT_XL_PX: f32 = 20.0;
pub const TEXT_2XL_PX: f32 = 24.0;
pub const TEXT_3XL_PX: f32 = 30.0;
pub const TEXT_4XL_PX: f32 = 36.0;
pub const TEXT_5XL_PX: f32 = 48.0;
pub const TEXT_6XL_PX: f32 = 60.0;
pub const TEXT_7XL_PX: f32 = 72.0;
pub const TEXT_8XL_PX: f32 = 96.0;
pub const TEXT_9XL_PX: f32 = 128.0;

// ---------------------------------------------------------------------------
// Color palette (Tailwind v4 default colors). Helper to convert hex.
// ---------------------------------------------------------------------------

const fn hex(hex: u32) -> Color {
    // const-friendly hex unpack. `as` casts are unavoidable here -- there
    // is no const `From<u32>` for u8/f32 in stable Rust.
    #[allow(clippy::as_conversions)]
    let r = ((hex >> 16) & 0xFF) as u8;
    #[allow(clippy::as_conversions)]
    let g = ((hex >> 8) & 0xFF) as u8;
    #[allow(clippy::as_conversions)]
    let b = (hex & 0xFF) as u8;
    #[allow(clippy::as_conversions)]
    let rf = r as f32 / 255.0;
    #[allow(clippy::as_conversions)]
    let gf = g as f32 / 255.0;
    #[allow(clippy::as_conversions)]
    let bf = b as f32 / 255.0;
    Color::srgb(rf, gf, bf)
}

// Slate
pub const SLATE_50: Color = hex(0xf8fafc);
pub const SLATE_100: Color = hex(0xf1f5f9);
pub const SLATE_200: Color = hex(0xe2e8f0);
pub const SLATE_300: Color = hex(0xcbd5e1);
pub const SLATE_400: Color = hex(0x94a3b8);
pub const SLATE_500: Color = hex(0x64748b);
pub const SLATE_600: Color = hex(0x475569);
pub const SLATE_700: Color = hex(0x334155);
pub const SLATE_800: Color = hex(0x1e293b);
pub const SLATE_900: Color = hex(0x0f172a);
pub const SLATE_950: Color = hex(0x020617);

// Gray
pub const GRAY_50: Color = hex(0xf9fafb);
pub const GRAY_100: Color = hex(0xf3f4f6);
pub const GRAY_200: Color = hex(0xe5e7eb);
pub const GRAY_300: Color = hex(0xd1d5db);
pub const GRAY_400: Color = hex(0x9ca3af);
pub const GRAY_500: Color = hex(0x6b7280);
pub const GRAY_600: Color = hex(0x4b5563);
pub const GRAY_700: Color = hex(0x374151);
pub const GRAY_800: Color = hex(0x1f2937);
pub const GRAY_900: Color = hex(0x111827);
pub const GRAY_950: Color = hex(0x030712);

// Zinc
pub const ZINC_50: Color = hex(0xfafafa);
pub const ZINC_100: Color = hex(0xf4f4f5);
pub const ZINC_200: Color = hex(0xe4e4e7);
pub const ZINC_300: Color = hex(0xd4d4d8);
pub const ZINC_400: Color = hex(0xa1a1aa);
pub const ZINC_500: Color = hex(0x71717a);
pub const ZINC_600: Color = hex(0x52525b);
pub const ZINC_700: Color = hex(0x3f3f46);
pub const ZINC_800: Color = hex(0x27272a);
pub const ZINC_900: Color = hex(0x18181b);
pub const ZINC_950: Color = hex(0x09090b);

// Neutral
pub const NEUTRAL_50: Color = hex(0xfafafa);
pub const NEUTRAL_100: Color = hex(0xf5f5f5);
pub const NEUTRAL_200: Color = hex(0xe5e5e5);
pub const NEUTRAL_300: Color = hex(0xd4d4d4);
pub const NEUTRAL_400: Color = hex(0xa3a3a3);
pub const NEUTRAL_500: Color = hex(0x737373);
pub const NEUTRAL_600: Color = hex(0x525252);
pub const NEUTRAL_700: Color = hex(0x404040);
pub const NEUTRAL_800: Color = hex(0x262626);
pub const NEUTRAL_900: Color = hex(0x171717);
pub const NEUTRAL_950: Color = hex(0x0a0a0a);

// Stone
pub const STONE_50: Color = hex(0xfafaf9);
pub const STONE_100: Color = hex(0xf5f5f4);
pub const STONE_200: Color = hex(0xe7e5e4);
pub const STONE_300: Color = hex(0xd6d3d1);
pub const STONE_400: Color = hex(0xa8a29e);
pub const STONE_500: Color = hex(0x78716c);
pub const STONE_600: Color = hex(0x57534e);
pub const STONE_700: Color = hex(0x44403c);
pub const STONE_800: Color = hex(0x292524);
pub const STONE_900: Color = hex(0x1c1917);
pub const STONE_950: Color = hex(0x0c0a09);

// Red
pub const RED_50: Color = hex(0xfef2f2);
pub const RED_100: Color = hex(0xfee2e2);
pub const RED_200: Color = hex(0xfecaca);
pub const RED_300: Color = hex(0xfca5a5);
pub const RED_400: Color = hex(0xf87171);
pub const RED_500: Color = hex(0xef4444);
pub const RED_600: Color = hex(0xdc2626);
pub const RED_700: Color = hex(0xb91c1c);
pub const RED_800: Color = hex(0x991b1b);
pub const RED_900: Color = hex(0x7f1d1d);
pub const RED_950: Color = hex(0x450a0a);

// Orange
pub const ORANGE_50: Color = hex(0xfff7ed);
pub const ORANGE_100: Color = hex(0xffedd5);
pub const ORANGE_200: Color = hex(0xfed7aa);
pub const ORANGE_300: Color = hex(0xfdba74);
pub const ORANGE_400: Color = hex(0xfb923c);
pub const ORANGE_500: Color = hex(0xf97316);
pub const ORANGE_600: Color = hex(0xea580c);
pub const ORANGE_700: Color = hex(0xc2410c);
pub const ORANGE_800: Color = hex(0x9a3412);
pub const ORANGE_900: Color = hex(0x7c2d12);
pub const ORANGE_950: Color = hex(0x431407);

// Amber
pub const AMBER_50: Color = hex(0xfffbeb);
pub const AMBER_100: Color = hex(0xfef3c7);
pub const AMBER_200: Color = hex(0xfde68a);
pub const AMBER_300: Color = hex(0xfcd34d);
pub const AMBER_400: Color = hex(0xfbbf24);
pub const AMBER_500: Color = hex(0xf59e0b);
pub const AMBER_600: Color = hex(0xd97706);
pub const AMBER_700: Color = hex(0xb45309);
pub const AMBER_800: Color = hex(0x92400e);
pub const AMBER_900: Color = hex(0x78350f);
pub const AMBER_950: Color = hex(0x451a03);

// Yellow
pub const YELLOW_50: Color = hex(0xfefce8);
pub const YELLOW_100: Color = hex(0xfef9c3);
pub const YELLOW_200: Color = hex(0xfef08a);
pub const YELLOW_300: Color = hex(0xfde047);
pub const YELLOW_400: Color = hex(0xfacc15);
pub const YELLOW_500: Color = hex(0xeab308);
pub const YELLOW_600: Color = hex(0xca8a04);
pub const YELLOW_700: Color = hex(0xa16207);
pub const YELLOW_800: Color = hex(0x854d0e);
pub const YELLOW_900: Color = hex(0x713f12);
pub const YELLOW_950: Color = hex(0x422006);

// Lime
pub const LIME_50: Color = hex(0xf7fee7);
pub const LIME_100: Color = hex(0xecfccb);
pub const LIME_200: Color = hex(0xd9f99d);
pub const LIME_300: Color = hex(0xbef264);
pub const LIME_400: Color = hex(0xa3e635);
pub const LIME_500: Color = hex(0x84cc16);
pub const LIME_600: Color = hex(0x65a30d);
pub const LIME_700: Color = hex(0x4d7c0f);
pub const LIME_800: Color = hex(0x3f6212);
pub const LIME_900: Color = hex(0x365314);
pub const LIME_950: Color = hex(0x1a2e05);

// Green
pub const GREEN_50: Color = hex(0xf0fdf4);
pub const GREEN_100: Color = hex(0xdcfce7);
pub const GREEN_200: Color = hex(0xbbf7d0);
pub const GREEN_300: Color = hex(0x86efac);
pub const GREEN_400: Color = hex(0x4ade80);
pub const GREEN_500: Color = hex(0x22c55e);
pub const GREEN_600: Color = hex(0x16a34a);
pub const GREEN_700: Color = hex(0x15803d);
pub const GREEN_800: Color = hex(0x166534);
pub const GREEN_900: Color = hex(0x14532d);
pub const GREEN_950: Color = hex(0x052e16);

// Emerald
pub const EMERALD_50: Color = hex(0xecfdf5);
pub const EMERALD_100: Color = hex(0xd1fae5);
pub const EMERALD_200: Color = hex(0xa7f3d0);
pub const EMERALD_300: Color = hex(0x6ee7b7);
pub const EMERALD_400: Color = hex(0x34d399);
pub const EMERALD_500: Color = hex(0x10b981);
pub const EMERALD_600: Color = hex(0x059669);
pub const EMERALD_700: Color = hex(0x047857);
pub const EMERALD_800: Color = hex(0x065f46);
pub const EMERALD_900: Color = hex(0x064e3b);
pub const EMERALD_950: Color = hex(0x022c22);

// Teal
pub const TEAL_50: Color = hex(0xf0fdfa);
pub const TEAL_100: Color = hex(0xccfbf1);
pub const TEAL_200: Color = hex(0x99f6e4);
pub const TEAL_300: Color = hex(0x5eead4);
pub const TEAL_400: Color = hex(0x2dd4bf);
pub const TEAL_500: Color = hex(0x14b8a6);
pub const TEAL_600: Color = hex(0x0d9488);
pub const TEAL_700: Color = hex(0x0f766e);
pub const TEAL_800: Color = hex(0x115e59);
pub const TEAL_900: Color = hex(0x134e4a);
pub const TEAL_950: Color = hex(0x042f2e);

// Cyan
pub const CYAN_50: Color = hex(0xecfeff);
pub const CYAN_100: Color = hex(0xcffafe);
pub const CYAN_200: Color = hex(0xa5f3fc);
pub const CYAN_300: Color = hex(0x67e8f9);
pub const CYAN_400: Color = hex(0x22d3ee);
pub const CYAN_500: Color = hex(0x06b6d4);
pub const CYAN_600: Color = hex(0x0891b2);
pub const CYAN_700: Color = hex(0x0e7490);
pub const CYAN_800: Color = hex(0x155e75);
pub const CYAN_900: Color = hex(0x164e63);
pub const CYAN_950: Color = hex(0x083344);

// Sky
pub const SKY_50: Color = hex(0xf0f9ff);
pub const SKY_100: Color = hex(0xe0f2fe);
pub const SKY_200: Color = hex(0xbae6fd);
pub const SKY_300: Color = hex(0x7dd3fc);
pub const SKY_400: Color = hex(0x38bdf8);
pub const SKY_500: Color = hex(0x0ea5e9);
pub const SKY_600: Color = hex(0x0284c7);
pub const SKY_700: Color = hex(0x0369a1);
pub const SKY_800: Color = hex(0x075985);
pub const SKY_900: Color = hex(0x0c4a6e);
pub const SKY_950: Color = hex(0x082f49);

// Blue
pub const BLUE_50: Color = hex(0xeff6ff);
pub const BLUE_100: Color = hex(0xdbeafe);
pub const BLUE_200: Color = hex(0xbfdbfe);
pub const BLUE_300: Color = hex(0x93c5fd);
pub const BLUE_400: Color = hex(0x60a5fa);
pub const BLUE_500: Color = hex(0x3b82f6);
pub const BLUE_600: Color = hex(0x2563eb);
pub const BLUE_700: Color = hex(0x1d4ed8);
pub const BLUE_800: Color = hex(0x1e40af);
pub const BLUE_900: Color = hex(0x1e3a8a);
pub const BLUE_950: Color = hex(0x172554);

// Indigo
pub const INDIGO_50: Color = hex(0xeef2ff);
pub const INDIGO_100: Color = hex(0xe0e7ff);
pub const INDIGO_200: Color = hex(0xc7d2fe);
pub const INDIGO_300: Color = hex(0xa5b4fc);
pub const INDIGO_400: Color = hex(0x818cf8);
pub const INDIGO_500: Color = hex(0x6366f1);
pub const INDIGO_600: Color = hex(0x4f46e5);
pub const INDIGO_700: Color = hex(0x4338ca);
pub const INDIGO_800: Color = hex(0x3730a3);
pub const INDIGO_900: Color = hex(0x312e81);
pub const INDIGO_950: Color = hex(0x1e1b4b);

// Violet
pub const VIOLET_50: Color = hex(0xf5f3ff);
pub const VIOLET_100: Color = hex(0xede9fe);
pub const VIOLET_200: Color = hex(0xddd6fe);
pub const VIOLET_300: Color = hex(0xc4b5fd);
pub const VIOLET_400: Color = hex(0xa78bfa);
pub const VIOLET_500: Color = hex(0x8b5cf6);
pub const VIOLET_600: Color = hex(0x7c3aed);
pub const VIOLET_700: Color = hex(0x6d28d9);
pub const VIOLET_800: Color = hex(0x5b21b6);
pub const VIOLET_900: Color = hex(0x4c1d95);
pub const VIOLET_950: Color = hex(0x2e1065);

// Purple
pub const PURPLE_50: Color = hex(0xfaf5ff);
pub const PURPLE_100: Color = hex(0xf3e8ff);
pub const PURPLE_200: Color = hex(0xe9d5ff);
pub const PURPLE_300: Color = hex(0xd8b4fe);
pub const PURPLE_400: Color = hex(0xc084fc);
pub const PURPLE_500: Color = hex(0xa855f7);
pub const PURPLE_600: Color = hex(0x9333ea);
pub const PURPLE_700: Color = hex(0x7e22ce);
pub const PURPLE_800: Color = hex(0x6b21a8);
pub const PURPLE_900: Color = hex(0x581c87);
pub const PURPLE_950: Color = hex(0x3b0764);

// Fuchsia
pub const FUCHSIA_50: Color = hex(0xfdf4ff);
pub const FUCHSIA_100: Color = hex(0xfae8ff);
pub const FUCHSIA_200: Color = hex(0xf5d0fe);
pub const FUCHSIA_300: Color = hex(0xf0abfc);
pub const FUCHSIA_400: Color = hex(0xe879f9);
pub const FUCHSIA_500: Color = hex(0xd946ef);
pub const FUCHSIA_600: Color = hex(0xc026d3);
pub const FUCHSIA_700: Color = hex(0xa21caf);
pub const FUCHSIA_800: Color = hex(0x86198f);
pub const FUCHSIA_900: Color = hex(0x701a75);
pub const FUCHSIA_950: Color = hex(0x4a044e);

// Pink
pub const PINK_50: Color = hex(0xfdf2f8);
pub const PINK_100: Color = hex(0xfce7f3);
pub const PINK_200: Color = hex(0xfbcfe8);
pub const PINK_300: Color = hex(0xf9a8d4);
pub const PINK_400: Color = hex(0xf472b6);
pub const PINK_500: Color = hex(0xec4899);
pub const PINK_600: Color = hex(0xdb2777);
pub const PINK_700: Color = hex(0xbe185d);
pub const PINK_800: Color = hex(0x9d174d);
pub const PINK_900: Color = hex(0x831843);
pub const PINK_950: Color = hex(0x500724);

// Rose
pub const ROSE_50: Color = hex(0xfff1f2);
pub const ROSE_100: Color = hex(0xffe4e6);
pub const ROSE_200: Color = hex(0xfecdd3);
pub const ROSE_300: Color = hex(0xfda4af);
pub const ROSE_400: Color = hex(0xfb7185);
pub const ROSE_500: Color = hex(0xf43f5e);
pub const ROSE_600: Color = hex(0xe11d48);
pub const ROSE_700: Color = hex(0xbe123c);
pub const ROSE_800: Color = hex(0x9f1239);
pub const ROSE_900: Color = hex(0x881337);
pub const ROSE_950: Color = hex(0x4c0519);
