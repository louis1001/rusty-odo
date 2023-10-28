use uuid::Uuid;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crate::native::function::NativeFn;

#[derive(Debug)]
pub struct ValueTable<'a> {
    values: HashMap<Uuid, Value<'a>>,
}

#[derive(Clone, Debug)]
pub struct Value<'a> {
    pub content: ValueVariant<'a>,
    pub uuid: Uuid,
}

impl<'a> Value<'a> {
    pub fn new(content: ValueVariant<'a>) -> Value<'a> {
        Value {
            content,
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ValueVariant<'a> {
    Nothing,
    Primitive(PrimitiveValue),
    Function(FunctionValue<'a>)
}

#[derive(Clone, Debug)]
pub enum PrimitiveValue {
    Int(i64),
    Dec(f64),
    Text(String),
    Bool(bool),
}

#[derive(Clone)]
pub enum FunctionValue<'a> {
    Native(Arc<NativeFn<'a>>),
}

impl<'a> Debug for FunctionValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionValue::Native(_) => write!(f, "FunctionValue::Native(<native code>)"),
        }
    }
}

impl<'a> ValueTable<'a> {
    pub fn new() -> ValueTable<'a> {
        ValueTable {
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: Value<'a>) {
        self.values.insert(value.uuid, value);
    }

    pub fn get(&self, uuid: Uuid) -> Option<&Value<'a>> {
        self.values.get(&uuid)
    }
}