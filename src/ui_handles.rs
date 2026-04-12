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
                {
                    let mut opt = json_utils::CURRENT_JSON.lock().unwrap();
                    *opt = Some(env);
                }
                json_utils::render_values(&ui_weak);
            }
        }
    }
}
pub fn on_set_filter(ui: &AppWindow, filter: &str) {
    let regex = Regex::new(filter);

    if let Ok(regex) = regex {
        let start = Instant::now();
        let first = {
            let mut mutex = json_utils::CURRENT_JSON.lock().unwrap();
            let env = unwrap_option!(mutex.as_mut());

            env.set_filter(&regex);
            env.filtered_indicies.get(0).map(|x| x.0 as i32)
        };
        println!("Filtered in {:?}", start.elapsed());

        json_utils::render_values(ui);
        if let Some(index) = first {
            ui.invoke_move_to(index);
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
    let filter = &enviroment.filtered_indicies;

    let current_index = &enviroment.entries_rendered[ui.get_current_value() as usize];

    let last_p = filter.iter().position(|index| index.0 > current_index.0);
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
        let p = enviroment
            .entries_rendered
            .iter()
            .position(|v| v.0 == pos.0)
            .unwrap_or(0);
        ui.invoke_move_to(p as i32);
    }
}

#[derive(Debug)]
pub(crate) enum SetErrors {
    InvalidIndex,
    InvalidJson,
    InvalidKey,
}
pub fn set_value(ui: &AppWindow, id: i32, str: &str) -> Result<(), SetErrors> {
    {
        let mut mutex = json_utils::CURRENT_JSON.lock();
        let env = unwrap_result!(mutex.as_deref_mut().unwrap());
        env.set_value(id as usize, str)?;
    }

    json_utils::render_values(ui);

    Ok(())
}
pub fn set_key(ui: &AppWindow, id: i32, str: &str) -> Result<(), SetErrors> {
    {
        let mut mutex = json_utils::CURRENT_JSON.lock();
        let env = unwrap_result!(mutex.as_deref_mut().unwrap());
        env.set_key(id as usize, str)?;
    }

    json_utils::render_values(ui);

    Ok(())
}
