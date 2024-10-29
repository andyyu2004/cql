pub fn flatten(v: serde_json::Value) -> serde_json::Value {
    let mut result = serde_json::Map::new();

    match v {
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
            flatten_rec(v, None, &mut result)
        }
        v => return v,
    }

    serde_json::Value::Object(result)
}

fn flatten_rec(
    v: serde_json::Value,
    key: Option<&str>,
    result: &mut serde_json::Map<String, serde_json::Value>,
) {
    match v {
        serde_json::Value::Array(xs) => flatten_array(xs, key, result),
        serde_json::Value::Object(map) => flatten_map(map, key, result),
        v => drop(result.insert(key.unwrap().to_string(), v.clone())),
    }
}

fn flatten_map(
    map: serde_json::Map<String, serde_json::Value>,
    key: Option<&str>,
    result: &mut serde_json::Map<String, serde_json::Value>,
) {
    for (k, v) in map {
        let key = match key {
            Some(key) => format!("{key}.{k}"),
            None => k,
        };
        flatten_rec(v, Some(&key), result);
    }
}

fn flatten_array(
    xs: Vec<serde_json::Value>,
    key: Option<&str>,
    result: &mut serde_json::Map<String, serde_json::Value>,
) {
    for (i, v) in xs.into_iter().enumerate() {
        let key = key.map_or_else(|| format!("{i}"), |key| format!("{key}[{i}]"));
        flatten_rec(v, Some(&key), result);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use expect_test::{expect, Expect};
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

        assert_eq!(flatten(json), expected);
        assert_eq!(flatten(serde_json::Value::Null), serde_json::Value::Null);

        #[track_caller]
        fn check(v: serde_json::Value, expect: Expect) {
            expect.assert_eq(&serde_json::to_string_pretty(&flatten(v)).unwrap());
        }

        check(json!({}), expect![[r#"{}"#]]);
        check(
            json!([1, 2, 3]),
            expect![[r#"
                {
                  "0": 1,
                  "1": 2,
                  "2": 3
                }"#]],
        );
    }
}
