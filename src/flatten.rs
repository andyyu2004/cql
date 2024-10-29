use serde::Serializer;

pub struct DotFlattened<S> {
    inner: S,
    state: State,
}

impl<S> DotFlattened<S> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            state: State::Top,
        }
    }
}

enum State {
    Top,
    InMap,
    InList,
}

impl<S: Serializer> Serializer for DotFlattened<S> {
    type Ok = S::Ok;

    type Error = S::Error;

    type SerializeSeq = S::SerializeSeq;

    type SerializeTuple = S::SerializeTuple;

    type SerializeTupleStruct = S::SerializeTupleStruct;

    type SerializeTupleVariant = S::SerializeTupleVariant;

    type SerializeMap = S::SerializeMap;

    type SerializeStruct = S::SerializeStruct;

    type SerializeStructVariant = S::SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_none()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.inner.serialize_some(value)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner
            .serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.inner.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.inner
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(&mut self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.inner.serialize_struct(name, len)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.inner
            .serialize_struct_variant(name, variant_index, variant, len)
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
