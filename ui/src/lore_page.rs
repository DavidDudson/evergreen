use bevy::prelude::*;
use dialog::asset::LoreCategory;
use dialog::history::LoreBook;
use dialog::locale::LocaleMap;
use models::game_states::GameState;

use crate::fonts::UiFont;
use crate::theme;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const PAGE_PADDING_PX: f32 = 40.0;
const TITLE_FONT_SIZE_PX: f32 = 36.0;
const TITLE_MARGIN_BOTTOM_PX: f32 = 24.0;

const SIDEBAR_WIDTH_PX: f32 = 180.0;
const SIDEBAR_GAP_PX: f32 = 16.0;

const CATEGORY_FONT_SIZE_PX: f32 = 16.0;
const CATEGORY_PADDING_H_PX: f32 = 14.0;
const CATEGORY_PADDING_V_PX: f32 = 8.0;
const CATEGORY_MARGIN_PX: f32 = 3.0;
const CATEGORY_RADIUS_PX: f32 = 4.0;

const TOPIC_FONT_SIZE_PX: f32 = 14.0;
const TOPIC_PADDING_H_PX: f32 = 12.0;
const TOPIC_PADDING_V_PX: f32 = 6.0;
const TOPIC_MARGIN_PX: f32 = 2.0;
const TOPIC_RADIUS_PX: f32 = 4.0;

const ENTRY_SPEAKER_FONT_SIZE_PX: f32 = 16.0;
const ENTRY_TEXT_FONT_SIZE_PX: f32 = 14.0;
const ENTRY_MARGIN_BOTTOM_PX: f32 = 16.0;
const ENTRY_PADDING_PX: f32 = 12.0;
const ENTRY_RADIUS_PX: f32 = 6.0;
const ENTRY_BORDER_PX: f32 = 1.0;

const PORTRAIT_SIZE_PX: f32 = 64.0;
const PORTRAIT_MARGIN_BOTTOM_PX: f32 = 12.0;

const EMPTY_FONT_SIZE_PX: f32 = 16.0;

const BACK_FONT_SIZE_PX: f32 = 18.0;
const BACK_PADDING_H_PX: f32 = 24.0;
const BACK_PADDING_V_PX: f32 = 10.0;
const BACK_MARGIN_TOP_PX: f32 = 16.0;

// ---------------------------------------------------------------------------
// Components / state
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct LorePage;

#[derive(Component)]
pub(crate) struct LoreCategoryButton(LoreCategory);

#[derive(Component)]
pub(crate) struct LoreTopicButton(String);

#[derive(Component)]
pub(crate) struct LoreBackButton;

/// Marker for the topic list panel (second sidebar column).
#[derive(Component)]
pub(crate) struct LoreTopicList;

/// Marker for the content panel (right side).
#[derive(Component)]
pub(crate) struct LoreContentPanel;

/// Marker for dynamically spawned content elements.
#[derive(Component)]
pub(crate) struct LoreContentItem;

/// Current selection state.
#[derive(Resource, Default)]
pub(crate) struct LoreSelection {
    category: Option<LoreCategory>,
    topic: Option<String>,
}

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

