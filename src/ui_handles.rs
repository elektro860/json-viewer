use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use std::{env, rc::Weak};

use regex::Regex;
use rfd::FileDialog;
use slint::ComponentHandle;

use crate::slint_generatedAppWindow::AppWindow;
use crate::JsonValue;
use crate::{json_utils, ui_handles};

pub fn on_select_file(ui_weak: &AppWindow, value_vec: &Rc<RefCell<Vec<JsonValue>>>) {
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
                let mut value_vec = value_vec.borrow_mut();

                value_vec.clear();
                value_vec.reserve_exact(value_count + 100); // extra in case of addition
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
}
pub fn on_set_filter(
    ui: &AppWindow,
    value_vec: &Rc<RefCell<Vec<JsonValue>>>,
    fitered_values: &Rc<RefCell<Vec<i32>>>,
    filter: &str,
) {
    let regex = Regex::new(filter);
    if let Ok(regex) = regex {
        let mut values = value_vec.borrow_mut();
        let start = Instant::now();
        let mut filtered = json_utils::filter_json(&regex, values.as_slice());
        println!("Filtered in {:?}", start.elapsed());
        let mut filtered_ids = fitered_values.borrow_mut();

        filtered_ids.clear();
        filtered_ids.reserve(filtered.len());
        filtered.iter().for_each(|v| filtered_ids.push(v.id));

        let mut iter = filtered_ids.iter();
        let end = values.len() as i32;
        let mut next = *(iter.next().unwrap_or(&end)) as usize;

        values.iter_mut().enumerate().for_each(|(id, v)| {
            if id >= next {
                next = *(iter.next().unwrap_or(&end)) as usize;
                v.in_filter = true;
            } else {
                v.in_filter = false;
            }
        });
        json_utils::set_json_values(ui, &values);
        if let Some(id) = filtered_ids.iter().next() {
            ui.invoke_move_to(*id);
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
pub fn filter_next(ui: &AppWindow, filtered_values: &Rc<RefCell<Vec<i32>>>, dir: Direction) {
    let mut prev: Option<&i32> = None;
    let mut next: Option<&i32> = None;

    let filter = filtered_values.borrow();
    let current_index = ui.get_current_value();
    let last_p = filter.iter().position(|v| *v > current_index);
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
        ui.invoke_move_to(*pos);
    }
}
