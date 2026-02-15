# Evergreen

A friendly game about a lost druid finding her way in life.

## Run

Although this can be cross compiled to many platforms, the primary platform is WASM.

The project uses [Trunk](https://trunkrs.dev/) as its WASM build tool and dev server.

```bash
trunk serve
```

This will build the project and start a dev server at `http://127.0.0.1:8080` with automatic rebuilds on file changes.

### Prerequisites

- [Trunk](https://trunkrs.dev/): `cargo install trunk`
- WASM target: `rustup target add wasm32-unknown-unknown`
- OS-specific Bevy dependencies — see the [Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/)