pub fn setup(
    mut commands: Commands,
    lore_book: Res<LoreBook>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
) {
    commands.insert_resource(LoreSelection::default());

    // Root container
    let root = commands
        .spawn((
            LorePage,
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(PAGE_PADDING_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::DARK_BG),
        ))
        .id();

    // Title
    commands.spawn((
        Text::new(locale.get("ui.lore.title").to_string()),
        TextColor(theme::TITLE),
        TextFont {
            font: fonts.0.clone(),
            font_size: TITLE_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(TITLE_MARGIN_BOTTOM_PX)),
            ..Node::default()
        },
        ChildOf(root),
    ));

    // Main content area: sidebar | topic list | content
    let body = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                column_gap: Val::Px(SIDEBAR_GAP_PX),
                overflow: Overflow::clip(),
                ..Node::default()
            },
            ChildOf(root),
        ))
        .id();

    // Category sidebar
    let sidebar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(SIDEBAR_WIDTH_PX),
                overflow: Overflow::scroll_y(),
                ..Node::default()
            },
            ChildOf(body),
        ))
        .id();

    let categories = lore_book.categories();
    for cat in &categories {
        let label = locale.get(cat.locale_key()).to_string();
        commands
            .spawn((
                LoreCategoryButton(*cat),
                Button,
                Node {
                    padding: UiRect::axes(
                        Val::Px(CATEGORY_PADDING_H_PX),
                        Val::Px(CATEGORY_PADDING_V_PX),
                    ),
                    margin: UiRect::bottom(Val::Px(CATEGORY_MARGIN_PX)),
                    border_radius: BorderRadius::all(Val::Px(CATEGORY_RADIUS_PX)),
                    ..Node::default()
                },
                BackgroundColor(theme::BUTTON_BG),
                ChildOf(sidebar),
            ))
            .with_child((
                Text::new(label),
                TextColor(theme::BUTTON_TEXT),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: CATEGORY_FONT_SIZE_PX,
                    ..default()
                },
            ));
    }

    // Topic list (populated when a category is selected)
    commands.spawn((
        LoreTopicList,
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(SIDEBAR_WIDTH_PX),
            overflow: Overflow::scroll_y(),
            ..Node::default()
        },
        ChildOf(body),
    ));

    // Content panel (populated when a topic is selected)
    let content = commands
        .spawn((
            LoreContentPanel,
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                overflow: Overflow::scroll_y(),
                ..Node::default()
            },
            ChildOf(body),
        ))
        .id();

    // Initial empty state
    if categories.is_empty() {
        commands.spawn((
            LoreContentItem,
            Text::new(locale.get("ui.lore.empty").to_string()),
            TextColor(theme::BUTTON_TEXT),
            TextFont {
                font: fonts.0.clone(),
                font_size: EMPTY_FONT_SIZE_PX,
                ..default()
            },
            ChildOf(content),
        ));
    } else {
        commands.spawn((
            LoreContentItem,
            Text::new(locale.get("ui.lore.select_category").to_string()),
            TextColor(theme::BUTTON_TEXT),
            TextFont {
                font: fonts.0.clone(),
                font_size: EMPTY_FONT_SIZE_PX,
                ..default()
            },
            ChildOf(content),
        ));
    }

    // Back button
    crate::widgets::ButtonBuilder::new(
        locale.get("ui.lore.back").to_string(),
        LoreBackButton,
        fonts.0.clone(),
    )
    .padding(BACK_PADDING_H_PX, BACK_PADDING_V_PX)
    .font_size(BACK_FONT_SIZE_PX)
    .margin(BACK_MARGIN_TOP_PX, 0.0)
    .spawn(&mut commands, root);
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<LorePage>>) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
    commands.remove_resource::<LoreSelection>();
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub fn handle_back_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LoreBackButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    interaction_q
        .iter()
        .filter(|i| **i == Interaction::Pressed)
        .for_each(|_| next_state.set(GameState::MainMenu));
}

