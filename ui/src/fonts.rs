use bevy::prelude::*;

pub const FONT_PATH: &str = "fonts/NotoSans-Regular.ttf";

/// Holds the pre-loaded handle to the game's main UI font.
#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

impl FromWorld for UiFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load(FONT_PATH))
    }
}
