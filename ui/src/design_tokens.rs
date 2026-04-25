//! UI design tokens, sourced from Tailwind v4 (`models::tailwind`).
//!
//! Each constant aliases the matching Tailwind step so we keep one canonical
//! scale for spacing / radius / typography. Screens that need bespoke values
//! should still go through Tailwind steps (e.g. `tw::SPACE_5_PX`) rather
//! than introducing fresh literals.

use models::tailwind as tw;

pub mod spacing {
    use super::tw;
    pub const BUTTON_PADDING_H_PX: f32 = tw::SPACE_8_PX;       // 32 px
    pub const BUTTON_PADDING_V_PX: f32 = tw::SPACE_3_PX;       // 12 px
    pub const BUTTON_MARGIN_TOP_PX: f32 = tw::SPACE_2_PX;      // 8 px
    pub const BUTTON_MARGIN_BOTTOM_PX: f32 = tw::SPACE_3_PX;   // 12 px
    pub const BUTTON_BORDER_PX: f32 = tw::SPACE_0_5_PX;        // 2 px
    pub const BUTTON_RADIUS_PX: f32 = tw::RADIUS_BASE_PX;      // 6 px

    pub const PANEL_PADDING_PX: f32 = tw::SPACE_4_PX;          // 16 px
    pub const PANEL_GAP_PX: f32 = tw::SPACE_2_PX;              // 8 px
    pub const PANEL_RADIUS_PX: f32 = tw::RADIUS_MD_PX;         // 8 px

    pub const SECTION_GAP_PX: f32 = tw::SPACE_6_PX;            // 24 px
    pub const ITEM_GAP_PX: f32 = tw::SPACE_3_PX;               // 12 px
    pub const COMPACT_GAP_PX: f32 = tw::SPACE_1_PX;            // 4 px
}

pub mod typography {
    use super::tw;
    pub const TITLE_FONT_SIZE_PX: f32 = tw::TEXT_4XL_PX;       // 36 px
    pub const HEADING_FONT_SIZE_PX: f32 = tw::TEXT_2XL_PX;     // 24 px
    pub const BUTTON_FONT_SIZE_PX: f32 = tw::TEXT_XL_PX;       // 20 px (was 22)
    pub const BODY_FONT_SIZE_PX: f32 = tw::TEXT_BASE_PX;       // 16 px
    pub const CAPTION_FONT_SIZE_PX: f32 = tw::TEXT_XS_PX;      // 12 px
}
