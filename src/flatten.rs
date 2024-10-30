use serde::{ser::SerializeMap, Serialize, Serializer};

pub struct DotFlattened<S: Serializer> {
    state: State<S>,
    prefix: String,
}

enum State<S: Serializer> {
    Top(S),
    Map {
        prefix: String,
        inner: S::SerializeMap,
    },
}

impl<S: Serializer<Ok = ()>> DotFlattened<S> {
    fn new(inner: S) -> Self {
        Self {
            state: State::Top(inner),
            prefix: String::new(),
        }
    }

    fn get_map_serializer(&mut self) -> (&str, &mut S::SerializeMap) {
        match &mut self.state {
            State::Top(_inner) => panic!("cannot flatten non-map value"),
            State::Map { prefix, inner } => (prefix, inner),
        }
    }

    fn serialize<T>(&mut self, value: T) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
    {
        let (prefix, inner) = self.get_map_serializer();
        inner.serialize_entry(prefix, &value)
    }
}

impl<S: Serializer<Ok = ()>> Serializer for DotFlattened<S> {
    type Ok = S::Ok;

    type Error = S::Error;

    type SerializeSeq = Self;

    type SerializeTuple = S::SerializeTuple;

    type SerializeTupleStruct = S::SerializeTupleStruct;

    type SerializeTupleVariant = S::SerializeTupleVariant;

    type SerializeMap = Self;

    type SerializeStruct = S::SerializeStruct;

    type SerializeStructVariant = S::SerializeStructVariant;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_i8(mut self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_i16(mut self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_i32(mut self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_i64(mut self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_u64(mut self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_f32(mut self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_f64(mut self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_char(mut self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.serialize(v)
    }

    fn serialize_none(mut self) -> Result<Self::Ok, Self::Error> {
        self.serialize(())
    }

    fn serialize_some<T>(mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize(value)
    }

    fn serialize_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.serialize(())
    }

    fn serialize_unit_struct(mut self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(mut self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.state = match self.state {
            State::Top(inner) => State::Map {
                prefix: String::new(),
                inner: inner.serialize_map(len)?,
            },
            State::Map { prefix, inner } => State::Map { prefix, inner },
        };

        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!("probably do the same as map")
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!("struct variant")
    }
}

impl<S: Serializer<Ok = ()>> serde::ser::SerializeSeq for DotFlattened<S> {
    type Ok = S::Ok;

    type Error = S::Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<S: Serializer<Ok = ()>> serde::ser::SerializeMap for DotFlattened<S> {
    type Ok = S::Ok;

    type Error = S::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let (prefix, inner) = self.get_map_serializer();
        inner.serialize_key(&format!("{prefix}.{key}"))
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub fn flatten(json: &serde_json::Value) -> serde_json::Value {
    let mut result = serde_json::Map::new();

    match json {
        serde_json::Value::Object(map) => {
            flatten_map(map, None, &mut result);
        }
        serde_json::Value::Array(arr) => {
            flatten_array(arr, None, &mut result);
        }
        _ => {}
    }

    serde_json::Value::Object(result)
}

fn flatten_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: Option<&str>,
    result: &mut serde_json::Map<String, serde_json::Value>,
) {
    for (k, v) in map {
        let key = key.map_or_else(|| k.clone(), |key| format!("{key}.{k}"));

        match v {
            serde_json::Value::Object(inner_map) => {
                flatten_map(inner_map, Some(&key), result);
            }
            serde_json::Value::Array(inner_arr) => {
                flatten_array(inner_arr, Some(&key), result);
            }
            _ => {
                result.insert(key, v.clone());
            }
        }
    }
}

fn flatten_array(
    arr: &[serde_json::Value],
    key: Option<&str>,
    result: &mut serde_json::Map<String, serde_json::Value>,
) {
    for (i, v) in arr.iter().enumerate() {
        let key = key.map_or_else(|| i.to_string(), |key| format!("{key}[{i}]"));

        match v {
            serde_json::Value::Object(inner_map) => {
                flatten_map(inner_map, Some(&key), result);
            }
            serde_json::Value::Array(inner_arr) => {
                flatten_array(inner_arr, Some(&key), result);
            }
            _ => {
                result.insert(key, v.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use expect_test::expect;
    use serde::Serialize;
    use serde_json::json;

    #[test]
    fn test_flatten() {
        let json = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": [3, 4],
            },
            "e": [
                {
                    "f": 5,
                    "g": 6,
                },
                {
                    "h": 7,
                    "i": 8,
                },
            ],
        });

        let expected = json!({
            "a": 1,
            "b.c": 2,
            "b.d[0]": 3,
            "b.d[1]": 4,
            "e[0].f": 5,
            "e[0].g": 6,
            "e[1].h": 7,
            "e[1].i": 8,
        });

        assert_eq!(flatten(&json), expected);
    }

    #[test]
    fn test_flatten2() {
        let json = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": [3, 4],
            },
            "e": [
                {
                    "f": 5,
                    "g": 6,
                },
                {
                    "h": 7,
                    "i": 8,
                },
            ],
        });

        let mut serializer = serde_json::Serializer::pretty(vec![]);
        json.serialize(DotFlattened::new(&mut serializer)).unwrap();
        expect![[r#"
            {
              "a": 1,
              "b": {
                "c": 2,
                "d": [
                  3,
                  4
                ]
              },
              "e": [
                {
                  "f": 5,
                  "g": 6
                },
                {
                  "h": 7,
                  "i": 8
                }
              ]
            }"#]]
        .assert_eq(&String::from_utf8(serializer.into_inner()).unwrap());
    }
}
