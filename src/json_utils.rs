use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde_json::Value;
use slint::{SharedString, ToSharedString};

use crate::JsonValue;

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

#[derive(Debug)]
pub struct Entry {
    json: JsonValue,
}
pub fn populate_vector<'a>(vec: &'a mut Vec<Entry>, value: &'a Value, name: &str, level: i32) {
    let v = match value {
        Value::Array(_) => SharedString::from("["),
        Value::Object(_) => SharedString::from("{"),
        _ => value.to_shared_string(),
    };
    vec.push(Entry {
        json: JsonValue {
            level: level.into(),
            name: name.into(),
            value: v,
        },
    });
    match value {
        Value::Array(a) => a
            .iter()
            .for_each(|v| populate_vector(vec, v, "", level + 1)),
        Value::Object(a) => a
            .iter()
            .for_each(|(key, v)| populate_vector(vec, v, key.as_str(), level + 1)),
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
