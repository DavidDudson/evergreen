//! Bloody Mary apparition: a short-lived ghostly Mary sprite that appears
//! next to the player whenever they pick the "consume" choice at the
//! purple-pond mirror-shard investigation. She fires a bark encouraging
//! more consumption, then fades and despawns.

use bevy::prelude::*;
use dialog::events::BarkFired;
use models::layer::Layer;
use models::palette;
use models::player::Player;
use quest::interact::InvestigateChoiceMade;
use rand::seq::IndexedRandom;

/// Choice flag that triggers an apparition. Any `InvestigateChoiceMade`
/// whose `flag_to_set` matches this is treated as a consume event.
const CONSUME_FLAG: &str = "quest:bigby.sick_animals:shard:consumed";

const APPARITION_LIFETIME_SECS: f32 = 4.5;
const APPARITION_FADE_SECS: f32 = 1.0;
const APPARITION_OFFSET_X_PX: f32 = 28.0;
const APPARITION_OFFSET_Y_PX: f32 = 6.0;
const APPARITION_DISPLAY_PX: f32 = 32.0;
const APPARITION_FRAME_PX: u32 = 68;
const APPARITION_FRAME_COUNT: u32 = 4;
const APPARITION_FRAME_SECS: f32 = 0.18;
const APPARITION_SHEET: &str = "sprites/portals/mirror_mary_breathing.webp";

const ENCOURAGE_KEYS: &[&str] = &[
    "npc.bloody_mary.encourage.consume.0",
    "npc.bloody_mary.encourage.consume.1",
    "npc.bloody_mary.encourage.consume.2",
    "npc.bloody_mary.encourage.consume.3",
];

/// Component on the temporary apparition sprite. Carries the lifetime
/// timer and the breathing-idle frame timer.
#[derive(Component)]
pub struct MaryApparition {
    pub life: Timer,
    pub frame: Timer,
}

impl Default for MaryApparition {
    fn default() -> Self {
        Self {
            life: Timer::from_seconds(APPARITION_LIFETIME_SECS, TimerMode::Once),
            frame: Timer::from_seconds(APPARITION_FRAME_SECS, TimerMode::Repeating),
        }
    }
}

/// Listen for `InvestigateChoiceMade` events matching the consume flag.
/// Spawn a Mary apparition next to the player and fire a random encourage
/// bark on that entity.
pub fn spawn_mary_on_consume(
    mut commands: Commands,
    mut events: MessageReader<InvestigateChoiceMade>,
    player_q: Query<&GlobalTransform, With<Player>>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut bark_writer: MessageWriter<BarkFired>,
) {
    for event in events.read() {
        let Some(flag) = event.choice.flag_to_set.as_deref() else {
            continue;
        };
        if flag != CONSUME_FLAG {
            continue;
        }
        let Ok(player_tf) = player_q.single() else {
            continue;
        };
        let player_pos = player_tf.translation().truncate();
        let pos = Vec3::new(
            player_pos.x + APPARITION_OFFSET_X_PX,
            player_pos.y + APPARITION_OFFSET_Y_PX,
            Layer::World.z_f32(),
        );
        let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(APPARITION_FRAME_PX),
            APPARITION_FRAME_COUNT,
            1,
            None,
            None,
        ));
        let entity = commands
            .spawn((
                MaryApparition::default(),
                Name::new("Bloody Mary (apparition)"),
                Sprite {
                    image: asset_server.load(APPARITION_SHEET),
                    texture_atlas: Some(TextureAtlas { layout, index: 0 }),
                    custom_size: Some(Vec2::splat(APPARITION_DISPLAY_PX)),
                    color: palette::APPARITION_TINT,
                    ..default()
                },
                Transform::from_translation(pos),
            ))
            .id();

        let mut rng = rand::rng();
        let Some(text_key) = ENCOURAGE_KEYS.choose(&mut rng).map(|s| (*s).to_string()) else {
            continue;
        };
        bark_writer.write(BarkFired {
            npc: entity,
            text_key,
        });
    }
}

/// Despawn every active apparition on world teardown.
pub fn despawn_apparitions(mut commands: Commands, q: Query<Entity, With<MaryApparition>>) {
    for entity in &q {
        commands.entity(entity).despawn();
    }
}

/// Tick the lifetime + breathing animation, fade out in the final second,
/// and despawn when finished.
pub fn tick_mary_apparition(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut MaryApparition, &mut Sprite)>,
) {
    for (entity, mut apparition, mut sprite) in &mut q {
        apparition.life.tick(time.delta());
        apparition.frame.tick(time.delta());

        if apparition.frame.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                let next = atlas.index + 1;
                atlas.index = if next >= usize::try_from(APPARITION_FRAME_COUNT).unwrap_or(4) {
                    0
                } else {
                    next
                };
            }
        }

        let remaining = apparition.life.remaining_secs();
        if remaining < APPARITION_FADE_SECS {
            let base_alpha = palette::APPARITION_TINT.alpha();
            let fade = (remaining / APPARITION_FADE_SECS).clamp(0.0, 1.0);
            sprite.color = sprite.color.with_alpha(base_alpha * fade);
        }

        if apparition.life.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
