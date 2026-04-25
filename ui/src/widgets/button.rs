use bevy::prelude::*;

use crate::design_tokens::{spacing, typography};
use crate::theme;

/// Reusable menu-button builder. Eliminates the ~25-line spawn boilerplate
/// duplicated across `main_menu`, `pause_menu`, etc.
///
/// Usage:
/// ```ignore
/// ButtonBuilder::new("Begin Journey", StartButton, font.clone())
///     .spawn(&mut commands, root);
/// ```
pub struct ButtonBuilder<M: Component> {
    marker: M,
    label: String,
    font: Handle<Font>,
    padding_h_px: f32,
    padding_v_px: f32,
    margin_top_px: f32,
    margin_bottom_px: f32,
    margin_left_px: f32,
    margin_right_px: f32,
    border_px: f32,
    radius_px: f32,
    font_size_px: f32,
    border_color: Color,
    background_color: Color,
    text_color: Color,
}

impl<M: Component> ButtonBuilder<M> {
    pub fn new(label: impl Into<String>, marker: M, font: Handle<Font>) -> Self {
        Self {
            marker,
            label: label.into(),
            font,
            padding_h_px: spacing::BUTTON_PADDING_H_PX,
            padding_v_px: spacing::BUTTON_PADDING_V_PX,
            margin_top_px: spacing::BUTTON_MARGIN_TOP_PX,
            margin_bottom_px: spacing::BUTTON_MARGIN_BOTTOM_PX,
            margin_left_px: 0.0,
            margin_right_px: 0.0,
            border_px: spacing::BUTTON_BORDER_PX,
            radius_px: spacing::BUTTON_RADIUS_PX,
            font_size_px: typography::BUTTON_FONT_SIZE_PX,
            border_color: theme::ACCENT,
            background_color: theme::BUTTON_BG,
            text_color: theme::BUTTON_TEXT,
        }
    }

    pub fn padding(mut self, h_px: f32, v_px: f32) -> Self {
        self.padding_h_px = h_px;
        self.padding_v_px = v_px;
        self
    }

    pub fn font_size(mut self, font_size_px: f32) -> Self {
        self.font_size_px = font_size_px;
        self
    }

    pub fn margin(mut self, top_px: f32, bottom_px: f32) -> Self {
        self.margin_top_px = top_px;
        self.margin_bottom_px = bottom_px;
        self
    }

    pub fn margin_x(mut self, left_px: f32, right_px: f32) -> Self {
        self.margin_left_px = left_px;
        self.margin_right_px = right_px;
        self
    }

    pub fn colors(mut self, background: Color, text: Color, border: Color) -> Self {
        self.background_color = background;
        self.text_color = text;
        self.border_color = border;
        self
    }

    pub fn spawn(self, commands: &mut Commands, parent: Entity) -> Entity {
        let id = commands
            .spawn((
                self.marker,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(self.padding_h_px), Val::Px(self.padding_v_px)),
                    margin: UiRect {
                        top: Val::Px(self.margin_top_px),
                        bottom: Val::Px(self.margin_bottom_px),
                        left: Val::Px(self.margin_left_px),
                        right: Val::Px(self.margin_right_px),
                    },
                    border: UiRect::all(Val::Px(self.border_px)),
                    border_radius: BorderRadius::all(Val::Px(self.radius_px)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Node::default()
                },
                BorderColor::all(self.border_color),
                BackgroundColor(self.background_color),
                ChildOf(parent),
            ))
            .with_child((
                Text::new(self.label),
                TextColor(self.text_color),
                TextFont {
                    font: self.font,
                    font_size: self.font_size_px,
                    ..default()
                },
            ))
            .id();
        id
    }
}
