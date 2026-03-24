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

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let mut value_vec: Rc<RefCell<Vec<JsonValue>>> = Rc::new(RefCell::new(Vec::new()));
    let mut filtered_values: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

    let ui_weak = ui.as_weak().unwrap();

    let mut value_vec_ref = value_vec.clone();
    ui.on_select_file(move || {
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
                Ok(_) => {
                    let j = json_utils::CURRENT_JSON.lock().unwrap();
                    let json = j.as_ref().unwrap();
                    let value_count = json_utils::get_value_count(json.json());
                    let mut value_vec = value_vec_ref.borrow_mut();

                    value_vec.clear();
                    value_vec.reserve(value_count + 100); // extra in case of addition
                    json_utils::populate_vector(&mut value_vec, json.json(), "", 0);

                    // println!("{:?}", value_vec);
                    println!(
                        "Total values: {value_count}\nTotal entries: {}",
                        value_vec.len()
                    );

                    json_utils::set_json_values(&ui_weak, &value_vec);
                }
            }
        }
    });

    let ui_weak = ui.as_weak().unwrap();
    ui.on_key_press(move |key| {
        let t = key.text.as_str().to_ascii_lowercase();
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
        let values = value_vec_ref.borrow();
        let regex = Regex::new(&filter);
        if let Ok(regex) = regex {
            let mut filtered = json_utils::filter_json(&regex, values.as_slice());
            let mut values = filtered_values_ref.borrow_mut();
            values.clear();
            values.reserve(filtered.len());
            filtered.iter().for_each(|v| values.push(v.id));
        } else {
            eprintln!("Invalid regex");
        }
    });
    let ui_weak = ui.as_weak().unwrap();
    ui.on_check_filter(move |filter| {
        let r = Regex::new(filter.as_str());
        ui_weak.set_is_regex_valid(r.is_ok());
    });

    ui.run()?;

    Ok(())
}
