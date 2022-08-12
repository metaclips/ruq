use std::any::Any;

use super::parser::{Operator, Parser};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Json {
    pub data: Value,
}

impl Json {
    pub fn json_data_operator(mut json: Vec<(Operator, Value)>) -> Value {
        let len = json.len();
        let (mut recent_operator, mut recent_value) = json.first().unwrap().clone();

        for (operator, value) in json.drain(1..) {
            let value = match recent_operator {
                Operator::Addition => Self::add_json_data(recent_value, value),
                Operator::Subtration => Self::subtract_json_data(recent_value, value),
                Operator::Multiplication => {
                    todo!()
                }
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
}
