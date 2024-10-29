use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use scylla::frame::response::result::CqlValue;
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Serialize as _,
};

use crate::{value::Value, SerializableCqlValue, SerializableCqlValueRef};

impl serde::Serialize for SerializableCqlValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableCqlValueRef(self.0.as_ref()).serialize(serializer)
    }
}

impl serde::Serialize for SerializableCqlValueRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self.0 {
            Some(value) => value,
            None => return serializer.serialize_none(),
        };

        match value {
            CqlValue::Text(s) | CqlValue::Ascii(s) => dwim_str(serializer, s),
            CqlValue::Blob(b) => dwim_bytes(serializer, b),
            CqlValue::Boolean(b) => serializer.serialize_bool(*b),
            CqlValue::Counter(c) => serializer.serialize_i64(c.0),
            CqlValue::Decimal(d) => BigDecimal::from(d.clone()).serialize(serializer),
            CqlValue::Date(d) => {
                let date: chrono::NaiveDate = (*d).try_into().unwrap();
                date.serialize(serializer)
            }
            CqlValue::Double(d) => serializer.serialize_f64(*d),
            CqlValue::Duration(_) => todo!("Duration"),
            CqlValue::Empty => serializer.serialize_unit(),
            CqlValue::Float(f) => serializer.serialize_f32(*f),
            CqlValue::Int(i) => serializer.serialize_i32(*i),
            CqlValue::BigInt(i) => serializer.serialize_i64(*i),
            CqlValue::Timestamp(t) => {
                let t: chrono::DateTime<chrono::Utc> = (*t).try_into().unwrap();
                t.serialize(serializer)
            }
            CqlValue::Inet(ip) => ip.serialize(serializer),
            CqlValue::List(xs) => {
                let mut seq = serializer.serialize_seq(Some(xs.len()))?;
                for x in xs {
                    seq.serialize_element(&SerializableCqlValueRef::new(x))?;
                }
                seq.end()
            }
            CqlValue::Map(map) => {
                let mut seq = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    seq.serialize_entry(
                        &SerializableCqlValueRef::new(k),
                        &SerializableCqlValueRef::new(v),
                    )?;
                }
                seq.end()
            }
            CqlValue::Set(set) => {
                let mut seq = serializer.serialize_seq(Some(set.len()))?;
                for x in set {
                    seq.serialize_element(&SerializableCqlValueRef::new(x))?;
                }
                seq.end()
            }
            CqlValue::UserDefinedType {
                keyspace: _,
                type_name: _,
                fields,
            } => {
                // just use serialize_map not serialize_struct, (requires 'static lifetime and we don't care about any formats that require the type name)
                let mut s = serializer.serialize_map(Some(fields.len()))?;
                for (k, v) in fields {
                    s.serialize_entry(k, &SerializableCqlValueRef(v.as_ref()))?;
                }
                s.end()
            }
            CqlValue::SmallInt(i) => serializer.serialize_i16(*i),
            CqlValue::TinyInt(i) => serializer.serialize_i8(*i),
            CqlValue::Time(t) => {
                let t: chrono::NaiveTime = (*t).try_into().unwrap();
                t.serialize(serializer)
            }
            CqlValue::Timeuuid(id) => serializer.collect_str(id),
            CqlValue::Uuid(id) => serializer.collect_str(id),
            CqlValue::Tuple(tup) => {
                let mut seq = serializer.serialize_tuple(tup.len())?;
                for x in tup {
                    seq.serialize_element(&SerializableCqlValueRef(x.as_ref()))?;
                }
                seq.end()
            }
            CqlValue::Varint(i) => BigInt::from(i.clone()).serialize(serializer),
        }
    }
}

fn dwim_bytes<S>(serializer: S, bytes: &[u8]) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if is_json_start(bytes) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(bytes) {
            return v.serialize(serializer);
        }
    }

    #[cfg(feature = "msgpack")]
    if let Ok(v) = rmp_serde::from_slice::<Value>(bytes) {
        return v.serialize(serializer);
    }

    serializer.serialize_bytes(bytes)
}

fn dwim_str<S>(serializer: S, s: &str) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if is_json_start(s) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            return v.serialize(serializer);
        }
    }

    serializer.serialize_str(s)
}

// quick check to avoid false parsing of non-json strings
fn is_json_start(s: impl AsRef<[u8]>) -> bool {
    match s.as_ref().first() {
        Some(c) => matches!(c, b'{' | b'[' | b'"' | b'0'..=b'9' | b't' | b'f' | b'n'),
        None => false,
    }
}
