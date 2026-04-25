use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal};
use level::area::AreaAlignment;
use level::world::WorldMap;

use crate::lerp_config::PostProcessLerpConfig;
use crate::presets::{BiomePresets, GradingPreset};
use crate::toggles::DisableColorGrading;

/// Fallback alignment when the current area is missing (greenwood -- safe neutral).
const DEFAULT_AREA_ALIGNMENT: AreaAlignment = 50;

/// Apply a `GradingPreset` to a `ColorGrading` component (writes the
/// `global` section, leaves shadows/midtones/highlights untouched).
pub fn apply_target(grading: &mut ColorGrading, target: GradingPreset) {
    grading.global = ColorGradingGlobal {
        exposure: target.exposure,
        temperature: target.temperature,
        tint: target.tint,
        post_saturation: target.post_saturation,
        ..grading.global.clone()
    };
}

/// Lerp current grading toward target by `alpha` (0..1, single-frame step).
pub fn step_toward(current: GradingPreset, target: GradingPreset, alpha: f32) -> GradingPreset {
    current.lerp(target, alpha.clamp(0.0, 1.0))
}

/// Per-frame system: read current area's alignment from `WorldMap`,
/// compute the target `GradingPreset`, lerp the camera's grading toward it.
pub fn sync_color_grading(
    world: Res<WorldMap>,
    time: Res<Time>,
    presets: Res<BiomePresets>,
    lerp_config: Res<PostProcessLerpConfig>,
    mut query: Query<&mut ColorGrading, (With<Camera2d>, Without<DisableColorGrading>)>,
) {
    let alignment = world
        .get_area(world.current)
        .map_or(DEFAULT_AREA_ALIGNMENT, |a| a.alignment);
    let target = presets.target_for_alignment(alignment);
    let alpha = (lerp_config.grading_speed_per_sec * time.delta_secs()).min(1.0);

    for mut grading in &mut query {
        let current = GradingPreset {
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

    fn approx_target(actual: GradingPreset, expected: GradingPreset) {
        approx(actual.exposure, expected.exposure);
        approx(actual.temperature, expected.temperature);
        approx(actual.tint, expected.tint);
        approx(actual.post_saturation, expected.post_saturation);
    }

    #[test]
    fn step_toward_zero_alpha_returns_current() {
        let current = GradingPreset::CITY;
        let target = GradingPreset::DARKWOOD;
        approx_target(step_toward(current, target, 0.0), current);
    }

    #[test]
    fn step_toward_one_alpha_returns_target() {
        let current = GradingPreset::CITY;
        let target = GradingPreset::DARKWOOD;
        approx_target(step_toward(current, target, 1.0), target);
    }
}
