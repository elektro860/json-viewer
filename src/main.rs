// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    cell::RefCell,
    env,
    error::Error,
    mem::take,
    ops::Deref,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
};

use regex::Regex;
use rfd::FileDialog;
use slint::{ComponentHandle, ModelRc, VecModel};

slint::include_modules!();

mod json_utils;
mod ui_handles;

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let mut value_vec: Rc<RefCell<Vec<JsonValue>>> = Rc::new(RefCell::new(Vec::new()));
    let mut filtered_values: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
    // let mut value_vec: Rc<RefCell<Vec<JsonValue>>> = Rc::new(RefCell::new(Vec::new()));
    // let mut filtered_values: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

    let mut value_vec_ref = value_vec.clone();
    let ui_weak = ui.as_weak().unwrap();
    ui.on_select_file(move || {
        ui_handles::on_select_file(&ui_weak, &value_vec_ref);
    });

    let ui_weak = ui.as_weak().unwrap();
    let mut filtered_values_ref = filtered_values.clone();
    ui.on_key_press(move |key| {
        let t = key.text.as_str().to_ascii_lowercase();
        if t == "n" && key.modifiers.control {
            ui_handles::filter_next(
                &ui_weak,
                &filtered_values_ref,
                ui_handles::Direction::Forward,
            );
            return;
        }
        if t == "p" && key.modifiers.control {
            ui_handles::filter_next(
                &ui_weak,
                &filtered_values_ref,
                ui_handles::Direction::Backward,
            );
            return;
        }
        if t == "n" {
            ui_weak.invoke_toolbar_toggle();
        }
        if t == "f" && key.modifiers.control {
            ui_weak.invoke_focus_filter();
        }
    });

    let ui_weak = ui.as_weak().unwrap();
    let mut value_vec_ref = value_vec.clone();
    let mut filtered_values_ref = filtered_values.clone();
    ui.on_set_filter(move |filter| {
        ui_handles::on_set_filter(&ui_weak, &value_vec_ref, &filtered_values_ref, &filter);
    });
    let ui_weak = ui.as_weak().unwrap();
    ui.on_check_filter(move |filter| {
        let r = Regex::new(filter.as_str());
        ui_weak.set_is_regex_valid(r.is_ok());
    });

    ui.run()?;

    Ok(())
}
