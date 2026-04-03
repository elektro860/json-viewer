use std::{cell::RefCell, rc::Rc};

use i_slint_core::items::{KeyEvent, KeyboardModifiers};
use slint::platform::Key;

use crate::{json_enviroment::JsonEnviroment, json_utils, ui_handles};

pub(crate) fn process_input(keyEvent: &KeyEvent, ui_weak: &crate::AppWindow) {
    match (keyEvent.text.chars().next(), keyEvent.modifiers.control) {
        (Some('p'), true) => ui_handles::filter_next(&ui_weak, ui_handles::Direction::Backward),
        (Some('n'), true) => ui_handles::filter_next(&ui_weak, ui_handles::Direction::Forward),
        (Some('f'), true) => ui_weak.invoke_focus_filter(),
        (Some('n'), _) => ui_weak.invoke_toolbar_toggle(),

        _ => {}
    }
}
