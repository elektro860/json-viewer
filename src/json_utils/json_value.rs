use crate::json_enviroment::Entry;
use crate::JsonValue;
use crate::ValueType;
use serde_json::Value;
use slint::SharedString;
use slint::ToSharedString;

pub trait ToUI {
    fn to_ui(&self, id: i32) -> JsonValue;
    fn value_type(&self) -> ValueType;
}
