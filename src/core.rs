use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Key(Cow<'static, str>);

impl Key {
    pub fn new<S: Into<Cow<'static, str>>>(value: S) -> Self {
        Key(value.into())
    }

    pub fn bool(&self, value: bool) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::Bool(value),
        }
    }

    pub fn i64(&self, value: i64) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::I64(value),
        }
    }

    pub fn u64(&self, value: u64) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::U64(value),
        }
    }

    pub fn f64(&self, value: f64) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::F64(value),
        }
    }

    pub fn string<S: Into<String>>(&self, value: S) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::String(value.into()),
        }
    }

    pub fn bytes(&self, value: Vec<u8>) -> KeyValue {
        KeyValue {
            key: self.clone(),
            value: Value::Bytes(value),
        }
    }

    pub fn inner(&self) -> &Cow<'static, str> {
        &self.0
    }

    pub fn into_inner(self) -> Cow<'static, str> {
        self.0
    }
}

impl Into<Cow<'static, str>> for Key {
    fn into(self) -> Cow<'static, str> {
        self.0
    }
}

impl Into<String> for Key {
    fn into(self) -> String {
        self.0.to_string()
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Bool(value) => value.to_string(),
            Value::I64(value) => value.to_string(),
            Value::U64(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Bytes(value) => {
                String::from_utf8(value.clone()).unwrap_or_else(|_| String::new())
            }
        }
    }
}

impl Into<Cow<'static, str>> for Value {
    fn into(self) -> Cow<'static, str> {
        self.to_string().into()
    }
}

#[derive(Clone, Debug)]
pub struct KeyValue {
    pub key: Key,
    pub value: Value,
}

#[derive(Default)]
pub struct Unit(String);

impl Unit {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Unit(value.into())
    }
}
