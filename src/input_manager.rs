use i_slint_core::items::KeyEvent;

use crate::ui_handles;

pub(crate) fn process_input(key_event: &KeyEvent, ui_weak: &crate::AppWindow) {
    match (key_event.text.chars().next(), key_event.modifiers.control) {
        (Some('p'), true) => ui_handles::filter_next(ui_weak, ui_handles::Direction::Backward),
        (Some('n'), true) => ui_handles::filter_next(ui_weak, ui_handles::Direction::Forward),
        (Some('f'), true) => ui_weak.invoke_focus_filter(),
        (Some('n'), _) => ui_weak.invoke_toolbar_toggle(),

        _ => {}
    }
}
