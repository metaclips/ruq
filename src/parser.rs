use regex::Regex;
use serde_json::Value;
use std::fmt::Debug;

use crate::json_parser::Json;

#[derive(Clone, Debug, PartialEq)]
pub enum Parser {
    JSON(Value),
    Length,
    Operator(Vec<(Operator, Value)>),
    Nil,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Addition,
    Subtration,
    Multiplication,
    Division { ignore_infinite_divisor: bool },
    Modulo { ignore_infinite_divisor: bool },
    Nil,
}

impl Parser {
    pub fn new(json_data: Value, query: &str) -> Value {
        todo!()
    }

    fn parse(json_data: Value, data: &str) -> Value {
        let parser = Regex::new(r"\s*(?P<pre>.*?)\s*\|\s*(?P<post>.*)\s*").unwrap();

        let (mut pre, mut post) = Self::regexer(&parser, data);

        let mut value = Self::query(json_data, pre.to_string());

        while post != None {
            (pre, post) = Self::regexer(&parser, post.unwrap());
            value = Self::query(value.clone(), pre.to_string());
        }

        value
    }

    fn query(json_data: Value, query: String) -> Value {
        match Parser::parse_pipe(json_data.clone(), query) {
            Parser::JSON(e) => e,
            Parser::Length => Self::get_json_length(&json_data).into(),
            Parser::Operator(mut e) => Json::json_data_operator(e),
            Parser::Nil => Value::Null,
        }
    }

    fn get_json_length(json_data: &Value) -> f64 {
        if let Some(e) = json_data.as_array() {
            return e.len() as f64;
        } else if let Some(e) = json_data.as_object() {
            return e.len() as f64;
        } else if let Some(e) = json_data.as_str() {
            return e.len() as f64;
        } else if let Some(e) = json_data.as_f64() {
            return e;
        } else if let Some(e) = json_data.as_i64() {
            return e as f64;
        } else if let Some(e) = json_data.as_u64() {
            return e as f64;
        }

        panic!("Cannot get length of type")
    }

    fn regexer<'a>(parser: &Regex, data: &'a str) -> (&'a str, Option<&'a str>) {
        if parser.is_match(data) {
            let capture = parser.captures(data).unwrap();

            let pre = capture.name("pre").unwrap().as_str();
            let post = capture.name("post").map(|a| a.as_str());
            return (pre, post);
        }

        (data, None)
    }

    fn parse_pipe(json_data: Value, mut data: String) -> Self {
        let json_parser = Json::new(json_data);

        let length_compatibily = Regex::new(r"length").unwrap();
        let operator_compatibily =
            Regex::new(r"\s*(?P<pre>.*?)\s*(?P<operator>\+|\*|/|%|-)\s*(?P<post>.*)\s*").unwrap();

        if length_compatibily.is_match(&data) {
            return Parser::Length;
        } else if operator_compatibily.is_match(&data) {
            let mut operators = vec![];

            let mut data = data;
            let mut ignore_infinite_divisor_ = false;
            loop {
                if !operator_compatibily.is_match(&data) {
                    if !data.is_empty() {
                        let value = json_parser.parse_json(data.to_string());
                        operators.push((Operator::Nil, value));
                    }

                    return Parser::Operator(operators);
                }

                let (pre, post, operator) = {
                    if let Some(capture) = operator_compatibily.captures(&data) {
                        let mut operator = {
                            if let Some(op) = capture.name("operator") {
                                Operator::from(op.as_str())
                            } else {
                                Operator::Nil
                            }
                        };

                        let mut pre = {
                            if let Some(op) = capture.name("pre") {
                                op.as_str()
                            } else {
                                ""
                            }
                        };

                        let mut post = {
                            if let Some(op) = capture.name("post") {
                                op.as_str()
                            } else {
                                ""
                            }
                        };

                        if pre.starts_with("(") && post.ends_with(")?") {
                            ignore_infinite_divisor_ = true;
                            pre = pre.strip_prefix("(").unwrap();
                            post = post.strip_suffix(")?").unwrap();
                        }

                        match &mut operator {
                            Operator::Division {
                                ignore_infinite_divisor,
                            } => *ignore_infinite_divisor = ignore_infinite_divisor_,
                            Operator::Modulo {
                                ignore_infinite_divisor,
                            } => *ignore_infinite_divisor = ignore_infinite_divisor_,
                            _ => {}
                        }

                        let pre = json_parser.parse_json(pre.to_owned());

                        (pre, post, operator)
                    } else {
                        (Value::Null, "", Operator::Nil)
                    }
                };

                operators.push((operator, pre));

                data = post.to_string();
            }
        }

        Parser::JSON(json_parser.parse_json(data))
    }
}

