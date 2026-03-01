/// Rendering layers ordered back-to-front by z-index.
///
/// Each variant maps to a fixed z-index used in `Transform`.
/// Add new layers here to keep z-ordering centralized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Tilemap,
    SceneryTree,
    Player,
    SceneryBush,
    SceneryFlower,
}

impl Layer {
    pub const fn z(self) -> u16 {
        match self {
            Self::Tilemap => 0,
            Self::SceneryTree => 3,
            Self::Player => 10,
            Self::SceneryBush => 11,
            Self::SceneryFlower => 12,
        }
    }

    /// Convenience for use with `Transform::from_xyz`.
    /// `f32::from(u16)` is not available in const context.
    #[allow(clippy::as_conversions)]
    pub const fn z_f32(self) -> f32 {
        self.z() as f32
    }
}
