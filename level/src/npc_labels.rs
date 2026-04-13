//! Floating name-label and interact-icon system for NPC entities.
//!
//! - **Label**: a [`Text2d`] child spawned once per NPC (via [`Added<Talker>`]),
//!   showing the NPC's display name.
//! - **InteractIcon**: a "!" [`Text2d`] child spawned on whichever NPC the player
//!   is currently in range of, driven by the [`DialogueTrigger`] on the player.

use bevy::prelude::*;
use dialog::components::{DialogueTrigger, Talker};
use models::layer::Layer;
use models::palette;
use models::speed::Speed;

/// Y offset of the name label above the NPC entity origin (pixels).
const LABEL_Y_OFFSET_PX: f32 = 22.0;
const LABEL_FONT_SIZE_PX: f32 = 11.0;

/// Y offset and Z of the interact "!" icon (above the label).
const ICON_Y_OFFSET_PX: f32 = 34.0;
const ICON_FONT_SIZE_PX: f32 = 13.0;

/// World-space z for labels and icons — above all scenery.
/// Labels/icons are children of the NPC entity (NpcLabel z), so their local z
/// must place their world z above SceneryFlower (12).
const LABEL_LOCAL_Z: f32 = Layer::NpcLabel.z_f32() - Layer::World.z_f32();
const ICON_LOCAL_Z: f32 = LABEL_LOCAL_Z + 1.0;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker on the [`Text2d`] name-label child entity.
#[derive(Component)]
pub struct NpcLabel;

/// Marker on the "!" interact prompt child entity.
#[derive(Component)]
pub struct InteractIcon;

// ---------------------------------------------------------------------------
// Resource — tracks which icon entity is currently live
// ---------------------------------------------------------------------------

/// Tracks the currently displayed interact icon so we can despawn it cheaply.
#[derive(Resource, Default)]
pub struct InteractIconState {
    icon_entity: Option<Entity>,
    for_npc: Option<Entity>,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Attaches a floating name label above each newly spawned NPC with a [`Talker`].
/// Uses [`Added<Talker>`] so it fires exactly once per NPC, the frame it spawns.
pub fn attach_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    npc_q: Query<(Entity, &Name), Added<Talker>>,
) {
    for (entity, name) in &npc_q {
        commands.spawn((
            NpcLabel,
            Text2d::new(name.as_str()),
            TextFont {
                font: asset_server.load("fonts/NotoSans-Regular.ttf"),
                font_size: LABEL_FONT_SIZE_PX,
                ..default()
            },
            TextColor(palette::BARK_TEXT),
            Transform::from_xyz(0.0, LABEL_Y_OFFSET_PX, LABEL_LOCAL_Z),
            ChildOf(entity),
        ));
    }
}

/// Syncs the interact "!" icon with the player's current [`DialogueTrigger`].
///
/// Spawns the icon as a child of the targeted NPC; despawns it when the player
/// moves out of range or the target changes.
pub fn sync_interact_icon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_q: Query<Option<&DialogueTrigger>, With<Speed>>,
    mut state: ResMut<InteractIconState>,
) {
    let target_npc = player_q
        .single()
        .ok()
        .and_then(|opt| opt.map(|t| t.npc));

    if target_npc == state.for_npc {
        return;
    }

    // Despawn the old icon (if any).
    if let Some(old) = state.icon_entity.take() {
        commands.entity(old).despawn();
    }

    // Spawn a new icon if there is a target NPC.
    state.icon_entity = target_npc.map(|npc| {
        commands
            .spawn((
                InteractIcon,
                Text2d::new("!"),
                TextFont {
                    font: asset_server.load("fonts/NotoSans-Regular.ttf"),
                    font_size: ICON_FONT_SIZE_PX,
                    ..default()
                },
                TextColor(palette::INTERACT_PROMPT),
                Transform::from_xyz(0.0, ICON_Y_OFFSET_PX, ICON_LOCAL_Z),
                ChildOf(npc),
            ))
            .id()
    });

    state.for_npc = target_npc;
}
