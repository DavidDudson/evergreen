---
name: create-dialog
description: Add dialog scripts, bark pools, or NPC Talker entities to the Evergreen game. Use when the user asks to add dialogue, conversations, lore entries, barks, or NPC interactions.
tools: Read, Write, Edit, Bash, Glob, Grep
---

# Create Dialog Skill

This skill helps add dialog content to the Evergreen game using the `dialog` crate system.

## Architecture Quick Reference

The dialog system lives in `crates/dialog/` and consists of:

| Component | File | Purpose |
|-----------|------|---------|
| Script asset | `assets/dialogue/scripts/*.dialog.ron` | Scripted NPC conversations |
| Bark asset | `assets/dialogue/barks/*.dialog.ron` | Random ambient one-liners |
| Locale strings | `assets/locale/en-US.locale.ron` | All display text as key→value |
| `Talker` component | `dialog/src/components.rs` | Gives an entity scripted dialogue |
| `BarkPool` component | `dialog/src/components.rs` | Gives an entity random barks |
| `DialogueFlags` resource | `dialog/src/flags.rs` | Persistent bool flags for branching |

## Step-by-Step: Adding a New NPC with Dialog

### 1. Add locale strings to `assets/locale/en-US.locale.ron`

```ron
// NPC name
"npc.villager.name": "Villager",

// Script lines
"npc.villager.greeting.0": "Oh! You startled me.",
"npc.villager.greeting.1": "Please be careful out there.",

// Choice labels
"npc.villager.choice.ask": "What do you know?",
"npc.villager.choice.leave": "Never mind.",

// Branch lines
"npc.villager.lore.0": "They say the old oak at the forest's heart remembers everything.",
```

### 2. Create a script at `assets/dialogue/scripts/<npc_name>.dialog.ron`

```ron
DialogueScript(
    id: "villager.greeting",
    speaker_key: "npc.villager.name",
    keyword_tags: ["world", "nature"],
    lines: [
        Speech(text_key: "npc.villager.greeting.0"),
        Speech(text_key: "npc.villager.greeting.1"),
        PlayerChoice(options: [
            ChoiceOption(
                text_key: "npc.villager.choice.ask",
                flags_required: [],
                flags_set: ["heard_villager"],
                next: [
                    Speech(text_key: "npc.villager.lore.0"),
                ],
            ),
            ChoiceOption(
                text_key: "npc.villager.choice.leave",
                flags_required: [],
                flags_set: [],
                next: [],
            ),
        ]),
    ],
)
```

### 3. (Optional) Create bark scripts at `assets/dialogue/barks/<npc_name>_<topic>.dialog.ron`

```ron
DialogueScript(
    id: "villager.bark.worried",
    speaker_key: "npc.villager.name",
    keyword_tags: ["ambient"],
    lines: [
        Speech(text_key: "npc.villager.bark.worried"),
    ],
)
```

Add the bark line to the locale file too.

### 4. Spawn the NPC entity with `Talker` and/or `BarkPool`

```rust
use dialog::components::{BarkPool, Talker};
use bevy::prelude::*;

fn spawn_villager(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // Required for dialog range detection
        Transform::from_xyz(200.0, 0.0, 0.0),
        GlobalTransform::default(),

        // Scripted greeting (fires once per session; use Talker::repeating() for every time)
        Talker::new(asset_server.load("dialogue/scripts/villager.dialog.ron")),

        // Optional: random barks when player is nearby
        BarkPool {
            barks: vec![
                asset_server.load("dialogue/barks/villager_worried.dialog.ron"),
            ],
            trigger_radius_px: 80.0,
            cooldown: Timer::from_seconds(20.0, TimerMode::Repeating),
        },
    ));
}
```

## Flags Reference

Flags are stored in `DialogueFlags` resource (persists across conversations in a session).

- `flags_required: ["flag_name"]` — option only visible if flag is set
- `flags_set: ["flag_name"]` — sets flag when option is chosen

Use flags to unlock follow-up dialogue:
```ron
ChoiceOption(
    text_key: "npc.villager.followup",
    flags_required: ["heard_villager"],   // only shown after first meeting
    flags_set: [],
    next: [ ... ],
),
```

## Lore Page

All completed dialogue scripts are automatically recorded in `LoreBook` and visible on the Lore page from the main menu. The `keyword_tags` field controls which filter tab they appear under.

## Conventions

- Script `id` must be globally unique (used as the lore entry key)
- All display strings must be locale keys — never put literal text in `.dialog.ron` files
- Bark scripts always contain exactly one `Speech` line (the first Speech line is used)
- Keep scripts under 300 lines; split long conversations into multiple linked scripts via flags
