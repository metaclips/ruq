use regex::{Match, Regex};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum Parser {
    String(Vec<ParsedString>),
    Pipe(Vec<JSONType>),
    Nil,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParsedString {
    String(String),
    IndexedString { name: String, index: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub enum JSONType {
    Array(String),
    Output(String),
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
    pub fn parse(data: &str) -> Parser {
        let string_compatibility =
            Regex::new(r"(:?\s*)(\.(?P<string>\w*)(\[(?P<index>\d+)\])?)(:?\s*)(?P<others>.*)?")
                .unwrap();

        if string_compatibility.is_match(data) {
            let mut words = vec![];
            let mut data = data;

            loop {
                if data.is_empty() {
                    break;
                }

                let capture = string_compatibility.captures(data).unwrap();

                let value = {
                    let mut word = capture.name("string").unwrap().as_str();
                    remove_whitespace(&mut word);

                    if let Some(index) = capture.name("index") {
                        let index = index.as_str().parse::<usize>().unwrap();
                        ParsedString::IndexedString {
                            name: word.to_string(),
                            index,
                        }
                    } else {
                        ParsedString::String(word.to_string())
                    }
                };

                words.push(value);

                if let Some(other_chars) = capture.name("others") {
                    data = other_chars.as_str();
                } else {
                    break;
                }
            }

            return Parser::String(words);
        }

        Parser::Nil
    }
}

impl JSONType {
    pub fn parse(data: &str) -> JSONType {
        let array_compatibility =
            Regex::new(r"(:?\s*)\[(:?\s*)(?P<value>.*)(:?\s*)](:?\s*)").unwrap();
        let output_compatibility = Regex::new(r"(?:\s*)?(?P<value>.*)(:?\s*)?").unwrap();
        let length_compatibily = Regex::new(r".*\|?(?:\s*)length(?:\s*)\|?(.*)").unwrap();
        let operator_compatibily =
            Regex::new(r"((?P<pre>.*)(?P<Operator>\+|\*|/|%|-)(?P<post>.*))+").unwrap();

        if array_compatibility.is_match(data) {
            let mut array_characters = array_compatibility
                .captures(data)
                .unwrap()
                .name("value")
                .unwrap()
                .as_str();

            remove_whitespace(&mut array_characters);
            return JSONType::Array(array_characters.to_string());
        } else if operator_compatibily.is_match(data) {
            let mut operators = vec![];

            let mut data = data;
            loop {
                let (pre, mut post, operator) = {
                    if let Some(capture) = operator_compatibily.captures(data) {
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
                            if let Some(op) = capture.name("Operator") {
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

                match operator {
                    Operator::Nil => {
                        remove_whitespace(&mut data);
                        operators.push((operator, data.to_string()));

                        operators.reverse();
                        return JSONType::Operator(operators);
                    }
                    _ => {
                        remove_whitespace(&mut post);
                        operators.push((operator, post.to_string()));
                    }
                }

                data = pre;
            }
        } else if length_compatibily.is_match(data) {
            return JSONType::Length;
        } else if output_compatibility.is_match(data) {
            let mut output_characters = output_compatibility
                .captures(data)
                .unwrap()
                .name("value")
                .unwrap()
                .as_str();

            remove_whitespace(&mut output_characters);
            return JSONType::Output(output_characters.to_string());
        }

        JSONType::Nil
    }
}

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

fn remove_whitespace(data: &mut &str) {
    *data = data.trim_start_matches(char::is_whitespace);
    *data = data.trim_end_matches(char::is_whitespace);
}

mod test {
    use super::*;
    struct TestJSONType {
        query: String,
        json_types: Vec<JSONType>,
    }

    struct TestParserType {
        query: String,
        json_types: Vec<Parser>,
    }

    #[test]
    fn parse_dot() {
        // let data = ".";
        // assert_eq!(parser::parse(data), parser::Nil);
    }

    #[test]
    fn parse_array() {
        let data = "[.hello]";
        assert_eq!(JSONType::parse(data), JSONType::Array(".hello".to_string()));
    }
    #[test]
    fn parse_array_with_json_output() {
        let data = "[{michael_said: .hello}]";
        let array_content = JSONType::parse(data);
        println!("Done here");
        match array_content {
            JSONType::Array(e) => {
                println!("{e}");
                assert_eq!(
                    JSONType::parse(&e),
                    JSONType::Output("{michael_said: .hello}".to_string())
                );
            }
            _ => unreachable!(),
        }
    }
    #[test]
    fn appropriately_escape_space() {
        let data = "   [  {michael_said: .hello}  ]  ";
        let array_content = JSONType::parse(data);
        println!("Done here");
        match array_content {
            JSONType::Array(e) => {
                let output = JSONType::parse(&e);
                assert_eq!(
                    output,
                    JSONType::Output("{michael_said: .hello}".to_string())
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn find_length() {
        let tests = [
            TestJSONType {
                query: String::from(". | length"),
                json_types: vec![JSONType::Length],
            },
            TestJSONType {
                query: String::from(" length"),
                json_types: vec![JSONType::Length],
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let mut parsed_collection = vec![];

            let mut parsed = JSONType::parse(&test.query);
            loop {
                parsed_collection.push(parsed.clone());
                match parsed {
                    JSONType::Array(e) => {
                        parsed = JSONType::parse(&e);
                    }
                    JSONType::Output(e) => {
                        parsed = JSONType::parse(&e);
                    }
                    JSONType::Operator(e) => break,
                    JSONType::Length => break,
                    JSONType::Nil => break,
                }
            }

            assert_eq!(
                parsed_collection, test.json_types,
                "Failed testing index {}",
                i
            );
        }
    }

    #[test]
    fn find_operators() {
        let tests = [
            TestJSONType {
                query: String::from("{a: 1} + {b: 2} + {c: 3} + {a: 42}"),
                json_types: vec![JSONType::Operator(vec![
                    (Operator::Nil, String::from("{a: 1}")),
                    (Operator::Addition, String::from("{b: 2}")),
                    (Operator::Addition, String::from("{c: 3}")),
                    (Operator::Addition, String::from("{a: 42}")),
                ])],
            },
            TestJSONType {
                query: String::from("{a: 1}+{b: 2}%{c: 3}-{a: 42}"),
                json_types: vec![JSONType::Operator(vec![
                    (Operator::Nil, String::from("{a: 1}")),
                    (Operator::Addition, String::from("{b: 2}")),
                    (Operator::Modulo, String::from("{c: 3}")),
                    (Operator::Subtration, String::from("{a: 42}")),
                ])],
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let parsed = JSONType::parse(&test.query);
            assert_eq!(parsed, test.json_types[0], "Failed testing index {}", i);
        }
    }

    #[test]
    fn find_strings() {
        let tests = [
            TestParserType {
                query: String::from(".hello.shell"),
                json_types: vec![Parser::String(vec![
                    ParsedString::String(String::from("hello")),
                    ParsedString::String(String::from("shell")),
                ])],
            },
            TestParserType {
                query: String::from(".hello.shell[0]"),
                json_types: vec![Parser::String(vec![
                    ParsedString::String(String::from("hello")),
                    ParsedString::IndexedString {
                        index: 0,
                        name: String::from("shell"),
                    },
                ])],
            },
            TestParserType {
                query: String::from(".[0]"),
                json_types: vec![Parser::String(vec![
                    ParsedString::IndexedString {
                        index: 0,
                        name: String::from(""),
                    },
                ])],
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let parsed = Parser::parse(&test.query);
            assert_eq!(parsed, test.json_types[0], "Failed testing index {}", i);
        }
    }
}
