//! F3 performance overlay.
//!
//! Shows FPS, frame time (avg / min / max), entity count, area, time of day,
//! weather, and a 60-frame timing histogram. Toggle with the
//! `ToggleDiagnosticsOverlay` action (default F3). For per-system profiling
//! open Chrome DevTools > Performance > Record.

pub(crate) mod components;
pub(crate) mod histogram;
pub(crate) mod setup;
pub(crate) mod update;

pub use components::{overlay_visible, OverlayState, PerfOverlay};
pub(crate) use setup::setup_overlay;
pub(crate) use update::{toggle_overlay, update_overlay};
