use std::str::FromStr;

use regex::Regex;
use serde_json::{Map, Value};

use super::parser::{Operator, Parser};

#[derive(Debug, Clone)]
pub struct Json {
    pub json: Value,
}

#[derive(Debug, PartialEq)]
pub enum Output {
    Json,
    Array,
    Filter,
    Number,
    String,
    Invalid,
}

impl Json {
    pub fn new(json: Value) -> Self {
        Self { json }
    }

    pub fn create_valid_json(&self, mut query: String) -> Value {
        let parser = Regex::new(r#"(?P<filter>(\.\w*(\[\d+\])?)+)"#).unwrap();
        let filter_compatibility: Regex =
            Regex::new(r"(\.(?P<key>\w*)\s*(\[(?P<index>\d+)\])?)").unwrap();

        for capture in parser.captures_iter(query.clone().as_str()) {
            let filter = capture.name("filter").unwrap().as_str();

            let mut value = self.json.clone();
            for filter_capture in filter_compatibility.captures_iter(filter) {
                let key = filter_capture.name("key").unwrap().as_str();
                if !key.is_empty() {
                    value = value.get(key).cloned().unwrap_or_default();
                }

                if let Some(e) = filter_capture.name("index") {
                    value = value
                        .get(e.as_str().parse::<usize>().unwrap())
                        .cloned()
                        .unwrap_or_default()
                }
            }
            query = parser.replace(&query, value.to_string()).to_string();
        }

        serde_json::from_str(&query).unwrap()
    }

    pub fn json_data_operator(mut json: Vec<(Operator, Value)>) -> Value {
        let (mut recent_operator, mut recent_value) = json.first().unwrap().clone();

        for (operator, value) in json.drain(1..) {
            let value = match recent_operator {
                Operator::Addition => Self::add_json_data(recent_value, value),
                Operator::Subtration => Self::subtract_json_data(recent_value, value),
                Operator::Multiplication => Self::multiply_json_data(recent_value, value),
                Operator::Division => {
                    todo!()
                }
                Operator::Modulo => {
                    todo!()
                }
                Operator::Nil => {
                    todo!()
                }
            };

            recent_operator = operator.clone();
            recent_value = value;
        }

        recent_value
    }

    fn add_json_data(pre: Value, post: Value) -> Value {
        let pre_type_id = pre.to_string();
        let post_type_id = post.to_string();
        match (pre, post) {
            (Value::Array(e), Value::Array(f)) => [e, f].concat().into(),
            (Value::Object(mut e), Value::Object(mut f)) => {
                e.append(&mut f);
                e.into()
            }
            (Value::Number(a), Value::Number(e)) => {
                let value = a.as_f64().unwrap() + e.as_f64().unwrap();
                return value.into();
            }
            (Value::String(a), Value::String(e)) => [a, e].concat().into(),
            _ => panic!("{:?} and {:?} cannot be added", pre_type_id, post_type_id),
        }
    }

    fn subtract_json_data(pre: Value, post: Value) -> Value {
        let pre_type_id = pre.to_string();
        let post_type_id = post.to_string();
        match (pre, post) {
            (Value::Array(e), Value::Array(f)) => {
                let mut result = vec![];

                for value in e {
                    if !f.contains(&value) {
                        result.push(value)
                    }
                }

                result.into()
            }
            (Value::Number(a), Value::Number(e)) => {
                let value = a.as_f64().unwrap() - e.as_f64().unwrap();
                return value.into();
            }
            _ => panic!(
                "{:?} and {:?} cannot be subtracted",
                pre_type_id, post_type_id
            ),
        }
    }

    fn multiply_json_data(pre: Value, post: Value) -> Value {
        let pre_type_id = pre.to_string();
        let post_type_id = post.to_string();
        match (pre, post) {
            (Value::Object(a), Value::Object(e)) => {
                let mut result = Map::new();

                for (key, pre_value) in a {
                    if let Some(post_value) = e.get(&key) {
                        result.insert(key, Self::multiply_json_data(pre_value, post_value.clone()));
                    } else {
                        result.insert(key, pre_value);
                    }
                }
            }
            (Value::Number(a), Value::Number(e)) => {
                let value = a.as_f64().unwrap() * e.as_f64().unwrap();
                return value.into();
            }
            _ => panic!(
                "{:?} and {:?} cannot be multiplied",
                pre_type_id, post_type_id
            ),
        }

        todo!()
    }
}

mod test {
    use std::str::FromStr;

    use super::{Json, Output};
    use serde_json::Value;

    #[test]
    fn test_make_valid_json() {
        struct TestParser {
            query: String,
            result: Value,
            json: Value,
        }
        let tests = [
            TestParser {
                query: String::from(r#"{"a": .a}"#),
                result: Value::from_str(r#"{"a":"Hello"}"#).unwrap(),
                json: serde_json::json!({
                    "a": "Hello",
                    "b": 1,
                    "c": true,
                }),
            },
            TestParser {
                query: String::from(r#"{"a": .a[0]}"#),
                result: Value::from_str(r#"{"a":{"a":55,"c":100}}"#).unwrap(),
                json: Value::from_str(
                    r#"{"a": [{"a": 55, "c": 100}, {"b": 2}], "b": 1,"c": true}"#,
                )
                .unwrap(),
            },
            TestParser {
                query: String::from(" .a[0]"),
                result: Value::from_str(r#"{"a":55,"c":100}"#).unwrap(),
                json: Value::from_str(r#"{"a": [{"a": 55, "c": 100}, {"b": 2}]}"#).unwrap(),
            },
        ];

        for (i, mut test) in tests.into_iter().enumerate() {
            let parser = Json::new(test.json.clone());
            let value = parser.create_valid_json(test.query.clone());
            assert_eq!(value, test.result, "Failed testing index {}", i);
        }
    }
}
