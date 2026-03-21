use bevy::prelude::*;
use dialog::history::LoreBook;
use keybinds::Keybinds;

use crate::file;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_from_storage);
        app.add_systems(PostUpdate, save_on_change);
    }
}

fn load_from_storage(mut keybinds: ResMut<Keybinds>, mut lore: ResMut<LoreBook>) {
    if let Some(saved) = file::load() {
        file::apply(saved, &mut keybinds, &mut lore);
    }
}

fn save_on_change(keybinds: Res<Keybinds>, lore: Res<LoreBook>) {
    if !keybinds.is_changed() && !lore.is_changed() {
        return;
    }
    file::persist(&file::from_resources(&keybinds, &lore));
}
