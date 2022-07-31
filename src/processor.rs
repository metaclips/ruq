use super::json_processor::Json;

pub trait Processor {
    type T;
    fn from_json(json_data: serde_json::Value) -> Self::T;
    fn to_json(&self) -> Json;
}
