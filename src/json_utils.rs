use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
    mem::take,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Mutex,
    time::Instant,
};

use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};
use regex::Regex;
use serde_json::{from_str, Value};
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};

use crate::{
    json_enviroment::{Entry, JsonEnviroment},
    json_utils::json_value::ToUI,
    AppWindow, JsonValue, ValueType,
};
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
                .for_each(|(key, v)| counter += get_value_count(v));
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
        Err(e) => false,
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

pub fn set_json_values(ui: &AppWindow, values: &Vec<Entry>) {
    let clone = {
        let mut ret = Vec::with_capacity(values.len());
        values.iter().for_each(|v| ret.push(v.render.clone()));
        ret
    };
    let model = ModelRc::new(Rc::new(VecModel::from(clone)));
    ui.set_json_values(model);
}
pub fn filter_json<'a>(regex: &Regex, values: &[Entry<'a>]) -> Vec<usize> {
    values
        .par_iter()
        .enumerate()
        .filter_map(|(i, v)| {
            if regex.is_match(v.render.name.as_str()) {
                return Some(i);
            }
            if regex.is_match(v.render.value.as_str()) {
                return Some(i);
            }
            None
        })
        .collect()
}
