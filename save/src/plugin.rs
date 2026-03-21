use bevy::prelude::*;
use dialog::history::LoreBook;
use keybinds::Keybinds;
use models::settings::GameSettings;

use crate::file;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameSettings::default());
        app.add_systems(PreStartup, load_from_storage);
        app.add_systems(PostUpdate, save_on_change);
    }
}

fn load_from_storage(
    mut keybinds: ResMut<Keybinds>,
    mut lore: ResMut<LoreBook>,
    mut settings: ResMut<GameSettings>,
) {
    if let Some(saved) = file::load() {
        file::apply(saved, &mut keybinds, &mut lore, &mut settings);
    }
}

fn save_on_change(
    keybinds: Res<Keybinds>,
    lore: Res<LoreBook>,
    settings: Res<GameSettings>,
) {
    if !keybinds.is_changed() && !lore.is_changed() && !settings.is_changed() {
        return;
    }
    file::persist(&file::from_resources(&keybinds, &lore, &settings));
}
