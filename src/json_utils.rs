use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Mutex,
    time::Instant,
};

use serde_json::{from_str, Value};
use slint::{ModelRc, VecModel};

use crate::{json_enviroment::JsonEnviroment, unwrap_option, AppWindow};
pub mod json_value;

pub static CURRENT_JSON: Mutex<Option<JsonEnviroment>> = Mutex::new(None);

pub fn get_value_count(json: &Value) -> usize {
    let mut counter = 1;

    match json {
        Value::Array(arr) => {
            arr.iter().for_each(|v| counter += get_value_count(v));
        }
        Value::Object(map) => {
            map.iter()
                .for_each(|(_key, v)| counter += get_value_count(v));
        }
        _ => {}
    }

    counter
}
pub fn validate_json_value(json_text: &str) -> Option<Value> {
    from_str::<Value>(json_text.trim()).ok()
}
pub fn valide_json_key(json_text: &str) -> bool {
    let str = json_text.trim();
    if str.is_empty() {
        return true;
    }
    let result = from_str::<Value>(str);
    match result {
        Ok(v) => v.is_string(),
        Err(_e) => false,
    }
}

pub fn read_file(path: PathBuf) -> Result<JsonEnviroment, Box<dyn Error>> {
    let v = parse_file(path.as_path())?;
    Ok(JsonEnviroment::from_json(v, path))
}

fn parse_file(path: &Path) -> Result<Value, Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let json: Value = serde_json::from_reader(reader)?;

    Ok(json)
}

pub fn render_values(ui: &AppWindow) {
    let start = Instant::now();
    let json_ui = {
        let mut lock = CURRENT_JSON.lock().unwrap();
        let enviroment = unwrap_option!(lock.as_mut());
        enviroment.to_ui()
    };

    let model = ModelRc::new(Rc::new(VecModel::from(json_ui)));
    ui.set_json_values(model);

    println!("Updated values in {:?}", start.elapsed());
}
