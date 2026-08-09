//! Quest log: full-screen panel listing every quest the player has touched,
//! with milestone progress and the lore body.
//!
//! State: `GameState::QuestLog`. Open from `Playing` with the `OpenQuestLog`
//! keybind (default J), close with the back button or `Pause` (Esc).

use bevy::prelude::*;
use dialog::locale::LocaleMap;
use models::game_states::GameState;
use quest::log::{build_log, QuestLogEntry};
use quest::progress::{QuestProgress, QuestStatus};
use quest::registry::QuestRegistry;

use crate::fonts::UiFont;
use crate::theme;

const PAGE_PADDING_PX: f32 = 32.0;
const TITLE_FONT_SIZE_PX: f32 = 30.0;
const TITLE_MARGIN_BOTTOM_PX: f32 = 16.0;

const SIDEBAR_WIDTH_PX: f32 = 220.0;
const SIDEBAR_GAP_PX: f32 = 16.0;

const ENTRY_FONT_SIZE_PX: f32 = 16.0;
const ENTRY_PADDING_H_PX: f32 = 12.0;
const ENTRY_PADDING_V_PX: f32 = 8.0;
const ENTRY_MARGIN_PX: f32 = 3.0;
const ENTRY_RADIUS_PX: f32 = 4.0;

const SECTION_FONT_SIZE_PX: f32 = 18.0;
const SECTION_MARGIN_TOP_PX: f32 = 12.0;
const SECTION_MARGIN_BOTTOM_PX: f32 = 6.0;
const BODY_FONT_SIZE_PX: f32 = 14.0;
const BODY_MARGIN_BOTTOM_PX: f32 = 6.0;

const STATUS_FONT_SIZE_PX: f32 = 12.0;
const STATUS_MARGIN_BOTTOM_PX: f32 = 8.0;

const BACK_FONT_SIZE_PX: f32 = 16.0;
const BACK_PADDING_H_PX: f32 = 20.0;
const BACK_PADDING_V_PX: f32 = 8.0;
const BACK_MARGIN_TOP_PX: f32 = 12.0;

#[derive(Component)]
pub struct QuestLogScreen;

#[derive(Component)]
pub(crate) struct QuestLogEntryButton(pub String);

#[derive(Component)]
pub(crate) struct QuestLogContentPanel;

#[derive(Component)]
pub(crate) struct QuestLogContentItem;

#[derive(Component)]
pub(crate) struct QuestLogBackButton;

/// Currently selected quest id (string form). `None` until the player picks.
#[derive(Resource, Default)]
pub(crate) struct QuestLogSelection(pub Option<String>);

pub struct QuestLogScreenSetup;

impl crate::screen::ScreenSetup for QuestLogScreenSetup {
    fn register(app: &mut App) {
        app.add_systems(OnEnter(GameState::QuestLog), setup)
            .add_systems(OnExit(GameState::QuestLog), teardown)
            .add_systems(
                Update,
                (handle_entry_buttons, handle_back_button).run_if(in_state(GameState::QuestLog)),
            );
    }
}

