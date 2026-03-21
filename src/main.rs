// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    error::Error,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Mutex,
};

use rfd::FileDialog;
use slint::{ComponentHandle, ModelRc, VecModel};

slint::include_modules!();

mod json_parser;

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let ui_weak = ui.as_weak().unwrap();
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
            match json_parser::read_file(file) {
                Err(e) => eprintln!("{e}"),
                Ok(_) => {}
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

    ui.run()?;

    Ok(())
}
