use bevy::prelude::*;
use models::game_states::GameState;

use crate::fonts::UiFont;
use crate::settings_screen::SettingsOrigin;
use crate::theme;
use crate::widgets::ButtonBuilder;

const LOGO_WIDTH_PX: u16 = 512;
const LOGO_HEIGHT_PX: u16 = 256;
const LOGO_MARGIN_BOTTOM_PX: u16 = 24;
/// Main menu uses larger buttons than the design-token defaults.
const MENU_BUTTON_FONT_SIZE_PX: f32 = 26.0;
const MENU_BUTTON_PADDING_H_PX: f32 = 40.0;
const MENU_BUTTON_PADDING_V_PX: f32 = 14.0;
const COG_BUTTON_SIZE_PX: f32 = 44.0;
const COG_ICON_SIZE_PX: f32 = 28.0;
const COG_MARGIN_PX: f32 = 12.0;
const COG_BORDER_RADIUS_PX: f32 = 8.0;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub(crate) struct StartButton;

#[derive(Component)]
pub(crate) struct LoreButton;

#[derive(Component)]
pub(crate) struct CreditsButton;

#[derive(Component)]
pub(crate) struct MainMenuSettingsButton;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, fonts: Res<UiFont>) {
    let root = commands
        .spawn((
            MainMenu,
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            BackgroundColor(theme::DARK_BG),
        ))
        .id();

    // Cog / settings button anchored to top-right corner
    commands
        .spawn((
            MainMenuSettingsButton,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(COG_MARGIN_PX),
                right: Val::Px(COG_MARGIN_PX),
                width: Val::Px(COG_BUTTON_SIZE_PX),
                height: Val::Px(COG_BUTTON_SIZE_PX),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(COG_BORDER_RADIUS_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::BUTTON_BG),
            ChildOf(root),
        ))
        .with_child((
            ImageNode::new(asset_server.load("sprites/ui/cog.webp")),
            Node {
                width: Val::Px(COG_ICON_SIZE_PX),
                height: Val::Px(COG_ICON_SIZE_PX),
                ..Node::default()
            },
        ));

    commands.spawn((
        ImageNode::new(asset_server.load("sprites/ui/logo.webp")),
        Node {
            width: Val::Px(f32::from(LOGO_WIDTH_PX)),
            height: Val::Px(f32::from(LOGO_HEIGHT_PX)),
            margin: UiRect::bottom(Val::Px(f32::from(LOGO_MARGIN_BOTTOM_PX))),
            ..Node::default()
        },
        ChildOf(root),
    ));

    ButtonBuilder::new("Begin Journey", StartButton, fonts.0.clone())
        .padding(MENU_BUTTON_PADDING_H_PX, MENU_BUTTON_PADDING_V_PX)
        .font_size(MENU_BUTTON_FONT_SIZE_PX)
        .spawn(&mut commands, root);
    ButtonBuilder::new("Lore", LoreButton, fonts.0.clone())
        .padding(MENU_BUTTON_PADDING_H_PX, MENU_BUTTON_PADDING_V_PX)
        .font_size(MENU_BUTTON_FONT_SIZE_PX)
        .spawn(&mut commands, root);
    ButtonBuilder::new("Credits", CreditsButton, fonts.0.clone())
        .padding(MENU_BUTTON_PADDING_H_PX, MENU_BUTTON_PADDING_V_PX)
        .font_size(MENU_BUTTON_FONT_SIZE_PX)
        .spawn(&mut commands, root);
}

pub fn button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut origin: ResMut<SettingsOrigin>,
    start_q: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    lore_q: Query<&Interaction, (Changed<Interaction>, With<LoreButton>)>,
    credits_q: Query<&Interaction, (Changed<Interaction>, With<CreditsButton>)>,
    settings_q: Query<&Interaction, (Changed<Interaction>, With<MainMenuSettingsButton>)>,
) {
    start_q
        .iter()
        .filter(|i| **i == Interaction::Pressed)
        .for_each(|_| next_state.set(GameState::Playing));

    lore_q
        .iter()
        .filter(|i| **i == Interaction::Pressed)
        .for_each(|_| next_state.set(GameState::LorePage));

    credits_q
        .iter()
        .filter(|i| **i == Interaction::Pressed)
        .for_each(|_| next_state.set(GameState::Credits));

    settings_q
        .iter()
        .filter(|i| **i == Interaction::Pressed)
        .for_each(|_| {
            *origin = SettingsOrigin::MainMenu;
            next_state.set(GameState::Settings);
        });
}
