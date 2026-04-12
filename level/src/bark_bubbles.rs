//! Floating speech bubbles for NPC barks.
//!
//! When a `BarkFired` message arrives, a `Text2d` child is spawned above the
//! NPC showing the bark text. The bubble fades out and despawns after a timer.

use bevy::prelude::*;
use dialog::events::BarkFired;
use dialog::locale::LocaleMap;
use models::layer::Layer;
use models::palette;

/// How long the bark bubble stays visible (seconds).
const BUBBLE_DURATION_SECS: f32 = 3.0;
/// Y offset above the NPC entity origin (above the name label and icon).
const BUBBLE_Y_OFFSET_PX: f32 = 40.0;
/// Font size for bark text.
const BUBBLE_FONT_SIZE_PX: f32 = 10.0;
/// Max width before wrapping (pixels).
const BUBBLE_MAX_WIDTH_PX: f32 = 120.0;
/// Local z to place the bubble above all labels/icons.
const BUBBLE_LOCAL_Z: f32 = Layer::NpcLabel.z_f32() - Layer::Npc.z_f32() + 2.0;

/// Marker for bark bubble entities so they can be despawned.
#[derive(Component)]
pub struct BarkBubble {
    timer: Timer,
}

/// Spawns a speech bubble above the NPC when a bark fires.
pub fn spawn_bark_bubble(
    mut commands: Commands,
    mut events: MessageReader<BarkFired>,
    locale: Res<LocaleMap>,
    asset_server: Res<AssetServer>,
    existing_q: Query<(Entity, &ChildOf), With<BarkBubble>>,
) {
    for event in events.read() {
        // Remove any existing bubble on this NPC.
        for (entity, child_of) in &existing_q {
            if child_of.0 == event.npc {
                commands.entity(entity).despawn();
            }
        }

        let text = locale.get(&event.text_key).to_string();

        commands.spawn((
            BarkBubble {
                timer: Timer::from_seconds(BUBBLE_DURATION_SECS, TimerMode::Once),
            },
            Text2d::new(text),
            TextFont {
                font: asset_server.load("fonts/NotoSans-Regular.ttf"),
                font_size: BUBBLE_FONT_SIZE_PX,
                ..default()
            },
            TextColor(palette::BARK_TEXT),
            TextLayout::new_with_linebreak(bevy::text::LineBreak::WordBoundary),
            bevy::text::TextBounds::new_horizontal(BUBBLE_MAX_WIDTH_PX),
            Transform::from_xyz(0.0, BUBBLE_Y_OFFSET_PX, BUBBLE_LOCAL_Z),
            ChildOf(event.npc),
        ));
    }
}

/// Fades out and despawns expired bark bubbles.
pub fn tick_bark_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BarkBubble, &mut TextColor)>,
) {
    for (entity, mut bubble, mut color) in &mut query {
        bubble.timer.tick(time.delta());

        // Fade out in the last second.
        let remaining = bubble.timer.remaining_secs();
        if remaining < 1.0 {
            let alpha = remaining.max(0.0);
            color.0 = color.0.with_alpha(alpha);
        }

        if bubble.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
