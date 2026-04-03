use std::cell::RefCell;
use std::error::Error;
use std::mem::swap;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Instant;
use std::{env, rc::Weak};

use regex::Regex;
use rfd::FileDialog;
use serde_json::{from_str, Value};
use slint::{ComponentHandle, SharedString};

use crate::json_enviroment::JsonEnviroment;
use crate::json_utils::json_value::ToUI;
use crate::json_utils::validate_json_value;
use crate::slint_generatedAppWindow::AppWindow;
use crate::{json_utils, ui_handles, unwrap_result};
use crate::{unwrap_option, JsonValue};

pub fn on_select_file(ui_weak: &AppWindow) {
    let handle = ui_weak.window().window_handle();
    let file = FileDialog::new()
        .set_parent(&handle)
        .set_directory(env::current_dir().unwrap())
        .add_filter("JSON", &["json"])
        .set_title("Select JSON file")
        .pick_file();
    if let Some(file) = file {
        println!("Selected: {:?}", file);
        match json_utils::read_file(file) {
            Err(e) => eprintln!("{e}"),
            Ok(env) => {
                let values = env.borrow_entries();

                json_utils::set_json_values(&ui_weak, values);
                let mut opt = json_utils::CURRENT_JSON.lock().unwrap();
                *opt = Some(env);
            }
        }
    }
}
pub fn on_set_filter(ui: &AppWindow, filter: &str) {
    let regex = Regex::new(filter);

    if let Ok(regex) = regex {
        let mut mutex = json_utils::CURRENT_JSON.lock().unwrap();
        let env = unwrap_option!(mutex.as_mut());
        let start = Instant::now();

        env.set_filter(&regex);
        println!("Filtered in {:?}", start.elapsed());
        let filtered = env.borrow_filtered_indicies();

        json_utils::set_json_values(ui, env.borrow_entries());
        if let Some(index) = filtered.iter().next() {
            ui.invoke_move_to(*index as i32);
        }
    } else {
        eprintln!("Invalid regex");
    }
}
#[derive(Debug)]
pub enum Direction {
    Forward,
    Backward,
}
pub fn filter_next(ui: &AppWindow, dir: Direction) {
    let mut prev = None;
    let mut next = None;

    let opt = json_utils::CURRENT_JSON.lock().unwrap();
    let enviroment = unwrap_option!(opt.deref());
    let filter = &enviroment.borrow_filtered_indicies();

    let current_index = ui.get_current_value() as usize;
    let last_p = filter.iter().position(|index| *index > current_index);
    match last_p {
        None => {
            prev = filter.iter().nth_back(2);
        }
        Some(id) => {
            next = filter.get(id);
            prev = filter.get(id.saturating_sub(2));
        }
    }

    let pos = match dir {
        Direction::Forward => next,
        Direction::Backward => prev,
    };
    if let Some(pos) = pos {
        ui.invoke_move_to(*pos as i32);
    }
}

#[derive(Debug)]
pub(crate) enum SetErrors {
    InvalidIndex,
    InvalidJson,
    InvalidKey,
}
pub fn set_value(ui: &AppWindow, id: i32, str: &str) -> Result<(), SetErrors> {
    let mut mutex = json_utils::CURRENT_JSON.lock();
    let env = unwrap_result!(mutex.as_deref_mut().unwrap());

    let result = env.set_value(id as usize, str);
    json_utils::set_json_values(ui, env.borrow_entries());
    result?;

    Ok(())
}
pub fn set_key(ui: &AppWindow, id: i32, str: &str) -> Result<(), SetErrors> {
    let mut mutex = json_utils::CURRENT_JSON.lock();
    let env = unwrap_result!(mutex.as_deref_mut().unwrap());

    let result = env.set_key(id as usize, str);
    json_utils::set_json_values(ui, env.borrow_entries());
    result?;

    Ok(())
}
