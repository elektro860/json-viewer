use std::env;
use std::fs::File;
use std::ops::Deref;
use std::time::Instant;

use regex::Regex;
use rfd::FileDialog;
use slint::ComponentHandle;

use crate::slint_generatedAppWindow::AppWindow;
use crate::unwrap_option;
use crate::{json_utils, unwrap_result};

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
                json_utils::render_values(ui_weak);
            }
        }
    }
}
pub fn on_save(ui_weak: &AppWindow) {
    let handle = ui_weak.window().window_handle();
    let (json, path) = {
        let lock = json_utils::CURRENT_JSON.lock().unwrap();
        let r = unwrap_option!(lock.as_ref());
        (unwrap_option!(r.to_json()), r.path().to_owned())
    };
    let path = unwrap_option!(path.parent());

    let dialog = FileDialog::new()
        .set_parent(&handle)
        .set_title("Save json")
        .add_filter("JSON", &["json"])
        .set_directory(path)
        .save_file();
    let dialog = unwrap_option!(dialog);

    let writer = match File::create(dialog) {
        Ok(v) => v,
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("Error while creating file")
                .set_buttons(rfd::MessageButtons::Ok)
                .set_description(e.to_string())
                .show();
            return;
        }
    };

    if let Err(e) = serde_json::to_writer_pretty(writer, &json) {
        rfd::MessageDialog::new()
            .set_title("Error while writing to file")
            .set_buttons(rfd::MessageButtons::Ok)
            .set_description(e.to_string())
            .show();
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
            env.filtered_indicies.first().map(|x| x.0 as i32)
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
    let opt = json_utils::CURRENT_JSON.lock().unwrap();
    let enviroment = unwrap_option!(opt.deref());
    let filter = &enviroment.filtered_indicies;

    let current_index = &enviroment.entries_rendered[ui.get_current_value() as usize];

    let last_p = filter.iter().position(|index| index.0 > current_index.0);
    let pos = match last_p {
        None => filter.iter().nth_back(2),
        Some(id) => match dir {
            Direction::Forward => filter.get(id),
            Direction::Backward => filter.get(id.saturating_sub(2)),
        },
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
pub fn toggle_key(ui: &AppWindow, id: i32) {
    let result = {
        let mut lock = json_utils::CURRENT_JSON.lock().unwrap();
        let env = unwrap_option!(lock.as_mut());
        let id = unwrap_option!(env.get_id(id as usize));
        env.toggle_fold(&id)
    };

    if let Ok(_r) = result {
        json_utils::render_values(ui);
    }
}
