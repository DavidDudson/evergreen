use bevy::prelude::*;
use dialog::history::{LoreBook, LoreEntry};
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

const FILTER_FONT_SIZE_PX: f32 = 14.0;
const FILTER_PADDING_H_PX: f32 = 12.0;
const FILTER_PADDING_V_PX: f32 = 6.0;
const FILTER_MARGIN_PX: f32 = 4.0;
const FILTER_RADIUS_PX: f32 = 4.0;
const FILTER_ROW_MARGIN_BOTTOM_PX: f32 = 20.0;

const ENTRY_SPEAKER_FONT_SIZE_PX: f32 = 16.0;
const ENTRY_TEXT_FONT_SIZE_PX: f32 = 14.0;
const ENTRY_MARGIN_BOTTOM_PX: f32 = 16.0;
const ENTRY_PADDING_PX: f32 = 12.0;
const ENTRY_RADIUS_PX: f32 = 6.0;
const ENTRY_BORDER_PX: f32 = 1.0;

const EMPTY_FONT_SIZE_PX: f32 = 16.0;

const BACK_FONT_SIZE_PX: f32 = 18.0;
const BACK_PADDING_H_PX: f32 = 24.0;
const BACK_PADDING_V_PX: f32 = 10.0;
const BACK_MARGIN_TOP_PX: f32 = 16.0;
const BACK_BORDER_PX: f32 = 2.0;
const BACK_RADIUS_PX: f32 = 6.0;

// ---------------------------------------------------------------------------
// Components / state
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct LorePage;

/// Marker for filter bar buttons (so they can be removed on re-filter).
#[derive(Component)]
pub(crate) struct LoreFilterButton(String);

#[derive(Component)]
pub(crate) struct LoreBackButton;

/// Marker for entry cards — bulk-despawned on filter change.
#[derive(Component)]
pub(crate) struct LoreEntryCard;

#[derive(Component)]
pub(crate) struct LoreEntriesRoot;

/// Currently active tag filter. Empty string = show all.
#[derive(Resource, Default)]
pub(crate) struct LoreFilter(String);

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

pub fn setup(
    mut commands: Commands,
    lore_book: Res<LoreBook>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
) {
    commands.insert_resource(LoreFilter::default());

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

    // Filter row
    let filter_row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                margin: UiRect::bottom(Val::Px(FILTER_ROW_MARGIN_BOTTOM_PX)),
                ..Node::default()
            },
            ChildOf(root),
        ))
        .id();

    // "All" filter button
    commands
        .spawn((
            LoreFilterButton(String::new()),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(FILTER_PADDING_H_PX), Val::Px(FILTER_PADDING_V_PX)),
                margin: UiRect::all(Val::Px(FILTER_MARGIN_PX)),
                border_radius: BorderRadius::all(Val::Px(FILTER_RADIUS_PX)),
                ..Node::default()
            },
            BackgroundColor(theme::BUTTON_BG),
            ChildOf(filter_row),
        ))
        .with_child((
            Text::new(locale.get("ui.lore.filter.all").to_string()),
            TextColor(theme::BUTTON_TEXT),
            TextFont { font: fonts.0.clone(), font_size: FILTER_FONT_SIZE_PX, ..default() },
        ));

    // Tag-specific filter buttons
    let tags = collect_unique_tags(&lore_book);
    for tag in tags {
        commands
            .spawn((
                LoreFilterButton(tag.clone()),
                Button,
                Node {
                    padding: UiRect::axes(
                        Val::Px(FILTER_PADDING_H_PX),
                        Val::Px(FILTER_PADDING_V_PX),
                    ),
                    margin: UiRect::all(Val::Px(FILTER_MARGIN_PX)),
                    border_radius: BorderRadius::all(Val::Px(FILTER_RADIUS_PX)),
                    ..Node::default()
                },
                BackgroundColor(theme::BUTTON_BG),
                ChildOf(filter_row),
            ))
            .with_child((
                Text::new(tag),
                TextColor(theme::BUTTON_TEXT),
                TextFont { font: fonts.0.clone(), font_size: FILTER_FONT_SIZE_PX, ..default() },
            ));
    }

    // Scrollable entries area
    let entries_root = commands
        .spawn((
            LoreEntriesRoot,
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                overflow: Overflow::scroll_y(),
                ..Node::default()
            },
            ChildOf(root),
        ))
        .id();

    spawn_entries(&mut commands, &lore_book, &locale, entries_root, "", fonts.0.clone());

    // Back button
    commands
        .spawn((
            LoreBackButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(BACK_PADDING_H_PX), Val::Px(BACK_PADDING_V_PX)),
                margin: UiRect::top(Val::Px(BACK_MARGIN_TOP_PX)),
                border: UiRect::all(Val::Px(BACK_BORDER_PX)),
                border_radius: BorderRadius::all(Val::Px(BACK_RADIUS_PX)),
                align_self: AlignSelf::FlexStart,
                ..Node::default()
            },
            BorderColor::all(theme::ACCENT),
            BackgroundColor(theme::BUTTON_BG),
            ChildOf(root),
        ))
        .with_child((
            Text::new(locale.get("ui.lore.back").to_string()),
            TextColor(theme::BUTTON_TEXT),
            TextFont { font: fonts.0.clone(), font_size: BACK_FONT_SIZE_PX, ..default() },
        ));
}

