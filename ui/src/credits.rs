use bevy::prelude::*;
use bevy::ui::ComputedNode;
use models::game_states::GameState;
use models::palette;

use crate::fonts::UiFont;
use crate::theme;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const PAGE_PADDING_PX: f32 = 60.0;
const MAX_WIDTH_PX: f32 = 720.0;

const TITLE_FONT_SIZE_PX: f32 = 42.0;
const TITLE_MARGIN_BOTTOM_PX: f32 = 48.0;

const SECTION_TITLE_FONT_SIZE_PX: f32 = 20.0;
const SECTION_TITLE_MARGIN_TOP_PX: f32 = 32.0;
const SECTION_TITLE_MARGIN_BOTTOM_PX: f32 = 8.0;

const BODY_FONT_SIZE_PX: f32 = 15.0;
const BODY_LINE_MARGIN_BOTTOM_PX: f32 = 4.0;

const DIVIDER_MARGIN_PX: f32 = 24.0;
const DIVIDER_HEIGHT_PX: f32 = 1.0;

const CLOSING_FONT_SIZE_PX: f32 = 14.0;
const CLOSING_MARGIN_TOP_PX: f32 = 40.0;

const BACK_FONT_SIZE_PX: f32 = 18.0;
const BACK_PADDING_H_PX: f32 = 32.0;
const BACK_PADDING_V_PX: f32 = 12.0;
const BACK_MARGIN_TOP_PX: f32 = 40.0;
const BACK_MARGIN_BOTTOM_PX: f32 = 60.0;
const BACK_BORDER_PX: f32 = 2.0;
const BACK_RADIUS_PX: f32 = 6.0;

const SCROLLBAR_WIDTH_PX: f32 = 8.0;
const SCROLLBAR_THUMB_MIN_HEIGHT_PX: f32 = 24.0;
const SCROLLBAR_RADIUS_PX: f32 = 4.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct CreditsScreen;

#[derive(Component)]
pub(crate) struct CreditsBackButton;

#[derive(Component)]
pub(crate) struct CreditsScrollArea;

#[derive(Component)]
pub(crate) struct CreditsContent;

#[derive(Component)]
pub(crate) struct CreditsScrollThumb;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub fn setup(mut commands: Commands, fonts: Res<UiFont>) {
    let font = fonts.0.clone();

    // Outer wrapper: flex-row so the scrollbar sits beside the content.
    let root = commands
        .spawn((
            CreditsScreen,
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            BackgroundColor(theme::DARK_BG),
        ))
        .id();

