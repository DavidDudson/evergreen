//! World-object investigation: a non-dialogue interact path that surfaces a
//! magnifying-glass prompt, fires a one-line description, sets a dialogue
//! flag, and optionally grants an inventory item.
//!
//! Mirrors `dialog`'s [`detect_interact_range`] / [`detect_interact_input`]
//! but for `Investigatable` entities, so quest props don't masquerade as
//! NPCs in the dialog runner.

use bevy::prelude::*;
use dialog::flags::DialogueFlags;
use keybinds::action::Action;
use keybinds::bindings::Keybinds;
use models::speed::Speed;

use crate::inventory::Inventory;

/// Range (pixels) at which the magnifying-glass prompt appears.
pub const INVESTIGATE_RADIUS_PX: f32 = 40.0;

/// World object the player can examine. Carries the localised description,
/// the flag to set on examination, and an optional item grant.
#[derive(Component, Debug, Clone)]
pub struct Investigatable {
    /// Locale key for the description shown in the investigation popup.
    pub description_key: String,
    /// Dialogue flag set when investigated. Quest milestones key off this.
    pub flag_to_set: String,
    /// Optional inventory item granted on investigation. `(item_id, count)`.
    pub item_grant: Option<(String, u32)>,
    /// Whether examining this object multiple times re-fires the popup.
    /// Default `false` -- one investigation per object.
    pub repeat: bool,
    /// Runtime: has this object already been investigated?
    pub investigated: bool,
}

impl Investigatable {
    pub fn new(description_key: impl Into<String>, flag_to_set: impl Into<String>) -> Self {
        Self {
            description_key: description_key.into(),
            flag_to_set: flag_to_set.into(),
            item_grant: None,
            repeat: false,
            investigated: false,
        }
    }

    pub fn with_item(mut self, item_id: impl Into<String>, count: u32) -> Self {
        self.item_grant = Some((item_id.into(), count));
        self
    }
}

/// Inserted on the player when they are in range of an [`Investigatable`].
/// Removed when out of range. Mirrors `dialog::components::DialogueTrigger`.
#[derive(Component, Debug)]
pub struct InvestigateTrigger {
    pub target: Entity,
}

/// Emitted when the player investigates an object. Consumed by UI to show
/// the description popup, by inventory/quest systems for side effects.
#[derive(Message, Debug, Clone)]
pub struct InvestigationFired {
    pub description_key: String,
    /// `Some(item_id)` if an item was granted, for toast UI.
    pub item_granted: Option<String>,
}

/// Each frame: find the nearest [`Investigatable`] within range and attach a
/// trigger component to the player. Detects entities even if they overlap an
/// NPC range, but the dialog detector wins on the interact press (its system
/// runs first).
#[allow(clippy::type_complexity)]
pub fn detect_investigate_range(
    targets: Query<(Entity, &GlobalTransform), With<Investigatable>>,
    player_q: Query<(Entity, &GlobalTransform), (With<Speed>, Without<Investigatable>)>,
    mut commands: Commands,
    trigger_q: Query<&InvestigateTrigger>,
) {
    let Ok((player_entity, player_tf)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation().truncate();

    let nearest = targets
        .iter()
        .filter_map(|(entity, tf)| {
            let dist = player_pos.distance(tf.translation().truncate());
            (dist <= INVESTIGATE_RADIUS_PX).then_some((entity, dist))
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal))
        .map(|(e, _)| e);

    let current = trigger_q.get(player_entity).ok().map(|t| t.target);

    match (nearest, current) {
        (Some(t), None) => {
            commands
                .entity(player_entity)
                .insert(InvestigateTrigger { target: t });
        }
        (None, Some(_)) => {
            commands.entity(player_entity).remove::<InvestigateTrigger>();
        }
        (Some(t), Some(c)) if t != c => {
            commands
                .entity(player_entity)
                .insert(InvestigateTrigger { target: t });
        }
        _ => {}
    }
}

/// Press Interact while in range to fire the investigation. Sets the target's
/// flag, grants any configured item, and emits [`InvestigationFired`].
#[allow(clippy::too_many_arguments)]
pub fn detect_investigate_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<Keybinds>,
    player_q: Query<&InvestigateTrigger>,
    mut targets: Query<&mut Investigatable>,
    mut flags: ResMut<DialogueFlags>,
    mut inventory: ResMut<Inventory>,
    mut writer: MessageWriter<InvestigationFired>,
) {
    if !keyboard.just_pressed(bindings.key(Action::Interact)) {
        return;
    }
    let Ok(trigger) = player_q.single() else {
        return;
    };
    let Ok(mut investigatable) = targets.get_mut(trigger.target) else {
        return;
    };
    if investigatable.investigated && !investigatable.repeat {
        return;
    }
    investigatable.investigated = true;
    flags.set(investigatable.flag_to_set.clone());
    let item_granted = investigatable.item_grant.as_ref().map(|(id, count)| {
        inventory.grant(id.clone(), *count);
        id.clone()
    });
    writer.write(InvestigationFired {
        description_key: investigatable.description_key.clone(),
        item_granted,
    });
}

// ---------------------------------------------------------------------------
// World-space magnifying-glass prompt
// ---------------------------------------------------------------------------

/// Marker for the child sprite that renders the magnifying-glass icon above
/// an investigatable when the player is in range.
#[derive(Component)]
pub struct InvestigatePromptIcon;

/// Asset paths for the prompt icon. Inserted as a resource by [`crate::plugin::QuestPlugin`].
#[derive(Resource, Debug, Clone)]
pub struct InvestigateIconAsset(pub Handle<Image>);

/// Vertical offset of the icon above the investigatable, in pixels.
const ICON_OFFSET_Y_PX: f32 = 20.0;
const ICON_SIZE_PX: f32 = 12.0;

/// Spawns / despawns the magnifying-glass child sprite based on whether each
/// investigatable is currently the player's trigger target.
#[allow(clippy::type_complexity)]
pub fn sync_investigate_prompt(
    mut commands: Commands,
    icon: Option<Res<InvestigateIconAsset>>,
    targets: Query<(Entity, Option<&Children>), With<Investigatable>>,
    player_q: Query<&InvestigateTrigger>,
    existing: Query<Entity, With<InvestigatePromptIcon>>,
) {
    let Some(icon) = icon else {
        return;
    };
    let active = player_q.single().ok().map(|t| t.target);

    for (entity, children) in &targets {
        let has_icon = children.is_some_and(|c| c.iter().any(|child| existing.contains(child)));
        let should_show = Some(entity) == active;
        if should_show && !has_icon {
            let icon_entity = commands
                .spawn((
                    InvestigatePromptIcon,
                    Sprite {
                        image: icon.0.clone(),
                        custom_size: Some(Vec2::splat(ICON_SIZE_PX)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, ICON_OFFSET_Y_PX, 0.1),
                ))
                .id();
            commands.entity(entity).add_child(icon_entity);
        } else if !should_show && has_icon {
            if let Some(children) = children {
                for &child in children {
                    if existing.contains(child) {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}
