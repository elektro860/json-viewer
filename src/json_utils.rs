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
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};
use regex::Regex;
use serde_json::Value;
use slint::{ModelRc, SharedString, ToSharedString, VecModel};

use crate::{json_utils::json_value::ToUI, AppWindow, JsonValue, ValueType};
mod json_value;

#[derive(Debug)]
pub struct CurrentJSON {
    path: PathBuf,
    json: Value,
}
impl CurrentJSON {
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn json(&self) -> &Value {
        &self.json
    }
}
pub static CURRENT_JSON: Mutex<Option<CurrentJSON>> = Mutex::new(None);

pub fn populate_vector<'a>(
    json_values: &'a mut Vec<JsonValue>,
    value: &'a Value,
    name: &str,
    level: i32,
) {
    json_values.push(value.to_ui(name.into(), json_values.len() as i32, level));

    match value {
        Value::Array(a) => a
            .iter()
            .for_each(|v| populate_vector(json_values, v, "", level + 1)),
        Value::Object(a) => a
            .iter()
            .for_each(|(key, v)| populate_vector(json_values, v, key.as_str(), level + 1)),
        _ => {}
    }
}
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

pub fn read_file(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let v = parse_file(path.as_path())?;
    let mut c_json = CURRENT_JSON.lock()?;
    *c_json = Some(CurrentJSON { path, json: v });

    Ok(())
}

fn parse_file(path: &Path) -> Result<Value, Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let json: Value = serde_json::from_reader(reader)?;

    Ok(json)
}

pub fn set_json_values(ui: &AppWindow, values: &Vec<JsonValue>) {
    let clone = values.clone();
    let model = ModelRc::new(Rc::new(VecModel::from(clone)));
    ui.set_json_values(model);
}
pub fn filter_json<'a>(regex: &Regex, values: &'a [JsonValue]) -> Vec<&'a JsonValue> {
    values
        .par_chunks(500)
        .flat_map_iter(|chunk| {
            chunk.iter().filter_map(|v| {
                if regex.is_match(v.name.as_str()) {
                    return Some(v);
                }
                if regex.is_match(v.value.as_str()) {
                    return Some(v);
                }
                None
            })
        })
        .collect()
}