fn setup(
    mut commands: Commands,
    registry: Res<QuestRegistry>,
    progress: Res<QuestProgress>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
) {
    commands.insert_resource(QuestLogSelection::default());

    let root = commands
        .spawn((
            QuestLogScreen,
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

    commands.spawn((
        Text::new(locale.get("ui.quest_log.title").to_string()),
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

    let entries = build_log(&registry, &progress);
    if entries.is_empty() {
        commands.spawn((
            Text::new(locale.get("ui.quest_log.empty").to_string()),
            TextColor(theme::BUTTON_TEXT),
            TextFont {
                font: fonts.0.clone(),
                font_size: ENTRY_FONT_SIZE_PX,
                ..default()
            },
            ChildOf(sidebar),
        ));
    }
    for entry in &entries {
        let title = locale.get(&entry.quest.title_key).to_string();
        let badge = status_badge(entry.status);
        let label = format!("{badge} {title}");
        commands
            .spawn((
                QuestLogEntryButton(entry.quest.id.as_str().to_string()),
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(ENTRY_PADDING_H_PX), Val::Px(ENTRY_PADDING_V_PX)),
                    margin: UiRect::bottom(Val::Px(ENTRY_MARGIN_PX)),
                    border_radius: BorderRadius::all(Val::Px(ENTRY_RADIUS_PX)),
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
                    font_size: ENTRY_FONT_SIZE_PX,
                    ..default()
                },
            ));
    }

    commands.spawn((
        QuestLogContentPanel,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            overflow: Overflow::scroll_y(),
            ..Node::default()
        },
        ChildOf(body),
    ));

    crate::widgets::ButtonBuilder::new(
        locale.get("ui.quest_log.back").to_string(),
        QuestLogBackButton,
        fonts.0.clone(),
    )
    .padding(BACK_PADDING_H_PX, BACK_PADDING_V_PX)
    .font_size(BACK_FONT_SIZE_PX)
    .margin(BACK_MARGIN_TOP_PX, 0.0)
    .spawn(&mut commands, root);
}

fn teardown(mut commands: Commands, query: Query<Entity, With<QuestLogScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<QuestLogSelection>();
}

fn status_badge(status: QuestStatus) -> &'static str {
    match status {
        QuestStatus::Offered => "*",
        QuestStatus::Active => ">",
        QuestStatus::Completed => "v",
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_entry_buttons(
    mut interaction_q: Query<
        (&Interaction, &QuestLogEntryButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut selection: ResMut<QuestLogSelection>,
    registry: Res<QuestRegistry>,
    progress: Res<QuestProgress>,
    locale: Res<LocaleMap>,
    fonts: Res<UiFont>,
    panel_q: Query<Entity, With<QuestLogContentPanel>>,
    item_q: Query<Entity, With<QuestLogContentItem>>,
    mut commands: Commands,
) {
    let mut new_pick: Option<String> = None;
    for (interaction, button, mut bg) in &mut interaction_q {
        match interaction {
            Interaction::Pressed => new_pick = Some(button.0.clone()),
            Interaction::Hovered => *bg = BackgroundColor(theme::DIALOG_CHOICE_HOVER),
            Interaction::None => *bg = BackgroundColor(theme::BUTTON_BG),
        }
    }
    let Some(quest_id) = new_pick else {
        return;
    };
    selection.0 = Some(quest_id.clone());

    for entity in &item_q {
        commands.entity(entity).despawn();
    }
    let Ok(panel) = panel_q.single() else {
        return;
    };

    let entries = build_log(&registry, &progress);
    let Some(entry) = entries
        .iter()
        .find(|e| e.quest.id.as_str() == quest_id.as_str())
    else {
        return;
    };
    render_entry(&mut commands, panel, entry, &locale, &fonts);
}

fn render_entry(
    commands: &mut Commands,
    panel: Entity,
    entry: &QuestLogEntry<'_>,
    locale: &LocaleMap,
    fonts: &UiFont,
) {
    let title = locale.get(&entry.quest.title_key).to_string();
    spawn_text(
        commands,
        panel,
        title,
        SECTION_FONT_SIZE_PX,
        theme::TITLE,
        fonts,
        SECTION_MARGIN_BOTTOM_PX,
    );

    let status_label = locale
        .get(match entry.status {
            QuestStatus::Offered => "ui.quest_log.status.offered",
            QuestStatus::Active => "ui.quest_log.status.active",
            QuestStatus::Completed => "ui.quest_log.status.completed",
        })
        .to_string();
    spawn_text(
        commands,
        panel,
        status_label,
        STATUS_FONT_SIZE_PX,
        theme::DIALOG_CHOICE_HOVER,
        fonts,
        STATUS_MARGIN_BOTTOM_PX,
    );

    let description = locale.get(&entry.quest.description_key).to_string();
    spawn_text(
        commands,
        panel,
        description,
        BODY_FONT_SIZE_PX,
        theme::DIALOG_TEXT,
        fonts,
        BODY_MARGIN_BOTTOM_PX,
    );

    spawn_text(
        commands,
        panel,
        locale.get("ui.quest_log.section.milestones").to_string(),
        SECTION_FONT_SIZE_PX,
        theme::TITLE,
        fonts,
        SECTION_MARGIN_BOTTOM_PX,
    );

    let total = entry.quest.milestones.len();
    for (i, ms) in entry.quest.milestones.iter().enumerate() {
        let done = i < entry.completed.len();
        let active = i == entry.completed.len() && !matches!(entry.status, QuestStatus::Completed);
        let prefix = if done {
            "[x]"
        } else if active {
            "[>]"
        } else {
            "[ ]"
        };
        let body = format!("{prefix} {}", locale.get(&ms.label_key));
        spawn_text(
            commands,
            panel,
            body,
            BODY_FONT_SIZE_PX,
            theme::DIALOG_TEXT,
            fonts,
            BODY_MARGIN_BOTTOM_PX,
        );
    }
    let _ = total;

    spawn_text(
        commands,
        panel,
        locale.get("ui.quest_log.section.lore").to_string(),
        SECTION_FONT_SIZE_PX,
        theme::TITLE,
        fonts,
        SECTION_MARGIN_BOTTOM_PX,
    );
    spawn_text(
        commands,
        panel,
        locale.get(&entry.quest.lore.body_key).to_string(),
        BODY_FONT_SIZE_PX,
        theme::DIALOG_TEXT,
        fonts,
        BODY_MARGIN_BOTTOM_PX,
    );
}

fn spawn_text(
    commands: &mut Commands,
    parent: Entity,
    body: String,
    font_size: f32,
    color: Color,
    fonts: &UiFont,
    margin_bottom_px: f32,
) {
    commands.spawn((
        QuestLogContentItem,
        Text::new(body),
        TextColor(color),
        TextFont {
            font: fonts.0.clone(),
            font_size,
            ..default()
        },
        Node {
            margin: UiRect {
                top: Val::Px(SECTION_MARGIN_TOP_PX),
                bottom: Val::Px(margin_bottom_px),
                ..UiRect::ZERO
            },
            ..Node::default()
        },
        ChildOf(parent),
    ));
}

fn handle_back_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<QuestLogBackButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_q {
        if matches!(interaction, Interaction::Pressed) {
            next_state.set(GameState::Playing);
        }
    }
}
