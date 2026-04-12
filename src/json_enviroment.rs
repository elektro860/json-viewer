use std::{
    marker::PhantomData,
    mem::swap,
    ops::ControlFlow,
    path::{Path, PathBuf},
    time::Instant,
};

use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator,
};
use regex::Regex;
use serde_json::{map::Keys, Number, Value};
use slint::{SharedString, ToSharedString};

use crate::{
    json_utils::{self, json_value::ToUI, validate_json_value},
    ui_handles::SetErrors,
    ValueType,
};

#[derive(Debug)]
pub struct EntryId(pub usize);
#[derive(Debug)]
pub struct JsonEnviroment {
    path: PathBuf,
    pub entries: Vec<Entry>,
    pub filtered_indicies: Vec<EntryId>,
}
#[derive(Debug, derive_getters::Getters)]
pub struct Entry {
    parent: usize,
    index: JsonIndex,
    value: JsonValue,
    level: usize,
}

impl ToUI for Entry {
    fn to_ui(&self, id: i32) -> crate::JsonValue {
        let value = match self.value() {
            JsonValue::Value(value) => value.to_shared_string(),
            JsonValue::Array(_) => SharedString::from("["),
            JsonValue::Object(_) => SharedString::from("{"),
        };
        let value_type = self.value_type();
        let mut has_key = false;
        let name = match self.index() {
            JsonIndex::Root => "".into(),
            JsonIndex::Index(index) => format!("{index}").into(),
            JsonIndex::Key(key) => {
                has_key = true;
                format!("\"{key}\"").into()
            }
        };

        crate::JsonValue {
            level: *self.level() as i32,
            name,
            value,
            value_type,
            id,
            in_filter: false,
            has_key,
        }
    }
    fn value_type(&self) -> ValueType {
        match self.value() {
            JsonValue::Array(_) => ValueType::Array,
            JsonValue::Object(_) => ValueType::Object,
            JsonValue::Value(v) => match v {
                Value::Null => ValueType::Null,
                Value::Number(_) => ValueType::Number,
                Value::Bool(_) => ValueType::Bool,
                Value::Array(_) => ValueType::Array,
                Value::Object(_) => ValueType::Object,
                Value::String(_) => ValueType::String,
            },
        }
    }
}

#[derive(Debug)]
pub(crate) enum JsonIndex {
    Root,
    Index(usize),
    Key(String),
}
#[derive(Debug)]
pub(crate) enum JsonValue {
    Value(Value),
    Array(List),
    Object(List),
}
#[derive(Debug, Default)]
pub(crate) struct List {
    pub size: usize,
    pub is_folded: bool,
}
impl List {
    pub fn new(size: usize) -> Self {
        List {
            size,
            is_folded: true,
        }
    }
}

impl JsonEnviroment {
    pub fn to_ui(&self) -> Vec<crate::JsonValue> {
        let start = Instant::now();
        let mut level: usize = 1;
        let mut rendered_values = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                match &v.value {
                    JsonValue::Value(_) => {
                        if v.level > level {
                            return None;
                        }
                    }
                    JsonValue::Object(list) => {
                        if list.is_folded {
                            level = v.level;
                        } else {
                            level = v.level + 1;
                        }
                    }
                    JsonValue::Array(list) => {
                        if list.is_folded {
                            level = v.level;
                        } else {
                            level = v.level + 1;
                        }
                    }
                }

                Some((i, v))
            })
            .collect::<Vec<_>>();

        let mut values = Vec::with_capacity(rendered_values.len());
        rendered_values
            .par_iter()
            .map(|(index, entry)| entry.to_ui(*index as i32))
            .collect_into_vec(&mut values);

        println!(
            "Converted ({}) to UI in {:?}",
            values.len(),
            start.elapsed()
        );

        let mut iter = self.filtered_indicies.iter();
        if let Some(value) = iter.next() {
            let mut value = value.0 as i32;
            for v in values.iter_mut() {
                if value != v.id {
                    continue;
                }

                v.in_filter = true;
                if let Some(v) = iter.next() {
                    value = v.0 as i32;
                } else {
                    break;
                }
            }
        }

        values
    }
    pub fn from_json(json: Value, path: PathBuf) -> JsonEnviroment {
        let entries = {
            let start = Instant::now();
            let mut ret = Vec::with_capacity(json_utils::get_value_count(&json) + 100);
            Self::populate_recursive(&mut ret, json, JsonIndex::Root, 0, 0);
            println!("Initialized enviroment in {:?}", start.elapsed());

            ret
        };
        JsonEnviroment {
            path,
            entries,
            filtered_indicies: Vec::new(),
        }
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn set_filter(&mut self, regex: &Regex) {
        self.filtered_indicies = self
            .entries
            .par_iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if let JsonIndex::Key(key) = &entry.index {
                    if regex.is_match(key) {
                        return Some(EntryId(index));
                    }
                }
                if let JsonValue::Value(value) = &entry.value {
                    let matches = match value {
                        // Use the internal reference directly! No allocation.
                        serde_json::Value::String(s) => regex.is_match(s),
                        // For non-strings, either skip or use a small stack buffer
                        serde_json::Value::Number(n) => regex.is_match(&n.to_string()),
                        serde_json::Value::Bool(b) => {
                            regex.is_match(if *b { "true" } else { "false" })
                        }
                        _ => false,
                    };

                    if matches {
                        return Some(EntryId(index));
                    }
                }

                None
            })
            .collect();
    }
    fn populate_recursive<'this>(
        vec: &mut Vec<Entry>,
        json_value: Value,
        index: JsonIndex,
        level: usize,
        parent: usize,
    ) {
        let id = vec.len();

        match json_value {
            Value::Array(v) => {
                vec.push(Entry {
                    index,
                    value: JsonValue::Array(List::new(v.len())),
                    parent,
                    level,
                });
                v.into_iter().enumerate().for_each(|(index, value)| {
                    Self::populate_recursive(vec, value, JsonIndex::Index(index), level + 1, id);
                });
            }
            Value::Object(v) => {
                vec.push(Entry {
                    index,
                    value: JsonValue::Object(List::new(v.len())),
                    parent,
                    level,
                });
                v.into_iter().for_each(|(index, value)| {
                    Self::populate_recursive(vec, value, JsonIndex::Key(index), level + 1, id);
                });
            }
            _ => {
                vec.push(Entry {
                    index,
                    value: JsonValue::Value(json_value),
                    parent,
                    level,
                });
            }
        };
    }
    pub fn set_value(&mut self, id: usize, str: &str) -> Result<(), SetErrors> {
        let v = validate_json_value(str).ok_or(SetErrors::InvalidJson)?;
        let v = match v {
            Value::Array(_) | Value::Object(_) => Err(SetErrors::InvalidJson),
            _ => Ok(JsonValue::Value(v)),
        }?;

        let mut value = self.entries.get_mut(id).ok_or(SetErrors::InvalidIndex)?;
        value.value = v;

        Ok(())
    }
    pub fn set_key(&mut self, id: usize, str: &str) -> Result<(), SetErrors> {
        let v = validate_json_value(str).ok_or(SetErrors::InvalidJson)?;
        let v = match v {
            Value::String(s) => Ok(s),
            _ => Err(SetErrors::InvalidKey),
        }?;

        let mut value = self.entries.get_mut(id).ok_or(SetErrors::InvalidIndex)?;
        value.index = JsonIndex::Key(v);

        Ok(())
    }
}