impl From<&str> for Operator {
    fn from(val: &str) -> Self {
        match val {
            "+" => Self::Addition,
            "-" => Self::Subtration,
            "/" => Self::Division {
                ignore_infinite_divisor: false,
            },
            "*" => Self::Multiplication,
            "%" => Self::Modulo {
                ignore_infinite_divisor: false,
            },
            _ => Self::Nil,
        }
    }
}

mod test_parser {
    use std::str::FromStr;

    use super::*;
    struct TestParser {
        query: String,
        result: Value,
        json: Value,
    }

    #[test]
    fn test_nil_parser() {
        let tests = [
            TestParser {
                query: String::from(".[]"),
                result: Value::from_str(r#"[1,0,-1]"#).unwrap(),
                json: Value::from_str(r#"[1,0,-1]"#).unwrap(),
            },
            TestParser {
                query: String::from(".[].[0]"),
                result: Value::from_str(r#"1"#).unwrap(),
                json: Value::from_str(r#"[1,0,-1]"#).unwrap(),
            },
            TestParser {
                query: String::from("."),
                result: Value::from_str(r#"{}"#).unwrap(),
                json: Value::from_str(r#"{}"#).unwrap(),
            },
            TestParser {
                query: String::from(" ."),
                result: Value::from_str(r#"{"a": 1}"#).unwrap(),
                json: Value::from_str(r#"{"a": 1}"#).unwrap(),
            },
            TestParser {
                query: String::from(" . "),
                result: Value::from_str(r#"{"a": 1}"#).unwrap(),
                json: Value::from_str(r#"{"a": 1}"#).unwrap(),
            },
        ];

        for (i, test) in tests.into_iter().enumerate() {
            let parsed = Parser::parse(test.json.clone(), &test.query);
            assert_eq!(parsed, test.result, "Failed testing index {}", i);
        }
    }

    #[test]
    fn test_nil_with_length() {
        let tests = [
            TestParser {
                query: String::from(".a | length"),
                result: serde_json::json!(1.0),
                json: Value::from_str(r#"{"a": 1}"#).unwrap(),
            },
            TestParser {
                query: String::from(" .a|length"),
                result: serde_json::json!(2.0),
                json: Value::from_str(r#"{"a": [{"a": 1}, {"b": 2}]}"#).unwrap(),
            },
            TestParser {
                query: String::from(" .a[0]|length"),
                result: serde_json::json!(2.0),
                json: Value::from_str(r#"{"a": [{"a": 55, "c": 100}, {"b": 2}]}"#).unwrap(),
            },
            TestParser {
                query: String::from(" .a[0].c.d|length"),
                result: serde_json::json!(100.0),
                json: Value::from_str(r#"{"a": [{"a": 55, "c": { "d": 100}}, {"b": 2}]}"#).unwrap(),
            },
        ];

        for (i, test) in tests.into_iter().enumerate() {
            let parsed = Parser::parse(test.json.clone(), &test.query);
            assert_eq!(parsed, test.result, "Failed testing index {}", i);
        }
    }

    #[test]
    fn test_piped_operator() {
        let tests = [
            TestParser {
                query: String::from(r#". | {"a": .a} + {"b": .b} + {"c": .c} + {"a": .c}"#),
                result: serde_json::json!({
                    "a": true,
                    "b": 1,
                    "c": true,
                }),
                json: serde_json::json!({
                    "a": "Hello",
                    "b": 1,
                    "c": true,
                }),
            },
            TestParser {
                query: String::from(r#". | {"a": .a} + {"b": {"a": .b}} + {"c": .c} + {"a": .c}"#),
                result: serde_json::json!({
                    "a": true,
                    "b": {"a": 1},
                    "c": true,
                }),
                json: serde_json::json!({
                    "a": "Hello",
                    "b": 1,
                    "c": true,
                }),
            },
            TestParser {
                query: String::from(
                    r#". | {"a": .a} + {"b": .b} + {"c": .c} + {"a": .c} | .b + 1"#,
                ),
                result: serde_json::json!(6.0),
                json: serde_json::json!({
                    "a": "Hello",
                    "b": 5,
                    "c": true,
                }),
            },
            TestParser {
                query: String::from(
                    r#". | {"a": .a} + {"b": {"a": .b}} + {"c": .c} + {"a": .c} | length"#,
                ),
                result: serde_json::json!(3.0),
                json: serde_json::json!({
                    "a": "Hello",
                    "b": 1,
                    "c": true,
                }),
            },
            TestParser {
                query: String::from(r#". | .d + 1 | length"#),
                result: serde_json::json!(1.0),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#"2 + .d  | length"#),
                result: serde_json::json!(2.0),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#""Hello" * 2"#),
                result: serde_json::json!("HelloHello"),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#". / ", ""#),
                result: serde_json::json!(["a", "b,c,d", "e"]),
                json: serde_json::json!("a, b,c,d, e"),
            },
            TestParser {
                query: String::from(r#"10 / . * 3"#),
                result: serde_json::json!(5.0),
                json: serde_json::json!(6),
            },
            TestParser {
                query: String::from(r#".[] | (1 / .)?"#),
                result: serde_json::json!([1.0, -1.0]),
                json: serde_json::json!([1, 0, -1]),
            },
            TestParser {
                query: String::from(r#".[] | (1 / 1 / .)?"#),
                result: serde_json::json!([1.0, -1.0]),
                json: serde_json::json!([1, 0, -1]),
            },
            TestParser {
                query: String::from(r#"12 % . * 3"#),
                result: serde_json::json!(6.0),
                json: serde_json::json!(5),
            },
            TestParser {
                query: String::from(r#".[] | (3 % .)?"#),
                result: serde_json::json!([1.0, 1.0]),
                json: serde_json::json!([2, 0, -2]),
            },
            TestParser {
                query: String::from(r#".[] | (3 % 2 / .)?"#),
                result: serde_json::json!([1.0, -1.0]),
                json: serde_json::json!([1, 0, -1]),
            },
            TestParser {
                query: String::from(r#". | ["xml", "json"] - ["xml"]"#),
                result: Value::from_str(r#"["json"]"#).unwrap(),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#". | ["xml", "json"] - ["xml"]"#),
                result: Value::from_str(r#"["json"]"#).unwrap(),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#"[{"xml": 1}, {"yaml": 2}] - [{"xml": 1}]"#),
                result: Value::from_str(r#"[{"yaml": 2}]"#).unwrap(),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#"[{"xml": 1}, {"yaml": 2}] - [{"xml": 2}]"#),
                result: Value::from_str(r#"[{"xml": 1}, {"yaml": 2}]"#).unwrap(),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#"{"k": {"a": 1, "b": 2}} * {"k": {"a": 0,"c": 3}}"#),
                result: Value::from_str(r#"{"k": {"a": 0.0, "b": 2, "c": 3}}"#).unwrap(),
                json: serde_json::json!({}),
            },
        ];

        for (i, test) in tests.into_iter().enumerate() {
            let parsed = Parser::parse(test.json.clone(), &test.query);
            assert_eq!(parsed, test.result, "Failed testing index {}", i);
        }
    }
}

mod test_json_types {
    use super::*;
    struct TestParser {
        query: String,
        json_types: Vec<Parser>,
    }

    #[test]
    fn find_length() {
        let tests = [
            TestParser {
                query: String::from("length"),
                json_types: vec![Parser::Length],
            },
            TestParser {
                query: String::from(" length"),
                json_types: vec![Parser::Length],
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let parsed = Parser::parse_pipe(Value::Null, test.query.clone());
            assert_eq!(vec![parsed], test.json_types, "Failed testing index {}", i);
        }
    }
}
