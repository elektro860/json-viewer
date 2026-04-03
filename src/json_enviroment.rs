use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use ouroboros::self_referencing;
use regex::Regex;
use serde_json::Value;
use slint::{SharedString, ToSharedString};

use crate::{
    json_utils::{self, json_value::ToUI, validate_json_value},
    ui_handles::SetErrors,
    JsonValue,
};

#[self_referencing]
#[derive(Debug)]
pub(crate) struct JsonEnviroment {
    path: PathBuf,
    json: Value,
    #[borrows(mut json)]
    #[covariant]
    pub entries: Vec<Entry<'this>>,
    pub filtered_indicies: Vec<usize>,
}
#[derive(Debug)]
pub enum ValueReference<'a> {
    Array,
    Object,
    Primitive(&'a mut Value),
}
#[derive(Debug)]
pub(crate) struct Entry<'a> {
    pub render: JsonValue,
    json: ValueReference<'a>,
}

impl JsonEnviroment {
    pub fn from_json(json: Value, path: PathBuf) -> JsonEnviroment {
        JsonEnviroment::new(
            path,
            json,
            |json_ref| {
                let start = Instant::now();
                let mut ret = Vec::with_capacity(json_utils::get_value_count(json_ref) + 100);
                Self::populate_recursive(&mut ret, json_ref, "", 0);
                println!("Initialized enviroment in {:?}", start.elapsed());

                ret
            },
            Vec::new(),
        )
    }
    pub fn path(&self) -> &Path {
        self.borrow_path().as_path()
    }
    pub fn set_filter(&mut self, regex: &Regex) {
        self.with_mut(|fields| {
            fields
                .entries
                .iter_mut()
                .for_each(|v| v.render.in_filter = false);
            let filtered = json_utils::filter_json(regex, fields.entries.as_slice());
            filtered
                .iter()
                .for_each(|i| fields.entries[*i].render.in_filter = true);
            *fields.filtered_indicies = filtered;
        })
    }
    fn populate_recursive<'this>(
        vec: &mut Vec<Entry<'this>>,
        value_ref: &'this mut Value,
        name: &str,
        level: i32,
    ) {
        let value = value_ref.to_ui(name.into(), vec.len() as i32, level);

        let json = match value_ref {
            Value::Array(a) => {
                a.iter_mut()
                    .for_each(|v| Self::populate_recursive(vec, v, "", level + 1));
                ValueReference::Array
            }
            Value::Object(a) => {
                a.iter_mut()
                    .for_each(|(key, v)| Self::populate_recursive(vec, v, key.as_str(), level + 1));
                ValueReference::Object
            }
            _ => ValueReference::Primitive(value_ref),
        };
        vec.push(Entry {
            render: value,
            json: json,
        });
    }
    pub fn set_value(&mut self, id: usize, str: &str) -> Result<(), SetErrors> {
        self.with_entries_mut(|ent| {
            let mut value = ent.get_mut(id).ok_or(SetErrors::InvalidIndex)?;

            let v = match validate_json_value(str) {
                Some(v) => v,
                None => {
                    if let ValueReference::Primitive(p) = &value.json {
                        dbg!("Setting value to {}", p.to_shared_string());
                        value.render.value = p.to_shared_string();
                    }
                    return Err(SetErrors::InvalidJson);
                }
            };
            match &mut value.json {
                ValueReference::Primitive(json) => {
                    value.render.value = SharedString::from(str);
                    value.render.value_type = v.value_type();
                    **json = v;
                }
                _ => todo!(),
            }
            Ok(())
        })?;
        Ok(())
    }
    pub fn set_key(&mut self, id: usize, str: &str) -> Result<(), SetErrors> {
        self.with_entries_mut(|ent| {
            let mut entry = ent.get_mut(id).ok_or(SetErrors::InvalidIndex)?;
            let v = validate_json_value(str).ok_or(SetErrors::InvalidJson)?;
            match v {
                Value::String(s) => {
                    entry.render.name = SharedString::from(str);
                }
                _ => return Err(SetErrors::InvalidKey),
            }
            Ok(())
        })
    }
}