#[allow(clippy::too_many_arguments)]
pub fn handle_category_buttons(
    mut interaction_q: Query<
        (&Interaction, &LoreCategoryButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut selection: ResMut<LoreSelection>,
    lore_book: Res<LoreBook>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
    topic_list_q: Query<Entity, With<LoreTopicList>>,
    topic_btn_q: Query<Entity, With<LoreTopicButton>>,
    content_panel_q: Query<Entity, With<LoreContentPanel>>,
    content_item_q: Query<Entity, With<LoreContentItem>>,
    mut commands: Commands,
) {
    let mut new_cat = None;

    for (interaction, button, mut bg) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => {
                new_cat = Some(button.0);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(theme::BUTTON_BG);
            }
        }
    }

    let Some(cat) = new_cat else { return };
    selection.category = Some(cat);
    selection.topic = None;

    // Rebuild topic list
    for entity in &topic_btn_q {
        commands.entity(entity).despawn();
    }
    let Ok(topic_list) = topic_list_q.single() else {
        return;
    };

    let topics = lore_book.topics_in(cat);
    for topic_key in &topics {
        let label = locale.get(topic_key).to_string();
        commands
            .spawn((
                LoreTopicButton(topic_key.clone()),
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(TOPIC_PADDING_H_PX), Val::Px(TOPIC_PADDING_V_PX)),
                    margin: UiRect::bottom(Val::Px(TOPIC_MARGIN_PX)),
                    border_radius: BorderRadius::all(Val::Px(TOPIC_RADIUS_PX)),
                    ..Node::default()
                },
                BackgroundColor(theme::BUTTON_BG),
                ChildOf(topic_list),
            ))
            .with_child((
                Text::new(label),
                TextColor(theme::BUTTON_TEXT),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: TOPIC_FONT_SIZE_PX,
                    ..default()
                },
            ));
    }

    // Clear content panel and show prompt
    for entity in &content_item_q {
        commands.entity(entity).despawn();
    }
    let Ok(content) = content_panel_q.single() else {
        return;
    };
    commands.spawn((
        LoreContentItem,
        Text::new(locale.get("ui.lore.select_topic").to_string()),
        TextColor(theme::BUTTON_TEXT),
        TextFont {
            font: fonts.0.clone(),
            font_size: EMPTY_FONT_SIZE_PX,
            ..default()
        },
        ChildOf(content),
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn handle_topic_buttons(
    mut interaction_q: Query<
        (&Interaction, &LoreTopicButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut selection: ResMut<LoreSelection>,
    lore_book: Res<LoreBook>,
    locale: Res<LocaleMap>,
    asset_server: Res<AssetServer>,
    fonts: Res<UiFont>,
    content_panel_q: Query<Entity, With<LoreContentPanel>>,
    content_item_q: Query<Entity, With<LoreContentItem>>,
    mut commands: Commands,
) {
    let mut new_topic = None;

    for (interaction, button, mut bg) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => {
                new_topic = Some(button.0.clone());
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(theme::BUTTON_BG);
            }
        }
    }

    let Some(topic) = new_topic else { return };
    selection.topic = Some(topic.clone());

    // Clear content panel
    for entity in &content_item_q {
        commands.entity(entity).despawn();
    }
    let Ok(content) = content_panel_q.single() else {
        return;
    };

    // Topic header with optional portrait
    let topic_image: Option<String> = lore_book.topic_image(&topic).map(String::from);
    if let Some(ref image_path) = topic_image {
        commands.spawn((
            LoreContentItem,
            ImageNode::new(asset_server.load(image_path.clone())),
            Node {
                width: Val::Px(PORTRAIT_SIZE_PX),
                height: Val::Px(PORTRAIT_SIZE_PX),
                margin: UiRect::bottom(Val::Px(PORTRAIT_MARGIN_BOTTOM_PX)),
                ..Node::default()
            },
            ChildOf(content),
        ));
    }

    // Topic title
    let topic_name = locale.get(&topic).to_string();
    commands.spawn((
        LoreContentItem,
        Text::new(topic_name),
        TextColor(theme::TITLE),
        TextFont {
            font: fonts.0.clone(),
            font_size: ENTRY_SPEAKER_FONT_SIZE_PX,
            ..default()
        },
        Node {
            margin: UiRect::bottom(Val::Px(ENTRY_MARGIN_BOTTOM_PX)),
            ..Node::default()
        },
        ChildOf(content),
    ));

    // Entries for this topic
    let entries = lore_book.entries_for_topic(&topic);
    for entry in entries {
        let card = commands
            .spawn((
                LoreContentItem,
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(ENTRY_PADDING_PX)),
                    margin: UiRect::bottom(Val::Px(ENTRY_MARGIN_BOTTOM_PX)),
                    border: UiRect::all(Val::Px(ENTRY_BORDER_PX)),
                    border_radius: BorderRadius::all(Val::Px(ENTRY_RADIUS_PX)),
                    ..Node::default()
                },
                BackgroundColor(theme::DIALOG_CHOICE_BG),
                BorderColor::all(theme::DIALOG_BORDER),
                ChildOf(content),
            ))
            .id();

        // Speaker name
        commands.spawn((
            Text::new(locale.get(&entry.speaker_key).to_string()),
            TextColor(theme::DIALOG_SPEAKER),
            TextFont {
                font: fonts.0.clone(),
                font_size: ENTRY_SPEAKER_FONT_SIZE_PX,
                ..default()
            },
            ChildOf(card),
        ));

        // Dialogue lines
        for line_key in &entry.lines_seen {
            commands.spawn((
                Text::new(format!("  {}", locale.get(line_key))),
                TextColor(theme::DIALOG_TEXT),
                TextFont {
                    font: fonts.0.clone(),
                    font_size: ENTRY_TEXT_FONT_SIZE_PX,
                    ..default()
                },
                ChildOf(card),
            ));
        }
    }
}
