use regex::{Match, Regex};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum Parser {
    String(String),
    Pipe(String),
    Index(String, usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JSONType {
    Array(String),
    Output(String),
    Length,
    Operand(Vec<(Operand, String)>),
    Nil,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Addition,
    Subtration,
    Multiplication,
    Division,
    Modulo,
    Nil,
}

impl From<&str> for Operand {
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

impl Parser {
    pub fn parse(data: &str) -> Parser {
        // Check if string is to processed as an array
        let mut data = &data[1..data.len() - 2];

        // match data {

        // }

        todo!()
    }
}

fn remove_whitespace(data: &mut &str) {
    *data = data.trim_start_matches(char::is_whitespace);
    *data = data.trim_end_matches(char::is_whitespace);
}

impl JSONType {
    pub fn parse(data: &str) -> JSONType {
        let array_compatibility =
            Regex::new(r"(:?\s*)\[(:?\s*)(?P<value>.*)(:?\s*)](:?\s*)").unwrap();
        let output_compatibility =
            Regex::new(r"(?:\s*)?.*\|(?:\s*)?(?P<value>.*)(:?\s*)?").unwrap();
        let length_compatibily = Regex::new(r".*\|?(?:\s*)?length").unwrap();
        let operand_compatibily =
            Regex::new(r"((?P<pre>.*)(?P<operand>\+|\*|/|%)(?P<post>.*))+").unwrap();

        if array_compatibility.is_match(data) {
            let mut array_characters = array_compatibility
                .captures(data)
                .unwrap()
                .name("value")
                .unwrap()
                .as_str();

            remove_whitespace(&mut array_characters);
            return JSONType::Array(array_characters.to_string());
        } else if operand_compatibily.is_match(data) {
            let mut operands = vec![];

            let mut data = data;
            loop {
                let (pre, mut post, operator) = {
                    if let Some(capture) = operand_compatibily.captures(data) {
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
                            if let Some(op) = capture.name("operand") {
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

                let operator = Operand::from(operator);

                match operator {
                    Operand::Nil => {
                        remove_whitespace(&mut data);
                        operands.push((operator, data.to_string()));

                        operands.reverse();
                        return JSONType::Operand(operands);
                    }
                    _ => {
                        remove_whitespace(&mut post);
                        operands.push((operator, post.to_string()));
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

mod test {
    use super::*;
    struct Test {
        query: String,
        json_types: Vec<JSONType>,
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
        let data = "[.hello | {michael_said: .hello}]";
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
        let data = "   [  .hello   |   {michael_said: .hello}  ]  ";
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
            Test {
                query: String::from(". | length"),
                json_types: vec![JSONType::Length],
            },
            Test {
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
                    JSONType::Operand(e) => break,
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
    fn find_operands() {
        let tests = [
            Test {
                query: String::from("{a: 1} + {b: 2} + {c: 3} + {a: 42}"),
                json_types: vec![JSONType::Operand(vec![
                    (Operand::Nil, String::from("{a: 1}")),
                    (Operand::Addition, String::from("{b: 2}")),
                    (Operand::Addition, String::from("{c: 3}")),
                    (Operand::Addition, String::from("{a: 42}")),
                ])],
            },
            Test {
                query: String::from("{a: 1}+{b: 2}+{c: 3}+{a: 42}"),
                json_types: vec![JSONType::Operand(vec![
                    (Operand::Nil, String::from("{a: 1}")),
                    (Operand::Addition, String::from("{b: 2}")),
                    (Operand::Addition, String::from("{c: 3}")),
                    (Operand::Addition, String::from("{a: 42}")),
                ])],
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let parsed = JSONType::parse(&test.query);

            assert_eq!(parsed, test.json_types[0], "Failed testing index {}", i);
        }
    }
}
