use crate::JsonValue;
use crate::ValueType;

pub trait ToUI {
    fn to_ui(&self, id: i32) -> JsonValue;
    fn value_type(&self) -> ValueType;
}
