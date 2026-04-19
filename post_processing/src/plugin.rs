use bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin;
use bevy::prelude::*;
use models::game_states::GameState;
use models::time::GameClock;

use crate::atmosphere::BiomeAtmosphere;
use crate::grading::{reset_color_grading, sync_color_grading};
use crate::sync;
use crate::time_sync;

pub struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<BiomeAtmosphere>::default());

        app.init_resource::<GameClock>();

        app.add_systems(
            Update,
            (
                sync::sync_atmosphere,
                time_sync::tick_game_clock,
                sync_color_grading,
            )
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            PostUpdate,
            time_sync::sync_time_of_day.run_if(in_state(GameState::Playing)),
        );

        app.add_systems(OnExit(GameState::Playing), reset_color_grading);
    }
}
