use crate::JsonValue;
use crate::ValueType;
use serde_json::Value;
use slint::SharedString;
use slint::ToSharedString;

impl ToUI for Value {
    fn to_ui(self: &Self, name: SharedString, id: i32, level: i32) -> JsonValue {
        let value = match self {
            Value::Array(_) => SharedString::from("["),
            Value::Object(_) => SharedString::from("{"),
            _ => self.to_shared_string(),
        };
        let value_type = self.value_type();
        let name = match name.len() {
            0 => "".into(),
            _ => format!("\"{}\"", name).into(),
        };

        JsonValue {
            level,
            name,
            value,
            value_type,
            id,
            in_filter: false,
        }
    }
    fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Number(_) => ValueType::Number,
            Value::Bool(_) => ValueType::Bool,
            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
            Value::String(_) => ValueType::String,
        }
    }
}
pub trait ToUI {
    fn to_ui(&self, name: SharedString, id: i32, level: i32) -> JsonValue;
    fn value_type(&self) -> ValueType;
}
