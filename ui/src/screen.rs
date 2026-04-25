//! Screen self-registration trait.
//!
//! Each menu/screen module implements [`ScreenSetup`] and registers its
//! own systems. `UiPlugin::build` just calls `Screen::register(app)` for
//! each screen instead of holding a giant per-screen match.

use bevy::prelude::App;

pub trait ScreenSetup {
    fn register(app: &mut App);
}
