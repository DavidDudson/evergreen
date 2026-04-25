use bevy::post_process::bloom::Bloom;

use crate::bloom_config::BloomConfig;

/// Returns a `Bloom` component tuned for the project's pixel-art look.
///
/// Constructed from `BloomConfig::default()` so the camera setup site stays
/// stable while values can be tuned via the resource at runtime.
pub fn pixel_art_bloom() -> Bloom {
    BloomConfig::default().to_bloom()
}
