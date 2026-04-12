use bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin;
use bevy::prelude::*;
use models::game_states::GameState;

use crate::atmosphere::BiomeAtmosphere;
use crate::sync;

pub struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<BiomeAtmosphere>::default());

        app.add_systems(
            Update,
            sync::sync_atmosphere.run_if(in_state(GameState::Playing)),
        );
    }
}
