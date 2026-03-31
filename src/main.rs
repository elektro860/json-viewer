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

mod input_manager;
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
        input_manager::process_input(&key, &ui_weak, &filtered_values_ref);
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

    ui.global::<JsonUtils>()
        .on_is_valid_json_value(|str| json_utils::validate_json_value(str.as_str()));
    ui.global::<JsonUtils>()
        .on_is_valid_json_key(|str| json_utils::valide_json_key(str.as_str()));

    ui.run()?;

    Ok(())
}
