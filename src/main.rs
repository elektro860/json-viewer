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

use crate::json_enviroment::JsonEnviroment;

slint::include_modules!();

mod input_manager;
mod json_enviroment;
mod json_utils;
mod ui_handles;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let ui_weak = ui.as_weak().unwrap();
    ui.on_select_file(move || {
        ui_handles::on_select_file(&ui_weak);
    });

    let ui_weak = ui.as_weak().unwrap();
    ui.on_key_press(move |key| {
        input_manager::process_input(&key, &ui_weak);
    });

    let ui_weak = ui.as_weak().unwrap();
    ui.on_set_filter(move |filter| {
        ui_handles::on_set_filter(&ui_weak, &filter);
    });
    let ui_weak = ui.as_weak().unwrap();
    ui.on_check_filter(move |filter| {
        let r = Regex::new(filter.as_str());
        ui_weak.set_is_regex_valid(r.is_ok());
    });

    let json_utils = ui.global::<JsonUtils>();
    json_utils
        .on_is_valid_json_value(|str| json_utils::validate_json_value(str.as_str()).is_some());
    json_utils.on_is_valid_json_key(|str| json_utils::valide_json_key(str.as_str()));
    let ui_weak = ui.as_weak().unwrap();
    json_utils.on_set_value(move |id, str| ui_handles::set_value(&ui_weak, id, &str).is_ok());
    let ui_weak = ui.as_weak().unwrap();
    json_utils.on_set_key(move |id, str| ui_handles::set_key(&ui_weak, id, &str).is_ok());
    let ui_weak = ui.as_weak().unwrap();
    json_utils.on_toggle_fold(move |id| ui_handles::toggle_key(&ui_weak, id));
    let ui_weak = ui.as_weak().unwrap();
    json_utils.on_save(move || {
        ui_handles::on_save(&ui_weak);
    });

    ui.run()?;

    Ok(())
}
