use regex::Regex;
use serde_json::{Map, Number, Value};
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parser {
    Json(Value),
    Length,
    Operator(Vec<(Operator, Value)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Addition,
    Subtration,
    Multiplication,
    Division { ignore_infinite_divisor: bool },
    Modulo { ignore_infinite_divisor: bool },
    Nil,
}

struct JsonParser {
    json: Value,
    filter_regex: Regex,
    filter_converter: Regex,
    filter_reversal: Regex,
}

impl JsonParser {
    fn new(json: Value) -> Self {
        let filter_regex = Regex::new(r"(\.(?P<key>\w*)\s*(\[(?P<index>\d+?)\])?)").unwrap();
        let filter_converter = Regex::new(r#"(?P<filter>(\.\w*(\[\d*\])?)+)"#).unwrap();
        let filter_reversal = Regex::new(r#"(?P<filter>(""\.\w*(\[\d*\])?"")+)"#).unwrap();
        Self {
            json,
            filter_regex,
            filter_converter,
            filter_reversal,
        }
    }

    fn parse_json(&self, query: String) -> Value {
        let mut value = self.convert_filter_to_string(query);
        self.create_valid_json(&mut value);
        value
    }

    fn convert_filter_to_string(&self, mut query: String) -> Value {
        query = self
            .filter_converter
            .replace_all(&query, r#""$filter""#)
            .to_string();

        for capture in self.filter_reversal.find_iter(&query.clone()) {
            let mut capture = capture.as_str().to_string();
            if capture.starts_with(r#""""#) && capture.ends_with(r#""""#) {
                capture = format!(r#""_{}_""#, capture[2..capture.len() - 2].to_string());
                query = self.filter_reversal.replace(&query, capture).to_string();
            }
        }

        Value::from_str(&query).unwrap()
    }

    fn create_valid_json(&self, json: &mut Value) {
        match json {
            Value::Array(e) => {
                for value in e {
                    self.create_valid_json(value)
                }
            }
            Value::Object(e) => {
                for (_, value) in e {
                    self.create_valid_json(value)
                }
            }
            Value::String(e) => {
                if self.filter_regex.is_match(e) {
                    if e.starts_with('_') && e.ends_with('_') {
                        *e = e[1..e.len() - 1].to_string();
                    } else {
                        let mut value = self.json.clone();
                        for filter_capture in self.filter_regex.captures_iter(e) {
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

                        *json = value
                    }
                }
            }
            _ => {}
        }
    }

    pub fn json_data_operator(mut json: Vec<(Operator, Value)>) -> Value {
        let (mut recent_operator, mut recent_value) = json.first().unwrap().clone();

        for (operator, value) in json.drain(1..) {
            let value = match recent_operator {
                Operator::Addition => Self::add_json_data(recent_value, value),
                Operator::Subtration => Self::subtract_json_data(recent_value, value),
                Operator::Multiplication => Self::multiply_json_data(recent_value, value),
                Operator::Division {
                    ignore_infinite_divisor,
                } => Self::divide_json_data(recent_value, value, ignore_infinite_divisor),
                Operator::Modulo {
                    ignore_infinite_divisor,
                } => Self::modulo_json_data(recent_value, value, ignore_infinite_divisor),
                Operator::Nil => unreachable!(),
            };

            recent_operator = operator;
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
                value.into()
            }
            (Value::String(a), Value::String(e)) => [a, e].concat().into(),
            (Value::Number(e), Value::Null) | (Value::Null, Value::Number(e)) => e.into(),
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
                value.into()
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
            (Value::Object(a), Value::Object(mut e)) => {
                let mut result = Map::new();

                for (key, pre_value) in a {
                    if let Some(post_value) = e.get(key.as_str()) {
                        result.insert(
                            key.clone(),
                            Self::multiply_json_data(pre_value, post_value.clone()),
                        );
                        e.remove(&key);
                    } else {
                        println!("{key} {}", pre_value);
                        result.insert(key, pre_value);
                    }
                }

                result.extend(e);
                result.into()
            }
            (Value::Number(a), Value::Number(e)) => {
                let value = a.as_f64().unwrap() * e.as_f64().unwrap();
                value.into()
            }
            (Value::String(mut e), Value::Number(a)) | (Value::Number(a), Value::String(mut e)) => {
                let a = a.as_u64().unwrap();
                if a == 0 {
                    return Value::Null;
                }

                for _ in 0..(a - 1) {
                    e += e.clone().as_str();
                }
                e.into()
            }
            _ => panic!(
                "{:?} and {:?} cannot be multiplied",
                pre_type_id, post_type_id
            ),
        }
    }

    fn divide_json_data(pre: Value, post: Value, ignore_infinite_divisor: bool) -> Value {
        let pre_type_id = pre.to_string();
        let post_type_id = post.to_string();

        match (pre, post) {
            (Value::String(e), Value::String(a)) => {
                let value: Vec<_> = e.split(&a).collect();
                value.into()
            }
            (Value::Number(e), Value::Array(a)) => {
                let mut result = vec![];
                let e = Self::convert_to_f64(e);

                for value in a {
                    match value {
                        Value::Number(a) => {
                            let a = Self::convert_to_f64(a);

                            if a == 0.0 {
                                if !ignore_infinite_divisor {
                                    panic!(
                                        "{:?} and {:?} cannot be divided",
                                        pre_type_id, post_type_id
                                    )
                                }

                                continue;
                            }

                            result.push(e / a)
                        }
                        _ => panic!("{:?} and {:?} cannot be divided", pre_type_id, post_type_id),
                    }
                }

                result.into()
            }
            (Value::Array(e), Value::Number(a)) => {
                let mut result = vec![];
                let a = Self::convert_to_f64(a);
                if a == 0.0 {
                    if !ignore_infinite_divisor {
                        panic!("{:?} and {:?} cannot be divided", pre_type_id, post_type_id)
                    }

                    return Value::Null;
                }

                for value in e {
                    match value {
                        Value::Number(e) => {
                            let e = Self::convert_to_f64(e);

                            result.push(e / a)
                        }
                        _ => panic!("{:?} and {:?} cannot be divided", pre_type_id, post_type_id),
                    }
                }

                result.into()
            }
            (Value::Number(e), Value::Number(a)) => {
                let e = Self::convert_to_f64(e);
                let a = Self::convert_to_f64(a);

                if a == 0.0 {
                    panic!("{:?} and {:?} cannot be divided", pre_type_id, post_type_id);
                }

                (e / a).into()
            }
            _ => panic!("{:?} and {:?} cannot be divided", pre_type_id, post_type_id),
        }
    }

    fn convert_to_f64(value: Number) -> f64 {
        if value.is_f64() {
            value.as_f64().unwrap()
        } else if value.is_i64() {
            value.as_i64().unwrap() as f64
        } else if value.is_u64() {
            value.as_u64().unwrap() as f64
        } else {
            0.0
        }
    }

    fn modulo_json_data(pre: Value, post: Value, ignore_infinite_divisor: bool) -> Value {
        let pre_type_id = pre.to_string();
        let post_type_id = post.to_string();

        match (pre, post) {
            (Value::Number(e), Value::Array(a)) => {
                let mut result = vec![];
                let e = Self::convert_to_f64(e);

                for value in a {
                    match value {
                        Value::Number(a) => {
                            let a = Self::convert_to_f64(a);

                            if a == 0.0 {
                                if !ignore_infinite_divisor {
                                    panic!(
                                        "Cannot compute {:?} and {:?} modulo",
                                        pre_type_id, post_type_id
                                    )
                                }

                                continue;
                            }

                            result.push(e % a)
                        }
                        _ => panic!(
                            "Cannot compute {:?} and {:?} modulo",
                            pre_type_id, post_type_id
                        ),
                    }
                }

                result.into()
            }
            (Value::Array(e), Value::Number(a)) => {
                let mut result = vec![];
                let a = Self::convert_to_f64(a);
                if a == 0.0 {
                    if !ignore_infinite_divisor {
                        panic!(
                            "Cannot compute {:?} and {:?} modulo",
                            pre_type_id, post_type_id
                        )
                    }

                    return Value::Null;
                }

                for value in e {
                    match value {
                        Value::Number(e) => {
                            let e = Self::convert_to_f64(e);

                            result.push(e % a)
                        }
                        _ => panic!(
                            "Cannot compute {:?} and {:?} modulo",
                            pre_type_id, post_type_id
                        ),
                    }
                }

                result.into()
            }
            (Value::Number(e), Value::Number(a)) => {
                let e = Self::convert_to_f64(e);
                let a = Self::convert_to_f64(a);

                if a == 0.0 {
                    panic!(
                        "Cannot compute {:?} and {:?} modulo",
                        pre_type_id, post_type_id
                    );
                }

                (e % a).into()
            }
            _ => panic!(
                "Cannot compute {:?} and {:?} modulo",
                pre_type_id, post_type_id
            ),
        }
    }
}

impl Parser {
    pub fn parse(json_data: Value, data: &str) -> Value {
        let parser = Regex::new(r"\s*(?P<pre>.*?)\s*\|\s*(?P<post>.*)\s*").unwrap();

        let (mut pre, mut post) = Self::regexer(&parser, data);

        let mut value = Self::query(json_data, pre.to_string());

        while post != None {
            (pre, post) = Self::regexer(&parser, post.unwrap());
            value = Self::query(value, pre.to_string());
        }

        value
    }

    fn query(json_data: Value, query: String) -> Value {
        match Parser::parse_pipe(json_data.clone(), query) {
            Parser::Json(e) => e,
            Parser::Length => Self::get_json_length(&json_data).into(),
            Parser::Operator(e) => JsonParser::json_data_operator(e),
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

    fn parse_pipe(json_data: Value, data: String) -> Self {
        let json_parser = JsonParser::new(json_data);

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
                        let value = json_parser.parse_json(data);
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

                        if pre.starts_with('(') && post.ends_with(")?") {
                            ignore_infinite_divisor_ = true;
                            pre = pre.strip_prefix('(').unwrap();
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

        Parser::Json(json_parser.parse_json(data))
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

    #[test]
    fn test_nil_parser() {
        use super::*;
        use std::str::FromStr;

        struct TestParser {
            query: String,
            result: Value,
            json: Value,
        }

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
                query: String::from(r#" . | {"michael_age": .a, "michael_height": .a}"#),
                result: Value::from_str(r#"{"michael_age": 1, "michael_height": 1}"#).unwrap(),
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
        use super::*;
        use std::str::FromStr;

        struct TestParser {
            query: String,
            result: Value,
            json: Value,
        }

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
        use super::*;
        use std::str::FromStr;

        struct TestParser {
            query: String,
            result: Value,
            json: Value,
        }

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

    #[test]
    fn find_length() {
        use super::*;
        struct TestParser {
            query: String,
            json_types: Vec<Parser>,
        }

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

mod test {

    #[test]
    fn test_convert_to_json() {
        use super::JsonParser;
        use serde_json::Value;
        use std::str::FromStr;

        struct TestParser {
            query: String,
            result: Value,
            json: Value,
        }
        let tests = [
            TestParser {
                query: String::from(r#"{"a": .a}"#),
                result: Value::from_str(r#"{"a":".a"}"#).unwrap(),
                json: serde_json::json!({}),
            },
            TestParser {
                query: String::from(r#"{"a": .a[0]}"#),
                result: Value::from_str(r#"{"a": ".a[0]"}"#).unwrap(),
                json: Value::from_str(r#"{}"#).unwrap(),
            },
            TestParser {
                query: String::from(" .a[0]"),
                result: Value::from_str(r#"".a[0]""#).unwrap(),
                json: Value::from_str(r#"{}"#).unwrap(),
            },
        ];

        for (i, test) in tests.into_iter().enumerate() {
            let parser = JsonParser::new(test.json.clone());
            let value = parser.convert_filter_to_string(test.query.clone());
            assert_eq!(value, test.result, "Failed testing index {}", i);
        }
    }

    #[test]
    fn test_make_valid_json() {
        use super::JsonParser;
        use serde_json::Value;
        use std::str::FromStr;

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
            TestParser {
                query: String::from(r#"{"a": ".a", "b": .a}"#),
                result: Value::from_str(r#"{"a":".a", "b": "Hello"}"#).unwrap(),
                json: serde_json::json!({
                    "a": "Hello",
                    "b": 1,
                    "c": true,
                }),
            },
        ];

        for (i, test) in tests.into_iter().enumerate() {
            let parser = JsonParser::new(test.json.clone());
            let value = parser.parse_json(test.query.clone());
            assert_eq!(value, test.result, "Failed testing index {}", i);
        }
    }
}
