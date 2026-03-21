---
name: bevy-18
description: Reference for Bevy 0.18 API patterns used in this project. Use proactively when writing any Bevy system, component, event, asset loader, or plugin. Covers renamed APIs, event system changes, and WASM-specific patterns.
tools: Read, Grep
---

# Bevy 0.18 Patterns

This skill documents the Bevy 0.18 API as used in this project. Consult it before writing any Bevy code.

## Events → Messages

Bevy 0.18 renamed the event system. **All old `Event`/`EventReader`/`EventWriter` APIs are gone.**

| Old (≤0.15) | New (0.18) |
|-------------|------------|
| `#[derive(Event)]` | `#[derive(Message)]` |
| `EventReader<T>` | `MessageReader<T>` |
| `EventWriter<T>` | `MessageWriter<T>` |
| `app.add_event::<T>()` | `app.add_message::<T>()` |
| `writer.send(e)` | `writer.write(e)` |
| `reader.read()` | `reader.read()` (same) |

```rust
use bevy::prelude::{Entity, Message};

#[derive(Message, Debug, Clone)]
pub struct DamageEvent { pub target: Entity }

// In system:
fn handle(mut events: MessageReader<DamageEvent>) {
    for event in events.read() { ... }
}

fn fire(mut writer: MessageWriter<DamageEvent>) {
    writer.write(DamageEvent { target: e });
}
```

## Asset Loaders

Custom asset loaders require `TypePath` on the loader struct and use `Box<dyn Error>` for the error type.

```rust
use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use bevy::reflect::TypePath;

#[derive(Asset, TypePath, Debug, serde::Deserialize)]
pub struct MyAsset { ... }

#[derive(Default, TypePath)]
pub struct MyAssetLoader;

impl AssetLoader for MyAssetLoader {
    type Asset = MyAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _ctx: &mut LoadContext<'_>,
    ) -> Result<MyAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes)?;
        Ok(ron::from_str(text)?)
    }

    fn extensions(&self) -> &[&str] { &["myasset.ron"] }
}
```

Register in plugin:
```rust
app.init_asset::<MyAsset>()
   .init_asset_loader::<MyAssetLoader>();
```

**Note:** `futures_lite::AsyncReadExt` is NOT needed — `read_to_end` is available on `Reader` directly in Bevy 0.18.

## Child Entity Spawning

`ChildBuilder` was renamed. Use `ChildOf` component for programmatic child spawning (preferred pattern):

```rust
let parent = commands.spawn(Node { ... }).id();

// Spawn child with ChildOf (matches minimap.rs pattern)
commands.spawn((
    SomeComponent,
    Node { ... },
    ChildOf(parent),
));
```

For inline children, `.with_children(|parent| { ... })` still works.

**`despawn_descendants()` is gone.** Instead query children by marker component:
```rust
// Marker component on children
#[derive(Component)]
struct MyChild;

// Despawn all children
fn clear_children(mut commands: Commands, q: Query<Entity, With<MyChild>>) {
    for entity in &q { commands.entity(entity).despawn(); }
}
```

## Component Visibility

Bevy 0.18 lint `private_interfaces` fires when `pub fn` uses private types in query parameters. Fix by making marker components `pub(crate)`:

```rust
#[derive(Component)]
pub(crate) struct MyMarker;    // not just `struct MyMarker`
```

## State Management

Unchanged from 0.14+:
```rust
app.add_systems(OnEnter(GameState::Playing), setup)
   .add_systems(OnExit(GameState::Playing), teardown.run_if(not(in_state(GameState::Paused))))
   .add_systems(Update, update.run_if(in_state(GameState::Playing)));
```

## WASM Considerations

- Target: `wasm32-unknown-unknown`
- Random: requires `getrandom = { version = "0.2", features = ["js"] }` alongside `rand`
- Local storage: use `web-sys` with `features = ["Window", "Storage"]`
- No filesystem I/O — all persistence via `web_sys::window().local_storage()`

## Despawn Pattern

All crates use `despawn()` (not `despawn_recursive()`):
```rust
pub fn despawn_all<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    query.iter().for_each(|entity| commands.entity(entity).despawn());
}
```

## Palette / Colors

All colors defined in `models/src/palette.rs`. Using inline `Color::srgb()` anywhere else is banned by clippy lint. Add new colors there with `#[allow(clippy::disallowed_methods)]`.
