//! Dialogue runner: state machine + per-phase systems.

pub(crate) mod advance;
pub(crate) mod choice;
pub(crate) mod detect;
pub(crate) mod end;
pub(crate) mod start;
pub(crate) mod state;

pub use advance::advance_runner;
pub use choice::handle_choice;
pub use detect::{detect_interact_input, detect_interact_range};
pub use end::on_dialogue_ended;
pub use start::start_dialogue;
pub use state::{DialogueRunner, DialogueTarget};
