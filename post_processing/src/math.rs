/// Linear interpolation between `a` and `b` by parameter `t`.
pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
