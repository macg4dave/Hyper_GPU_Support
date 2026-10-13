//! Compiled presentation only: no runtime resource access or unchecked message input.
include!(concat!(env!("OUT_DIR"), "/ui_text.rs"));

/// Select a display message from typed reviewed power, independently of wording.
pub fn power(power: &crate::model::Power) -> UiMessage {
    match power {
        crate::model::Power::Off => common::power::off(),
        crate::model::Power::Running => common::power::running(),
        crate::model::Power::Other(_) => common::state::unknown(),
    }
}
