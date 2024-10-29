use serde_json;

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
        let key = key.map_or_else(|| k.clone(), |key| format!("{}.{}", key, k));

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
        let key = key.map_or_else(|| i.to_string(), |key| format!("{}.{}", key, i));

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
            "b.d.0": 3,
            "b.d.1": 4,
            "e.0.f": 5,
            "e.0.g": 6,
            "e.1.h": 7,
            "e.1.i": 8,
        });

        assert_eq!(flatten(&json), expected);
    }
}