    // Scrollable area: flex-col, grows to fill row, overflows vertically.
    let scroll_area = commands
        .spawn((
            CreditsScrollArea,
            ScrollPosition::default(),
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                overflow: Overflow::scroll_y(),
                padding: UiRect::axes(Val::Px(PAGE_PADDING_PX), Val::Px(PAGE_PADDING_PX)),
                ..Node::default()
            },
            ChildOf(root),
        ))
        .id();

    // Scrollbar track: fixed-width column on the right edge.
    let track = commands
        .spawn((
            Node {
                width: Val::Px(SCROLLBAR_WIDTH_PX),
                height: Val::Percent(100.0),
                flex_shrink: 0.0,
                ..Node::default()
            },
            BackgroundColor(palette::SCROLLBAR_TRACK),
            ChildOf(root),
        ))
        .id();

    // Thumb: absolutely positioned within track, sized + positioned by sync system.
    commands.spawn((
        CreditsScrollThumb,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Px(0.0),
            top: Val::Px(0.0),
            border_radius: BorderRadius::all(Val::Px(SCROLLBAR_RADIUS_PX)),
            ..Node::default()
        },
        BackgroundColor(palette::SCROLLBAR_THUMB),
        ChildOf(track),
    ));

    // Content column inside scroll area.
    let column = commands
        .spawn((
            CreditsContent,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                max_width: Val::Px(MAX_WIDTH_PX),
                width: Val::Percent(100.0),
                ..Node::default()
            },
            ChildOf(scroll_area),
        ))
        .id();

    // Title
    spawn_text(
        &mut commands,
        column,
        "Credits",
        TITLE_FONT_SIZE_PX,
        theme::TITLE,
        UiRect::bottom(Val::Px(TITLE_MARGIN_BOTTOM_PX)),
        font.clone(),
    );

    // ── World Builder ──────────────────────────────────────────────────────
    spawn_section_title(
        &mut commands,
        column,
        "Dungeon Master & World Builder",
        font.clone(),
    );
    spawn_body(&mut commands, column, "Galen Graham", font.clone());
    spawn_body_italic(
        &mut commands,
        column,
        "Who has built worlds larger than ever imagined, and invested countless\nhours crafting the lore, characters, and stories that made Evergreen real.",
        font.clone(),
    );

    spawn_divider(&mut commands, column);

    // ── Players ───────────────────────────────────────────────────────────
    spawn_section_title(&mut commands, column, "Players", font.clone());
    for (name, character) in [
        ("Emma Donaldson", "Drizella Tremaine"),
        ("Jesse", "Bigby"),
        ("Brianna Merriman", "Mordred"),
        ("David Dudson", "Darian Sand & Briar Rose"),
    ] {
        spawn_player_line(&mut commands, column, name, character, font.clone());
    }
    spawn_body_italic(
        &mut commands,
        column,
        "Who enriched this world with their own lore and lifted every session\nwith their creativity, heart, and play.",
        font.clone(),
    );

    spawn_divider(&mut commands, column);

    // ── Pathfinder & D&D ──────────────────────────────────────────────────
    spawn_section_title(
        &mut commands,
        column,
        "Paizo & Wizards of the Coast",
        font.clone(),
    );
    spawn_body_italic(
        &mut commands,
        column,
        "To the creators of Pathfinder and Dungeons & Dragons — for providing\npeople across the world an escape from reality, one story at a time.",
        font.clone(),
    );

    spawn_divider(&mut commands, column);

    // ── Fairy Tale Authors ────────────────────────────────────────────────
    spawn_section_title(
        &mut commands,
        column,
        "The Authors of Fairy Tales",
        font.clone(),
    );
    spawn_body_italic(
        &mut commands,
        column,
        "To all the authors who created fairy tales throughout time —\nwho made the world believe that, quite possibly,\ndreams and wishes really can come true.",
        font.clone(),
    );

    // ── Closing ───────────────────────────────────────────────────────────
    commands.spawn((
        Text::new("✦  ✦  ✦"),
        TextColor(theme::ACCENT),
        TextFont {
            font: font.clone(),
            font_size: CLOSING_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::top(Val::Px(CLOSING_MARGIN_TOP_PX)),
            ..Node::default()
        },
        ChildOf(column),
    ));

    // ── Back button ───────────────────────────────────────────────────────
    commands
        .spawn((
            CreditsBackButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(BACK_PADDING_H_PX), Val::Px(BACK_PADDING_V_PX)),
                margin: UiRect::new(
                    Val::Auto,
                    Val::Auto,
                    Val::Px(BACK_MARGIN_TOP_PX),
                    Val::Px(BACK_MARGIN_BOTTOM_PX),
                ),
                border: UiRect::all(Val::Px(BACK_BORDER_PX)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(BACK_RADIUS_PX)),
                ..Node::default()
            },
            BorderColor::all(theme::ACCENT),
            BackgroundColor(theme::BUTTON_BG),
            ChildOf(column),
        ))
        .with_child((
            Text::new("Back"),
            TextColor(theme::BUTTON_TEXT),
            TextFont {
                font,
                font_size: BACK_FONT_SIZE_PX,
                ..default()
            },
        ));
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub fn handle_back(
    mut next_state: ResMut<NextState<GameState>>,
    q: Query<&Interaction, (Changed<Interaction>, With<CreditsBackButton>)>,
) {
    q.iter()
        .filter(|i| **i == Interaction::Pressed)
        .for_each(|_| next_state.set(GameState::MainMenu));
}

