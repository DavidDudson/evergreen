//! In-memory index of every loaded [`Quest`] keyed by [`QuestId`].
//!
//! Populated by [`drain_quest_assets`] each frame as `QuestAsset` handles
//! finish loading. Read-only after that.

use std::collections::HashMap;

use bevy::asset::{AssetEvent, AssetServer, Assets, Handle};
use bevy::prelude::*;

use crate::asset::QuestAsset;
use crate::model::{Quest, QuestId};

/// Resource: every quest definition currently loaded.
#[derive(Resource, Debug, Default)]
pub struct QuestRegistry {
    quests: HashMap<QuestId, Quest>,
}

impl QuestRegistry {
    pub fn get(&self, id: &QuestId) -> Option<&Quest> {
        self.quests.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&QuestId, &Quest)> {
        self.quests.iter()
    }

    pub fn len(&self) -> usize {
        self.quests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quests.is_empty()
    }
}

/// Resource: outstanding quest asset handles. Held to keep them alive while
/// the asset server loads them; drained once each loads.
#[derive(Resource, Debug, Default)]
pub struct QuestHandles(pub Vec<Handle<QuestAsset>>);

/// Startup system: kicks off loads for every `*.quest.ron` we know about.
///
/// Bevy's asset server cannot enumerate a directory on the wasm target, so we
/// list the quest files explicitly. Add a new entry here when authoring a
/// new quest.
pub fn load_quest_manifest(mut handles: ResMut<QuestHandles>, asset_server: Res<AssetServer>) {
    const QUEST_FILES: &[&str] = &["quests/bigby_sick_animals.quest.ron"];
    handles.0 = QUEST_FILES
        .iter()
        .map(|path| asset_server.load::<QuestAsset>(*path))
        .collect();
}

/// Update system: copy newly-loaded `QuestAsset` payloads into [`QuestRegistry`].
pub fn drain_quest_assets(
    mut events: MessageReader<AssetEvent<QuestAsset>>,
    assets: Res<Assets<QuestAsset>>,
    mut registry: ResMut<QuestRegistry>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => *id,
            _ => continue,
        };
        let Some(asset) = assets.get(id) else {
            continue;
        };
        registry.quests.insert(asset.0.id.clone(), asset.0.clone());
    }
}
