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
use keybinds::ActionInput;
use models::alignment::{AlignmentFaction, PlayerAlignment};
use models::speed::Speed;

use crate::inventory::Inventory;

/// Range (pixels) at which the magnifying-glass prompt appears.
pub const INVESTIGATE_RADIUS_PX: f32 = 40.0;

/// One follow-up choice surfaced after investigation. Selecting it grants
/// any configured rewards and sets any configured flag, then closes the
/// popup.
#[derive(Debug, Clone)]
pub struct InvestigateChoice {
    /// Locale key for the button label.
    pub text_key: String,
    /// Optional alignment grant (+1) when this choice is picked.
    pub alignment_grant: Option<AlignmentFaction>,
    /// Optional inventory item granted on this choice. `(item_id, count)`.
    pub item_grant: Option<(String, u32)>,
    /// Optional dialogue flag set on this choice (post-pick state, distinct
    /// from the investigatable's own `flag_to_set` which fires immediately).
    pub flag_to_set: Option<String>,
    /// Optional locale key for a brief response shown in place of the
    /// description after the choice is picked.
    pub response_key: Option<String>,
}

/// World object the player can examine. Carries the localised description,
/// the flag to set on examination, and an optional item grant.
#[derive(Component, Debug, Clone)]
pub struct Investigatable {
    /// Locale key for the description shown in the investigation popup.
    pub description_key: String,
    /// Dialogue flag set when investigated. Quest milestones key off this.
    pub flag_to_set: String,
    /// Optional inventory item granted on investigation. `(item_id, count)`.
    /// Ignored when `choices` is non-empty -- in that case the item lives
    /// on the picked [`InvestigateChoice`].
    pub item_grant: Option<(String, u32)>,
    /// Optional follow-up choices. Empty = no choice, fire-and-close.
    /// Non-empty = popup shows each as a button; nothing is granted by
    /// the investigation itself except `flag_to_set`.
    pub choices: Vec<InvestigateChoice>,
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
            choices: Vec::new(),
            repeat: false,
            investigated: false,
        }
    }

    pub fn with_item(mut self, item_id: impl Into<String>, count: u32) -> Self {
        self.item_grant = Some((item_id.into(), count));
        self
    }

    pub fn with_choices(mut self, choices: Vec<InvestigateChoice>) -> Self {
        self.choices = choices;
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
    /// `Some(item_id)` if an item was granted directly by the investigation,
    /// for toast UI. Always `None` when `choices` is non-empty.
    pub item_granted: Option<String>,
    /// Follow-up choices the UI should present. Empty = no choice prompt.
    pub choices: Vec<InvestigateChoice>,
}

/// Sent by the popup UI when the player picks one of the choices presented
/// for an [`InvestigationFired`] event. Quest plugin applies effects.
#[derive(Message, Debug, Clone)]
pub struct InvestigateChoiceMade {
    pub choice: InvestigateChoice,
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
            commands
                .entity(player_entity)
                .remove::<InvestigateTrigger>();
        }
        (Some(t), Some(c)) if t != c => {
            commands
                .entity(player_entity)
                .insert(InvestigateTrigger { target: t });
        }
        _ => {}
    }
}

/// Press Interact while in range to fire the investigation. Always sets the
/// investigatable's `flag_to_set` (so milestones fire on observation). When
/// the investigatable carries choices, they ride along on the event for the
/// popup UI; otherwise the configured `item_grant` is applied immediately.
#[allow(clippy::too_many_arguments)]
pub fn detect_investigate_input(
    input: ActionInput,
    player_q: Query<&InvestigateTrigger>,
    mut targets: Query<&mut Investigatable>,
    mut flags: ResMut<DialogueFlags>,
    mut inventory: ResMut<Inventory>,
    mut writer: MessageWriter<InvestigationFired>,
) {
    if !input.just_pressed(Action::Interact) {
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

    // Auto-grant only when no follow-up choice is offered. With choices,
    // grants live on the picked `InvestigateChoice` and apply later.
    let item_granted = if investigatable.choices.is_empty() {
        investigatable.item_grant.as_ref().map(|(id, count)| {
            inventory.grant(id.clone(), *count);
            id.clone()
        })
    } else {
        None
    };

    writer.write(InvestigationFired {
        description_key: investigatable.description_key.clone(),
        item_granted,
        choices: investigatable.choices.clone(),
    });
}

/// Apply the side effects of an [`InvestigateChoiceMade`] event: alignment
/// grant, inventory grant, follow-up flag.
pub fn apply_investigate_choice(
    mut events: MessageReader<InvestigateChoiceMade>,
    mut alignment: ResMut<PlayerAlignment>,
    mut inventory: ResMut<Inventory>,
    mut flags: ResMut<DialogueFlags>,
) {
    for event in events.read() {
        if let Some(faction) = event.choice.alignment_grant {
            alignment.grant(faction);
        }
        if let Some((id, count)) = &event.choice.item_grant {
            inventory.grant(id.clone(), *count);
        }
        if let Some(flag) = &event.choice.flag_to_set {
            flags.set(flag.clone());
        }
    }
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