pub fn teardown(
    mut commands: Commands,
    query: Query<Entity, With<LorePage>>,
) {
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
    commands.remove_resource::<LoreFilter>();
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

pub fn handle_filter_buttons(
    mut interaction_q: Query<
        (&Interaction, &LoreFilterButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut filter: ResMut<LoreFilter>,
    lore_book: Res<LoreBook>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
    entries_root_q: Query<Entity, With<LoreEntriesRoot>>,
    card_q: Query<Entity, With<LoreEntryCard>>,
    mut commands: Commands,
) {
    let mut new_filter = None;

    for (interaction, button, mut bg) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => {
                new_filter = Some(button.0.clone());
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(theme::BUTTON_BG);
            }
        }
    }

    let Some(tag) = new_filter else { return };
    filter.0 = tag.clone();

    // Despawn all current entry cards.
    for entity in &card_q {
        commands.entity(entity).despawn();
    }

    let Ok(root) = entries_root_q.single() else {
        return;
    };

    spawn_entries(&mut commands, &lore_book, &locale, root, &tag, fonts.0.clone());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_unique_tags(lore_book: &LoreBook) -> Vec<String> {
    let mut tags: Vec<String> = lore_book
        .entries
        .iter()
        .flat_map(|e| e.keyword_tags.iter().cloned())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

fn spawn_entries(
    commands: &mut Commands,
    lore_book: &LoreBook,
    locale: &LocaleMap,
    parent: Entity,
    filter: &str,
    font: Handle<Font>,
) {
    let entries: Vec<&LoreEntry> = lore_book
        .entries
        .iter()
        .filter(|e| filter.is_empty() || e.keyword_tags.iter().any(|t| t == filter))
        .collect();

    if entries.is_empty() {
        commands.spawn((
            LoreEntryCard,
            Text::new(locale.get("ui.lore.empty").to_string()),
            TextColor(theme::BUTTON_TEXT),
            TextFont {
                font: font.clone(),
                font_size: EMPTY_FONT_SIZE_PX,
                ..default()
            },
            ChildOf(parent),
        ));
        return;
    }

    for entry in entries {
        let card = commands
            .spawn((
                LoreEntryCard,
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
                ChildOf(parent),
            ))
            .id();

        commands.spawn((
            Text::new(locale.get(&entry.speaker_key).to_string()),
            TextColor(theme::DIALOG_SPEAKER),
            TextFont {
                font: font.clone(),
                font_size: ENTRY_SPEAKER_FONT_SIZE_PX,
                ..default()
            },
            ChildOf(card),
        ));

        for line_key in &entry.lines_seen {
            commands.spawn((
                Text::new(format!("  {}", locale.get(line_key))),
                TextColor(theme::DIALOG_TEXT),
                TextFont {
                    font: font.clone(),
                    font_size: ENTRY_TEXT_FONT_SIZE_PX,
                    ..default()
                },
                ChildOf(card),
            ));
        }
    }
}
