use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal};
use level::area::AreaAlignment;
use level::world::WorldMap;

use crate::math::lerp;

/// Fallback alignment when the current area is missing (greenwood -- safe neutral).
const DEFAULT_AREA_ALIGNMENT: AreaAlignment = 50;

/// Anchor alignment for the city biome.
const ALIGNMENT_CITY: f32 = 1.0;
/// Anchor alignment for the greenwood biome.
const ALIGNMENT_GREENWOOD: f32 = 50.0;
/// Anchor alignment for the darkwood biome.
const ALIGNMENT_DARKWOOD: f32 = 100.0;

/// City: bright, near-neutral daylight on stone; only a whisper of warmth.
const CITY_EXPOSURE: f32 = 0.05;
const CITY_TEMPERATURE: f32 = 0.04;
const CITY_TINT: f32 = 0.0;
const CITY_POST_SATURATION: f32 = 0.92;

/// Greenwood: vivid, neutral white balance.
const GREENWOOD_EXPOSURE: f32 = 0.0;
const GREENWOOD_TEMPERATURE: f32 = -0.02;
const GREENWOOD_TINT: f32 = 0.05;
const GREENWOOD_POST_SATURATION: f32 = 1.10;

/// Darkwood: cool, slightly underexposed, muted.
const DARKWOOD_EXPOSURE: f32 = -0.20;
const DARKWOOD_TEMPERATURE: f32 = -0.30;
const DARKWOOD_TINT: f32 = 0.10;
const DARKWOOD_POST_SATURATION: f32 = 0.85;

/// One biome's target grading values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeGradingTarget {
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub post_saturation: f32,
}

impl BiomeGradingTarget {
    pub const CITY: Self = Self {
        exposure: CITY_EXPOSURE,
        temperature: CITY_TEMPERATURE,
        tint: CITY_TINT,
        post_saturation: CITY_POST_SATURATION,
    };
    pub const GREENWOOD: Self = Self {
        exposure: GREENWOOD_EXPOSURE,
        temperature: GREENWOOD_TEMPERATURE,
        tint: GREENWOOD_TINT,
        post_saturation: GREENWOOD_POST_SATURATION,
    };
    pub const DARKWOOD: Self = Self {
        exposure: DARKWOOD_EXPOSURE,
        temperature: DARKWOOD_TEMPERATURE,
        tint: DARKWOOD_TINT,
        post_saturation: DARKWOOD_POST_SATURATION,
    };

    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            exposure: lerp(self.exposure, other.exposure, t),
            temperature: lerp(self.temperature, other.temperature, t),
            tint: lerp(self.tint, other.tint, t),
            post_saturation: lerp(self.post_saturation, other.post_saturation, t),
        }
    }
}

/// Map an alignment value to a target grading by interpolating between anchors.
pub fn target_for_alignment(alignment: AreaAlignment) -> BiomeGradingTarget {
    let a = f32::from(alignment.clamp(1, 100));
    if a <= ALIGNMENT_GREENWOOD {
        let t = (a - ALIGNMENT_CITY) / (ALIGNMENT_GREENWOOD - ALIGNMENT_CITY);
        BiomeGradingTarget::CITY.lerp(BiomeGradingTarget::GREENWOOD, t)
    } else {
        let t = (a - ALIGNMENT_GREENWOOD) / (ALIGNMENT_DARKWOOD - ALIGNMENT_GREENWOOD);
        BiomeGradingTarget::GREENWOOD.lerp(BiomeGradingTarget::DARKWOOD, t)
    }
}

/// Apply a `BiomeGradingTarget` to a `ColorGrading` component (writes the
/// `global` section, leaves shadows/midtones/highlights untouched).
pub fn apply_target(grading: &mut ColorGrading, target: BiomeGradingTarget) {
    grading.global = ColorGradingGlobal {
        exposure: target.exposure,
        temperature: target.temperature,
        tint: target.tint,
        post_saturation: target.post_saturation,
        ..grading.global.clone()
    };
}

