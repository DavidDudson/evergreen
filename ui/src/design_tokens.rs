//! Shared spacing, typography, and layout constants for menu screens.
//!
//! Each screen historically defined its own `BUTTON_PADDING_*`, `TITLE_FONT_*`,
//! etc. with slight variation. These tokens are the canonical defaults; pass
//! overrides into `widgets::ButtonBuilder` (or local consts) when a screen
//! genuinely needs different values.

pub mod spacing {
    pub const BUTTON_PADDING_H_PX: f32 = 32.0;
    pub const BUTTON_PADDING_V_PX: f32 = 12.0;
    pub const BUTTON_MARGIN_TOP_PX: f32 = 8.0;
    pub const BUTTON_MARGIN_BOTTOM_PX: f32 = 12.0;
    pub const BUTTON_BORDER_PX: f32 = 2.0;
    pub const BUTTON_RADIUS_PX: f32 = 6.0;
}

pub mod typography {
    pub const TITLE_FONT_SIZE_PX: f32 = 36.0;
    pub const HEADING_FONT_SIZE_PX: f32 = 24.0;
    pub const BUTTON_FONT_SIZE_PX: f32 = 22.0;
    pub const BODY_FONT_SIZE_PX: f32 = 16.0;
    pub const CAPTION_FONT_SIZE_PX: f32 = 12.0;
}
