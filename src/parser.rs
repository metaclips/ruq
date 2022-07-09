use regex::Regex;
use std::fmt::Debug;

pub enum Parser {
    String(String),
    Pipe(String),
    Index(String, usize),
}

pub enum JSONType {
    Array(String),
    Output(String),
    Length,
    Nil,
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

impl JSONType {
    pub fn parse(data: &str) -> JSONType {
        let array_compatibility =
            Regex::new(r"(:?\s*)\[(:?\s*)(?P<value>.*)(:?\s*)](:?\s*)").unwrap();
        let output_compatibility =
            Regex::new(r"(?:\s*)?.*\|(?:\s*)?(?P<value>.*)(:?\s*)?").unwrap();

        if array_compatibility.is_match(data) {
            let mut array_characters = array_compatibility
                .captures(data)
                .unwrap()
                .name("value")
                .unwrap()
                .as_str();

            array_characters = array_characters.trim_start_matches(char::is_whitespace);
            array_characters = array_characters.trim_end_matches(char::is_whitespace);

            return JSONType::Array(array_characters.to_string());
        } else if output_compatibility.is_match(data) {
            let mut output_characters = output_compatibility
                .captures(data)
                .unwrap()
                .name("value")
                .unwrap()
                .as_str();

            output_characters = output_characters.trim_start_matches(char::is_whitespace);
            output_characters = output_characters.trim_end_matches(char::is_whitespace);

            return JSONType::Output(output_characters.to_string());
        }

        JSONType::Nil
    }
}

impl Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for Parser {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(s), Self::String(t)) => {
                if s == t {
                    return true;
                }

                false
            }
            (Self::Pipe(s), Self::Pipe(t)) => {
                if s == t {
                    return true;
                }

                false
            }
            (Self::Index(s, i), Self::Index(t, j)) => {
                if (s == t) && (i == j) {
                    return true;
                }

                false
            }
            _ => false,
        }
    }
}

impl PartialEq for JSONType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(s), Self::Array(t)) => {
                if s == t {
                    return true;
                }

                false
            }
            (Self::Output(s), Self::Output(t)) => {
                if s == t {
                    return true;
                }

                false
            }
            (Self::Length, Self::Length) => true,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl Debug for JSONType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(s) => writeln!(f, "JSON type of Array, with inner string {}", s),
            Self::Output(s) => writeln!(f, "JSON type of Output, with inner string {}", s),
            Self::Length => writeln!(f, "JSON type of Length"),
            Self::Nil => writeln!(f, "JSON type of Nil"),
        }
    }
}

mod test {
    use super::*;

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
}
