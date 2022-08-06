use regex::Regex;
use serde_json::Value;
use std::fmt::Debug;

use crate::{json, json_parser::JSONParser};

#[derive(Clone, Debug, PartialEq)]
pub enum Parser {
    JSON(Value),
    Length,
    Operator(Vec<(Operator, String)>),
    Nil,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Addition,
    Subtration,
    Multiplication,
    Division,
    Modulo,
    Nil,
}

impl Parser {
    pub fn new(json_data: Value, query: &str) -> Value {
        todo!()
    }

    fn parse(json_data: &Value, data: &str) -> Vec<Parser> {
        let parser = Regex::new(r"\s*(?P<pre>.*?)(\s*)\|(\s*)(?P<post>.*)\s*").unwrap();

        let mut parsed_json_type = vec![];

        let (mut pre, mut post) = Self::regexer(&parser, data);
        parsed_json_type.push(Parser::parse_pipe(json_data, pre.to_string()));

        while post != None {
            (pre, post) = Self::regexer(&parser, post.unwrap());
            parsed_json_type.push(Parser::parse_pipe(json_data, pre.to_string()));
        }

        parsed_json_type
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

    fn parse_pipe(json_data: &Value, data: String) -> Self {
        let length_compatibily = Regex::new(r"length").unwrap();
        let operator_compatibily =
            Regex::new(r"\s*(?P<pre>.*?)\s*(?P<operator>\+|\*|/|%|-)\s*(?P<post>.*)\s*").unwrap();

        if length_compatibily.is_match(&data) {
            return Parser::Length;
        } else if operator_compatibily.is_match(&data) {
            let mut operators = vec![];

            let mut data = data;
            loop {
                if !operator_compatibily.is_match(&data) {
                    if !data.is_empty() {
                        operators.push((Operator::Nil, data.to_string()));
                    }

                    return Parser::Operator(operators);
                }

                let (pre, post, operator) = {
                    if let Some(capture) = operator_compatibily.captures(&data) {
                        let pre = {
                            if let Some(op) = capture.name("pre") {
                                op.as_str()
                            } else {
                                ""
                            }
                        };

                        let post = {
                            if let Some(op) = capture.name("post") {
                                op.as_str()
                            } else {
                                ""
                            }
                        };

                        let operator = {
                            if let Some(op) = capture.name("operator") {
                                op.as_str()
                            } else {
                                ""
                            }
                        };

                        (pre, post, operator)
                    } else {
                        ("", "", "")
                    }
                };

                let operator = Operator::from(operator);

                operators.push((operator, pre.to_string()));

                data = post.to_string();
            }
        }

        let json_parser = JSONParser::new();
        let (json_value, output) = json_parser.parse(json_data, data);

        Parser::JSON(json_value)
    }
}

impl Parser {}

impl From<&str> for Operator {
    fn from(val: &str) -> Self {
        match val {
            "+" => Self::Addition,
            "-" => Self::Subtration,
            "/" => Self::Division,
            "*" => Self::Multiplication,
            "%" => Self::Modulo,
            _ => Self::Nil,
        }
    }
}

// mod test_parser {
//     use super::*;
//     struct TestParser {
//         query: String,
//         json_types: Vec<Parser>,
//     }

//     #[test]
//     fn test_four_pipes() {
//         let tests = [
//             TestParser {
//                 query: String::from(". | {a: .a} + {b: .b} + {c: .c} + {a: .c}|[.] |length"),
//                 json_types: vec![
//                     Parser::String(vec![ParsedFilter::Nil]),
//                     Parser::Operator(vec![
//                         (Operator::Addition, String::from("{a: .a}")),
//                         (Operator::Addition, String::from("{b: .b}")),
//                         (Operator::Addition, String::from("{c: .c}")),
//                         (Operator::Nil, String::from("{a: .c}")),
//                     ]),
//                     Parser::Output(ParsedOutput::Array(String::from("."))),
//                     Parser::Length,
//                 ],
//             },
//             TestParser {
//                 query: String::from(". | {a: .a} +{b: .b} + {c: .c} + {a: .c} |[.]  | length"),
//                 json_types: vec![
//                     Parser::String(vec![ParsedFilter::Nil]),
//                     Parser::Operator(vec![
//                         (Operator::Addition, String::from("{a: .a}")),
//                         (Operator::Addition, String::from("{b: .b}")),
//                         (Operator::Addition, String::from("{c: .c}")),
//                         (Operator::Nil, String::from("{a: .c}")),
//                     ]),
//                     Parser::Output(ParsedOutput::Array(String::from("."))),
//                     Parser::Length,
//                 ],
//             },
//             TestParser {
//                 query: String::from(". | {a: .a}+{b: .b}+{c: .c}+{a: .c} | [.] | length"),
//                 json_types: vec![
//                     Parser::String(vec![ParsedFilter::Nil]),
//                     Parser::Operator(vec![
//                         (Operator::Addition, String::from("{a: .a}")),
//                         (Operator::Addition, String::from("{b: .b}")),
//                         (Operator::Addition, String::from("{c: .c}")),
//                         (Operator::Nil, String::from("{a: .c}")),
//                     ]),
//                     Parser::Output(ParsedOutput::Array(String::from("."))),
//                     Parser::Length,
//                 ],
//             },
//         ];

//         for (i, test) in tests.into_iter().enumerate() {
//             let parsed = Parser::parse(&test.query);
//             assert_eq!(parsed, test.json_types, "Failed testing index {}", i);
//         }
//     }

//     #[test]
//     fn test_nil_parser() {
//         let tests = [
//             TestParser {
//                 query: String::from("."),
//                 json_types: vec![Parser::String(vec![ParsedFilter::Nil])],
//             },
//             TestParser {
//                 query: String::from(" ."),
//                 json_types: vec![Parser::String(vec![ParsedFilter::Nil])],
//             },
//             TestParser {
//                 query: String::from(" . "),
//                 json_types: vec![Parser::String(vec![ParsedFilter::Nil])],
//             },
//         ];

//         for (i, test) in tests.into_iter().enumerate() {
//             let parsed = Parser::parse(&test.query);
//             assert_eq!(parsed, test.json_types, "Failed testing index {}", i);
//         }
//     }

//     #[test]
//     fn test_nil_with_length() {
//         let tests = [
//             TestParser {
//                 query: String::from(". | length"),
//                 json_types: vec![Parser::String(vec![ParsedFilter::Nil]), Parser::Length],
//             },
//             TestParser {
//                 query: String::from(" .|length"),
//                 json_types: vec![Parser::String(vec![ParsedFilter::Nil]), Parser::Length],
//             },
//             TestParser {
//                 query: String::from(".|length"),
//                 json_types: vec![Parser::String(vec![ParsedFilter::Nil]), Parser::Length],
//             },
//         ];

//         for (i, test) in tests.into_iter().enumerate() {
//             let parsed = Parser::parse(&test.query);
//             assert_eq!(parsed, test.json_types, "Failed testing index {}", i);
//         }
//     }

//     #[test]
//     fn test_piped_operator() {
//         let tests = [
//             TestParser {
//                 query: String::from(". | {a: .a} + {b: .b} + {c: .c} + {a: .c}"),
//                 json_types: vec![
//                     Parser::String(vec![ParsedFilter::Nil]),
//                     Parser::Operator(vec![
//                         (Operator::Addition, String::from("{a: .a}")),
//                         (Operator::Addition, String::from("{b: .b}")),
//                         (Operator::Addition, String::from("{c: .c}")),
//                         (Operator::Nil, String::from("{a: .c}")),
//                     ]),
//                 ],
//             },
//             TestParser {
//                 query: String::from(". | {a: .a} +{b: .b} + {c: .c} + {a: .c}"),
//                 json_types: vec![
//                     Parser::String(vec![ParsedFilter::Nil]),
//                     Parser::Operator(vec![
//                         (Operator::Addition, String::from("{a: .a}")),
//                         (Operator::Addition, String::from("{b: .b}")),
//                         (Operator::Addition, String::from("{c: .c}")),
//                         (Operator::Nil, String::from("{a: .c}")),
//                     ]),
//                 ],
//             },
//             TestParser {
//                 query: String::from(". | {a: .a}+{b: .b}+{c: .c}+{a: .c} | length"),
//                 json_types: vec![
//                     Parser::String(vec![ParsedFilter::Nil]),
//                     Parser::Operator(vec![
//                         (Operator::Addition, String::from("{a: .a}")),
//                         (Operator::Addition, String::from("{b: .b}")),
//                         (Operator::Addition, String::from("{c: .c}")),
//                         (Operator::Nil, String::from("{a: .c}")),
//                     ]),
//                     Parser::Length,
//                 ],
//             },
//         ];

//         for (i, test) in tests.into_iter().enumerate() {
//             let parsed = Parser::parse(&test.query);
//             assert_eq!(parsed, test.json_types, "Failed testing index {}", i);
//         }
//     }
// }

// mod test_json_types {
//     use super::*;
//     struct TestParser {
//         query: String,
//         json_types: Vec<Parser>,
//     }

//     #[test]
//     fn parse_array() {
//         let data = "[ .hello]";
//         assert_eq!(
//             Parser::parse_pipe(data),
//             Parser::Output(ParsedOutput::Array(".hello".to_string()))
//         );
//     }
//     #[test]
//     fn parse_array_with_json_output() {
//         let data = "[{michael_said: .hello}]";
//         let array_content = Parser::parse_pipe(data);
//         match array_content {
//             Parser::Output(ParsedOutput::Array(e)) => {
//                 println!("{e}");
//                 assert_eq!(
//                     Parser::parse_pipe(&e),
//                     Parser::Output(ParsedOutput::Json("michael_said: .hello".to_string()))
//                 );
//             }
//             _ => unreachable!(),
//         }
//     }

//     #[test]
//     fn appropriately_escape_space() {
//         let data = "   [  {michael_said: .hello}  ]  ";
//         let array_content = Parser::parse_pipe(data);
//         println!("{:?}", array_content);
//         match array_content {
//             Parser::Output(ParsedOutput::Array(e)) => {
//                 println!("{e}");
//                 assert_eq!(
//                     Parser::parse_pipe(&e),
//                     Parser::Output(ParsedOutput::Json("michael_said: .hello".to_string()))
//                 );
//             }
//             _ => unreachable!(),
//         }
//     }

//     #[test]
//     fn find_length() {
//         let tests = [
//             TestParser {
//                 query: String::from("length"),
//                 json_types: vec![Parser::Length],
//             },
//             TestParser {
//                 query: String::from(" length"),
//                 json_types: vec![Parser::Length],
//             },
//         ];

//         for (i, test) in tests.iter().enumerate() {
//             let parsed = Parser::parse_pipe(&test.query);
//             assert_eq!(vec![parsed], test.json_types, "Failed testing index {}", i);
//         }
//     }

//     #[test]
//     fn find_operators() {
//         let tests = [
//             TestParser {
//                 query: String::from("{a: 1} + {b: 2} + {c: 3} + {a: 42}"),
//                 json_types: vec![Parser::Operator(vec![
//                     (Operator::Addition, String::from("{a: 1}")),
//                     (Operator::Addition, String::from("{b: 2}")),
//                     (Operator::Addition, String::from("{c: 3}")),
//                     (Operator::Nil, String::from("{a: 42}")),
//                 ])],
//             },
//             TestParser {
//                 query: String::from("{a: 1}+{b: 2}%{c: 3}-{a: 42}"),
//                 json_types: vec![Parser::Operator(vec![
//                     (Operator::Addition, String::from("{a: 1}")),
//                     (Operator::Modulo, String::from("{b: 2}")),
//                     (Operator::Subtration, String::from("{c: 3}")),
//                     (Operator::Nil, String::from("{a: 42}")),
//                 ])],
//             },
//         ];

//         for (i, test) in tests.iter().enumerate() {
//             let parsed = Parser::parse_pipe(&test.query);
//             assert_eq!(parsed, test.json_types[0], "Failed testing index {}", i);
//         }
//     }

//     #[test]
//     fn find_strings() {
//         let tests = [
//             TestParser {
//                 query: String::from(".hello.shell"),
//                 json_types: vec![Parser::String(vec![
//                     ParsedFilter::Filter(String::from("hello")),
//                     ParsedFilter::Filter(String::from("shell")),
//                 ])],
//             },
//             TestParser {
//                 query: String::from(".hello.shell[0]"),
//                 json_types: vec![Parser::String(vec![
//                     ParsedFilter::Filter(String::from("hello")),
//                     ParsedFilter::IndexedFilter {
//                         index: 0,
//                         name: String::from("shell"),
//                     },
//                 ])],
//             },
//             TestParser {
//                 query: String::from(".hello.shell [1001]"),
//                 json_types: vec![Parser::String(vec![
//                     ParsedFilter::Filter(String::from("hello")),
//                     ParsedFilter::IndexedFilter {
//                         index: 1001,
//                         name: String::from("shell"),
//                     },
//                 ])],
//             },
//             TestParser {
//                 query: String::from(".[0]"),
//                 json_types: vec![Parser::String(vec![ParsedFilter::IndexedFilter {
//                     index: 0,
//                     name: String::from(""),
//                 }])],
//             },
//             TestParser {
//                 query: String::from("."),
//                 json_types: vec![Parser::String(vec![ParsedFilter::Nil])],
//             },
//         ];

//         for (i, test) in tests.iter().enumerate() {
//             let parsed = Parser::parse_pipe(&test.query);
//             assert_eq!(parsed, test.json_types[0], "Failed testing index {}", i);
//         }
//     }
// }
