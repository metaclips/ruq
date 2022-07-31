use std::str::FromStr;

use regex::Regex;
use serde_json::Value;

use super::parser::Operator;

#[derive(Debug, Clone)]
pub struct JSONParser {
    json_compatibility: Regex,
    filter_compatibility: Regex,
    number_compatibility: Regex,
    string_compatibility: Regex,
}

#[derive(Debug, PartialEq)]
enum Output {
    Json,
    Array,
    Filter,
    Number,
    String,
}

impl JSONParser {
    pub fn new() -> Self {
        let json_compatibility: Regex =
            Regex::new(r"\s*\{((?P<other>.*),)?(\s*(?P<key>\S+)\s*:\s*(?P<value>.*)\s*)\}")
                .unwrap();
        let filter_compatibility: Regex = Regex::new(r"\s*\.(?P<key>\S)").unwrap();
        let number_compatibility: Regex = Regex::new(r"\s*(?P<value>\d+)").unwrap();
        let string_compatibility: Regex = Regex::new(r"\s*(?P<value>\s+)").unwrap();

        Self {
            json_compatibility,
            filter_compatibility,
            number_compatibility,
            string_compatibility,
        }
    }

    pub fn parse(&self, json: serde_json::Value, operators: Vec<(Operator, String)>) {
        for (operator, value) in operators {
            // self.parser(&json, &value);
        }
    }

    fn parser(
        &self,
        json: &serde_json::Value,
        mut query: String,
    ) -> (Option<String>, Value, Output) {
        if self.json_compatibility.is_match(&query) {
            let mut json_value = serde_json::Map::new();

            loop {
                let (key, value, other) = {
                    let capture = self.json_compatibility.captures(&query).unwrap();

                    let key = capture.name("key").unwrap().as_str().to_string();
                    let value = capture.name("value").unwrap().as_str().to_string();

                    let other = match capture.name("other") {
                        Some(e) => e.as_str().to_string(),
                        None => "".to_string(),
                    };

                    (key, value, other)
                };

                json_value.insert(key, value.into());

                if other.is_empty() {
                    break;
                }
                query = format!("{{{other}}}");
            }

            (None, json_value.into(), Output::Json)
        } else if self.number_compatibility.is_match(&query) {
            let capture = self.number_compatibility.captures(&query).unwrap();

            let value = capture
                .name("value")
                .unwrap()
                .as_str()
                .parse::<usize>()
                .unwrap();

            (None, value.into(), Output::Number)
        } else if self.filter_compatibility.is_match(&query) {
            todo!()
        } else if self.string_compatibility.is_match(&query) {
            todo!()
        } else {
            todo!()
        }
    }
}

mod test {
    use super::{JSONParser, Output};
    use serde_json::Value;

    #[test]
    fn test_multiple_json_input() {
        let json: Value = serde_json::from_str("{}").unwrap();
        let query = String::from("{a: 1, b: 2, c: 3, d: 42}");

        let parser = JSONParser::new();
        let (key, value, output) = parser.parser(&json, query);

        let res = serde_json::json!({
            "a": "1",
            "b": "2",
            "c": "3",
            "d": "42",
        });

        assert_eq!(res, value);
        assert_eq!(output, Output::Json);
    }
}
