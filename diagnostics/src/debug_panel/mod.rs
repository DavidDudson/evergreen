//! F5 debug panel. Cycle time-of-day and weather state live.
//!
//! `[` / `]` (or `,` / `.`) -- step current hour between 8 fixed periods.
//! `-` / `=` (or `;` / `/` / `'`) -- cycle `WeatherKind` (Clear/Breezy/Windy/Rain/Storm).

pub(crate) mod components;
pub(crate) mod input;
pub(crate) mod setup;
pub(crate) mod update;

pub use components::{panel_visible, DebugPanel, DebugPanelState};
pub(crate) use input::{handle_debug_input, toggle_debug_panel};
pub(crate) use setup::setup_debug_panel;
pub(crate) use update::update_debug_panel;
