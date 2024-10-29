use core::fmt;

use serde::{
    self, de,
    ser::{SerializeMap, SerializeSeq as _},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Int(i64),
    Uint(u64),
    Float(f32),
    Double(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Value::Nil => serializer.serialize_unit(),
            Value::Boolean(v) => serializer.serialize_bool(v),
            Value::Int(v) => serializer.serialize_i64(v),
            Value::Uint(v) => serializer.serialize_u64(v),
            Value::Float(v) => serializer.serialize_f32(v),
            Value::Double(v) => serializer.serialize_f64(v),
            Value::String(ref v) => serializer.serialize_str(v),
            Value::Binary(ref bytes) => {
                if let Ok(id) = uuid::Uuid::from_slice(bytes) {
                    id.serialize(serializer)
                } else {
                    bytes.serialize(serializer)
                }
            }
            Value::Array(ref xs) => {
                let mut seq = serializer.serialize_seq(Some(xs.len()))?;
                for x in xs {
                    seq.serialize_element(x)?;
                }
                seq.end()
            }
            Value::Map(ref map) => {
                let mut state = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    state.serialize_entry(k, v)?;
                }
                state.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Value;

            #[cold]
            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                write!(fmt, "any valid value")
            }

            #[inline]
            fn visit_some<D>(self, de: D) -> Result<Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(de)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Boolean(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::Uint(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Int(value))
            }

            #[inline]
            fn visit_f32<E>(self, value: f32) -> Result<Value, E> {
                Ok(Value::Float(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::Double(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
            where
                E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }
                Ok(Value::Array(vec))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Binary(v.to_owned()))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Binary(v))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut pairs = vec![];

                while let Some(key) = visitor.next_key()? {
                    let val = visitor.next_value()?;
                    pairs.push((key, val));
                }

                Ok(Value::Map(pairs))
            }
        }

        de.deserialize_any(Visitor)
    }
}
