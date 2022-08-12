use std::str::FromStr;

use regex::Regex;
use serde_json::Value;

use super::parser::Parser;

#[derive(Debug, Clone)]
pub struct JSONParser {
    array_compatibility: Regex,
    json_compatibility: Regex,
    filter_compatibility: Regex,
    number_compatibility: Regex,
    string_compatibility: Regex,
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

impl JSONParser {
    pub fn new() -> Self {
        let array_compatibility: Regex =
            Regex::new(r"\s*^\[((?P<other>.*),)?\s*(?P<value>.*)\s*\]$").unwrap();
        let json_compatibility: Regex =
            Regex::new(r#"\s*^\{((?P<other>.*),)?(\s*(?P<key>\S+)\s*:\s*(?P<value>.*)\s*)\}$"#)
                .unwrap();
        let filter_compatibility: Regex =
            Regex::new(r"(\.(?P<key>\w*)\s*(\[(?P<index>\d+)\])?)").unwrap();
        let number_compatibility: Regex = Regex::new(r"\s*(?P<value>\d+\s*)").unwrap();
        let string_compatibility: Regex = Regex::new(r#"\s*"(?P<value>.*)"\s*"#).unwrap();

        Self {
            array_compatibility,
            json_compatibility,
            filter_compatibility,
            number_compatibility,
            string_compatibility,
        }
    }

    pub fn parse(&self, json: &serde_json::Value, mut query: String) -> (Value, Output) {
        if self.json_compatibility.is_match(&query) {
            let mut json_value = serde_json::Map::new();

            loop {
                let (key, value, other) = {
                    let capture = self.json_compatibility.captures(&query).unwrap();

                    let mut key = capture.name("key").unwrap().as_str().to_string();
                    let value = capture.name("value").unwrap().as_str().to_string();

                    let other = match capture.name("other") {
                        Some(e) => e.as_str().to_string(),
                        None => "".to_string(),
                    };

                    if (key.starts_with("\"") && key.ends_with("\""))
                        || (key.starts_with("'") && key.ends_with("'"))
                    {
                        key.remove(0);
                        key.remove(key.len() - 1);
                    }

                    (key, value, other)
                };

                let (value, _) = self.parse(json, value);

                json_value.insert(key, value);

                if other.is_empty() {
                    break;
                }
                query = format!("{{{other}}}");
            }

            (json_value.into(), Output::Json)
        } else if self.filter_compatibility.is_match(&query) {
            let mut queries = vec![];

            for capture in self.filter_compatibility.captures_iter(&query) {
                let key = capture.name("key").unwrap().as_str();
                match capture.name("index") {
                    Some(e) => {
                        let index = e.as_str().parse::<usize>().unwrap();
                        queries.push((key.to_owned(), Some(index)))
                    }
                    None => queries.push((key.to_owned(), None)),
                };
            }

            let query_callback = |key: &String, index: &Option<usize>, json: &Value| -> Value {
                let mut value = Value::Null;

                if key.is_empty() {
                    match index {
                        Some(e) => value = json.get(e).cloned().unwrap_or_default(),
                        None => value = json.clone(),
                    }
                    if let Some(index) = index {
                        value = json.get(index).cloned().unwrap_or_default();
                    }
                } else {
                    value = json.get(key).cloned().unwrap_or_default();

                    if let Some(index) = index {
                        value = value.get(index).cloned().unwrap_or_default();
                    }
                }

                value
            };

            if let Some((key, index)) = queries.first() {
                let mut value = query_callback(key, index, json);

                for (_, (key, index)) in queries[1..].iter().enumerate() {
                    value = query_callback(key, index, &value);
                }

                return (value, Output::Filter);
            }

            (Value::Null, Output::Filter)
        } else if self.array_compatibility.is_match(&query) {
            let mut array_value = vec![];
            loop {
                let (value, other) = {
                    let capture = self.array_compatibility.captures(&query).unwrap();

                    let value = capture.name("value").unwrap().as_str().to_string();

                    let other = match capture.name("other") {
                        Some(e) => e.as_str().to_string(),
                        None => "".to_string(),
                    };

                    (value, other)
                };

                let (value, _) = self.parse(json, value);

                array_value.push(value);

                if other.is_empty() {
                    break;
                }

                query = format!("[{other}]");
            }

            array_value.reverse();
            (array_value.into(), Output::Array)
        } else if self.string_compatibility.is_match(&query) {
            let capture = self.string_compatibility.captures(&query).unwrap();

            let value = capture.name("value").unwrap().as_str();

            (value.into(), Output::String)
        } else if self.number_compatibility.is_match(&query) {
            let capture = self.number_compatibility.captures(&query).unwrap();

            let value = capture
                .name("value")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .unwrap();

            (value.into(), Output::Number)
        } else {
            (Value::Null, Output::Invalid)
        }
    }
}

mod test {
    use super::{JSONParser, Output};
    use serde_json::Value;

    #[test]
    fn test_multiple_json_input() {
        let json: Value = serde_json::from_str("{}").unwrap();
        let query = String::from(r#"{a: "1", b: 2, c: 3, d: 42}"#);

        let parser = JSONParser::new();
        let (value, output) = parser.parse(&json, query);

        let res = serde_json::json!({
            "a": "1",
            "b": 2,
            "c": 3,
            "d": 42,
        });

        assert_eq!(res, value);
        assert_eq!(output, Output::Json);
    }

    #[test]
    fn test_nested_json() {
        let json: Value = serde_json::from_str("{}").unwrap();
        let query = String::from(r#"{a: {b: "1"}, b: 2, c: 3, d: 42}"#);

        let parser = JSONParser::new();
        let (value, output) = parser.parse(&json, query);

        let res = serde_json::json!({
            "a": {"b": "1"},
            "b": 2,
            "c": 3,
            "d": 42,
        });

        assert_eq!(res, value);
        assert_eq!(output, Output::Json);
    }

    #[test]
    fn test_json_with_filter() {
        let json: Value = serde_json::from_str(r#"{"a": { "b": "1" }}"#).unwrap();
        let query = String::from(r#"{a: .a.b, b: 2, c: 3, d: 42}"#);

        let parser = JSONParser::new();
        let (value, output) = parser.parse(&json, query);

        let res = serde_json::json!({
            "a": "1",
            "b": 2,
            "c": 3,
            "d": 42,
        });

        assert_eq!(res, value);
    }

    #[test]
    fn test_json_with_filter_null() {
        let json: Value = serde_json::from_str(r#"{"a": { "b": "1" }}"#).unwrap();
        let query = String::from(r#"{a: ., b: 2, c: 3, d: 42}"#);

        let parser = JSONParser::new();
        let (value, output) = parser.parse(&json, query);

        let res = serde_json::json!({
            "a": {"a": { "b": "1" }},
            "b": 2,
            "c": 3,
            "d": 42,
        });

        assert_eq!(res, value);
    }
}
