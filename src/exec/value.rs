use lazy_static::lazy_static;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ValueTable {
    values: HashMap<Uuid, Value>,
}

#[derive(Clone, Debug)]
pub struct Value {
    pub content: ValueVariant,
    pub uuid: Uuid,
}

impl Value {
    pub fn new(content: ValueVariant) -> Value {
        Value {
            content,
            uuid: Uuid::new_v4(),
        }
    }
}

lazy_static! {
    pub static ref NO_VALUE: Value = Value { content: ValueVariant::Nothing, uuid: Uuid::new_v4() };
}

#[derive(Clone, Debug)]
pub enum ValueVariant {
    Nothing,
    Primitive(PrimitiveValue),
}

#[derive(Clone, Debug)]
pub enum PrimitiveValue {
    Int(i64),
    Dec(f64),
    Text(String),
    Bool(bool),
}

impl ValueTable {
    pub fn new() -> ValueTable {
        ValueTable {
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: Value) {
        self.values.insert(value.uuid, value);
    }

    pub fn get(&self, uuid: Uuid) -> Option<&Value> {
        self.values.get(&uuid)
    }
}