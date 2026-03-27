use std::{cell::RefCell, rc::Rc};

use i_slint_core::items::{KeyEvent, KeyboardModifiers};
use slint::platform::Key;

use crate::ui_handles;

pub(crate) fn process_input(
    keyEvent: &KeyEvent,
    ui_weak: &crate::AppWindow,
    filtered_values_ref: &Rc<RefCell<Vec<i32>>>,
) {
    match (keyEvent.text.chars().next(), keyEvent.modifiers.control) {
        (Some('p'), true) => ui_handles::filter_next(
            &ui_weak,
            &filtered_values_ref,
            ui_handles::Direction::Backward,
        ),
        (Some('n'), true) => ui_handles::filter_next(
            &ui_weak,
            &filtered_values_ref,
            ui_handles::Direction::Forward,
        ),
        (Some('f'), true) => ui_weak.invoke_focus_filter(),
        (Some('n'), _) => ui_weak.invoke_toolbar_toggle(),

        _ => {}
    }
}
