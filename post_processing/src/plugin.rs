use bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin;
use bevy::prelude::*;
use models::game_states::{should_despawn_world, GameState};
use models::time::GameClock;

use crate::atmosphere::BiomeAtmosphere;
use crate::clock::tick_game_clock;
use crate::grading::{reset_color_grading, sync_color_grading};
use crate::sync;

pub struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<BiomeAtmosphere>::default());

        app.init_resource::<GameClock>();

        app.add_systems(
            Update,
            (sync::sync_atmosphere, tick_game_clock, sync_color_grading)
                .run_if(in_state(GameState::Playing)),
        );

        // Reset only on true world teardown -- keep grading during Paused/Dialogue.
        app.add_systems(
            OnExit(GameState::Playing),
            reset_color_grading.run_if(should_despawn_world),
        );
    }
}
