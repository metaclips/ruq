use serde_json::Map;

use crate::processor::Processor;

use super::parser::{Operator, Parser};

#[derive(Debug, Clone)]
pub struct Json {
    pub data: serde_json::Value,
}

impl Json {
    fn parse(data: &str) -> Vec<Parser> {
        Parser::parse(data)
    }

    fn json_data_operator(json: Vec<(Operator, String)>) -> serde_json::Value {
        for (operator, value) in json {
            match operator {
                Operator::Addition => {}
                Operator::Subtration => {}
                Operator::Multiplication => {}
                Operator::Division => {}
                Operator::Modulo => {}
                Operator::Nil => {}
            }
        }

        todo!()
        // let mut map = Map::new();

        // for mut a in json {
        //     let b = a.as_object_mut().unwrap();
        //     map.append(b);
        // }

        // serde_json::to_value(&map).unwrap()
    }

    // fn subtract_json_data(json: Vec<serde_json::Value>) -> serde_json::Value {}
}
