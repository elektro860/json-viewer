// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, path::Path, sync::Mutex};

use slint::ComponentHandle;

slint::include_modules!();

static SELECTED_FILE: Mutex<Option<Box<Path>>> = Mutex::new(None);

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.on_select_file(|| {
        println!("Selecting file");
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