pub fn sync_scrollbar(
    scroll_q: Query<(&ComputedNode, &ScrollPosition), With<CreditsScrollArea>>,
    content_q: Query<&ComputedNode, With<CreditsContent>>,
    mut thumb_q: Query<&mut Node, With<CreditsScrollThumb>>,
) {
    let Ok((scroll_computed, scroll_pos)) = scroll_q.single() else {
        return;
    };
    let Ok(content_computed) = content_q.single() else {
        return;
    };
    let Ok(mut thumb) = thumb_q.single_mut() else {
        return;
    };

    let viewport_h = scroll_computed.size().y;
    let content_h = content_computed.size().y;

    if content_h <= viewport_h {
        thumb.height = Val::Px(0.0);
        return;
    }

    let scroll_max = content_h - viewport_h;
    let ratio = (viewport_h / content_h).clamp(0.0, 1.0);
    let thumb_h = (ratio * viewport_h).max(SCROLLBAR_THUMB_MIN_HEIGHT_PX);
    let scroll_frac = (scroll_pos.0.y / scroll_max).clamp(0.0, 1.0);

    thumb.height = Val::Px(thumb_h);
    thumb.top = Val::Px(scroll_frac * (viewport_h - thumb_h));
}

// ---------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------

fn spawn_section_title(commands: &mut Commands, parent: Entity, text: &str, font: Handle<Font>) {
    commands.spawn((
        Text::new(text),
        TextColor(theme::TITLE),
        TextFont {
            font,
            font_size: SECTION_TITLE_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect {
                top: Val::Px(SECTION_TITLE_MARGIN_TOP_PX),
                bottom: Val::Px(SECTION_TITLE_MARGIN_BOTTOM_PX),
                ..UiRect::default()
            },
            ..Node::default()
        },
        ChildOf(parent),
    ));
}

fn spawn_body(commands: &mut Commands, parent: Entity, text: &str, font: Handle<Font>) {
    commands.spawn((
        Text::new(text),
        TextColor(theme::BUTTON_TEXT),
        TextFont {
            font,
            font_size: BODY_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(BODY_LINE_MARGIN_BOTTOM_PX)),
            ..Node::default()
        },
        ChildOf(parent),
    ));
}

fn spawn_body_italic(commands: &mut Commands, parent: Entity, text: &str, font: Handle<Font>) {
    commands.spawn((
        Text::new(text),
        TextColor(theme::DIALOG_TEXT),
        TextFont {
            font,
            font_size: BODY_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(BODY_LINE_MARGIN_BOTTOM_PX)),
            ..Node::default()
        },
        ChildOf(parent),
    ));
}

fn spawn_player_line(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    character: &str,
    font: Handle<Font>,
) {
    commands.spawn((
        Text::new(format!("{name}  —  {character}")),
        TextColor(theme::BUTTON_TEXT),
        TextFont {
            font,
            font_size: BODY_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(BODY_LINE_MARGIN_BOTTOM_PX)),
            ..Node::default()
        },
        ChildOf(parent),
    ));
}

fn spawn_text(
    commands: &mut Commands,
    parent: Entity,
    text: &str,
    font_size: f32,
    color: Color,
    margin: UiRect,
    font: Handle<Font>,
) {
    commands.spawn((
        Text::new(text),
        TextColor(color),
        TextFont {
            font,
            font_size,
            ..default()
        },
        Node {
            margin,
            ..Node::default()
        },
        ChildOf(parent),
    ));
}

fn spawn_divider(commands: &mut Commands, parent: Entity) {
    commands.spawn((
        Node {
            width: Val::Percent(80.0),
            height: Val::Px(DIVIDER_HEIGHT_PX),
            margin: UiRect::axes(Val::Auto, Val::Px(DIVIDER_MARGIN_PX)),
            ..Node::default()
        },
        BackgroundColor(theme::ACCENT),
        ChildOf(parent),
    ));
}
