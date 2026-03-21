use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde_json::Value;

#[derive(Debug)]
pub struct CurrentJSON {
    path: PathBuf,
    json: Value,
}
pub static CURRENT_JSON: Mutex<Option<CurrentJSON>> = Mutex::new(None);

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