/// Lerp speed for color grading transitions between areas (per second).
pub const GRADING_LERP_SPEED: f32 = 2.5;

/// Lerp current grading toward target by `alpha` (0..1, single-frame step).
pub fn step_toward(
    current: BiomeGradingTarget,
    target: BiomeGradingTarget,
    alpha: f32,
) -> BiomeGradingTarget {
    current.lerp(target, alpha.clamp(0.0, 1.0))
}

/// Per-frame system: read current area's alignment from `WorldMap`,
/// compute the target `BiomeGradingTarget`, lerp the camera's grading toward it.
pub fn sync_color_grading(
    world: Res<WorldMap>,
    time: Res<Time>,
    mut query: Query<&mut ColorGrading, With<Camera2d>>,
) {
    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_AREA_ALIGNMENT, |a| a.alignment);
    let target = target_for_alignment(alignment);
    let alpha = (GRADING_LERP_SPEED * time.delta_secs()).min(1.0);

    for mut grading in &mut query {
        let current = BiomeGradingTarget {
            exposure: grading.global.exposure,
            temperature: grading.global.temperature,
            tint: grading.global.tint,
            post_saturation: grading.global.post_saturation,
        };
        let next = step_toward(current, target, alpha);
        apply_target(&mut grading, next);
    }
}

/// Reset the camera's `ColorGrading` to defaults so menus and non-gameplay
/// states render with neutral grading instead of inheriting the last biome.
pub fn reset_color_grading(mut query: Query<&mut ColorGrading, With<Camera2d>>) {
    for mut grading in &mut query {
        *grading = ColorGrading::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected ~{b}, got {a}");
    }

    #[test]
    fn target_at_city_anchor_returns_city() {
        let t = target_for_alignment(1);
        approx(t.exposure, CITY_EXPOSURE);
        approx(t.temperature, CITY_TEMPERATURE);
        approx(t.tint, CITY_TINT);
        approx(t.post_saturation, CITY_POST_SATURATION);
    }

    #[test]
    fn target_at_greenwood_anchor_returns_greenwood() {
        let t = target_for_alignment(50);
        approx(t.exposure, GREENWOOD_EXPOSURE);
        approx(t.temperature, GREENWOOD_TEMPERATURE);
        approx(t.tint, GREENWOOD_TINT);
        approx(t.post_saturation, GREENWOOD_POST_SATURATION);
    }

    #[test]
    fn target_at_darkwood_anchor_returns_darkwood() {
        let t = target_for_alignment(100);
        approx(t.exposure, DARKWOOD_EXPOSURE);
        approx(t.temperature, DARKWOOD_TEMPERATURE);
        approx(t.tint, DARKWOOD_TINT);
        approx(t.post_saturation, DARKWOOD_POST_SATURATION);
    }

    #[test]
    fn target_midway_city_greenwood_is_average() {
        let t = target_for_alignment(25);
        let expected_exposure = lerp(CITY_EXPOSURE, GREENWOOD_EXPOSURE, (25.0 - 1.0) / 49.0);
        approx(t.exposure, expected_exposure);
    }

    #[test]
    fn target_clamps_below_one() {
        let t = target_for_alignment(0);
        approx(t.exposure, CITY_EXPOSURE);
    }

    fn approx_target(actual: BiomeGradingTarget, expected: BiomeGradingTarget) {
        approx(actual.exposure, expected.exposure);
        approx(actual.temperature, expected.temperature);
        approx(actual.tint, expected.tint);
        approx(actual.post_saturation, expected.post_saturation);
    }

    #[test]
    fn step_toward_zero_alpha_returns_current() {
        let current = BiomeGradingTarget::CITY;
        let target = BiomeGradingTarget::DARKWOOD;
        approx_target(step_toward(current, target, 0.0), current);
    }

    #[test]
    fn step_toward_one_alpha_returns_target() {
        let current = BiomeGradingTarget::CITY;
        let target = BiomeGradingTarget::DARKWOOD;
        approx_target(step_toward(current, target, 1.0), target);
    }
}
